//! `transfer_fixed_delegation`, converted to the World/scenario pattern.
//!
//! Each test builds its own `World`, draws actors from the cast (`alice` the
//! delegator, `bob` the delegatee, `charlie` a third party, `mallory` the
//! adversary), stages the fixed delegation through `setup_fixed_delegation`, and
//! pulls against it through the observed `send_*`. Every send renders its surface
//! into the test's report under `target/md-reports/`.

use litesvm_utils::{LiteSvmBackend, TestSVM};

use crate::{
    event_engine::event_authority_pda,
    instructions::transfer_fixed_delegation,
    state::{header::VERSION_OFFSET, FixedDelegation},
    tests::{
        constants::{MINT_DECIMALS, PROGRAM_ID, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID},
        idl,
        pda::get_subscription_authority_pda,
        utils::{
            days, token_balance, init_aux_token_account, init_mint, install_transfer_hook_extra_metas,
            load_transfer_hook_example, set_transfer_hook_config, CloseSubscriptionAuthority, CreateDelegation,
            ObservedResultExt, RevokeDelegation, TransferDelegation, make_backend, World, TRANSFER_HOOK_EXAMPLE_PROGRAM_ID,
        },
    },
    SubscriptionsError,
};
use litesvm_utils::Keypair;
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use spl_associated_token_account_interface::address::get_associated_token_address_with_program_id;
use spl_token_2022_interface::extension::ExtensionType;
use spl_token_interface::instruction::TokenInstruction::Approve;

/// Stage a fixed delegation: Alice initializes her authority over a fresh SPL
/// mint, funds her ATA, and creates a fixed delegation to Bob. Mirrors the
/// suite's `setup_fixed_delegation`, but every send is observed. Returns the cast
/// and the derived accounts (minus the LiteSVM, which the World owns).
fn setup_fixed_delegation(
    world: &mut World<LiteSvmBackend>,
    amount: u64,
    expiry_ts: i64,
    nonce: u64,
) -> (Keypair, Keypair, Pubkey, Pubkey, Pubkey, Pubkey) {
    let alice = world.actor("alice");
    let bob = world.actor("bob");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let alice_ata = world.fund_ata(mint, &alice, 100_000_000);
    let bob_ata = world.fund_ata(mint, &bob, 0);

    world.md().step("Stage: Alice's authority and a fixed delegation to Bob");
    world.init_authority(&alice, mint, None).0.assert_ok();

    let (ix, delegation_pda) =
        CreateDelegation::new(world.svm_mut(), &alice, mint, bob.pubkey()).nonce(nonce).fixed_ix(amount, expiry_ts);
    world.prop(delegation_pda, "FixedDelegation");
    world.send_ok(&[ix], &[&alice], "CreateFixedDelegation");

    (alice, bob, delegation_pda, mint, alice_ata, bob_ata)
}

#[test]
fn test_fixed_transfer_success() {
    let mut world = World::new(make_backend(), 
        "Fixed transfer succeeds",
        "Bob pulls part of his fixed allowance; the remaining allowance decrements",
    );
    let amount: u64 = 50_000_000;
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce = 0;

    let (alice, bob, delegation_pda, mint, _alice_ata, bob_ata) =
        setup_fixed_delegation(&mut world, amount, expiry_ts, nonce);

    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob's ATA starts empty", 0, bob_balance);

    world.md().step("Bob pulls 30 tokens against his fixed delegation");
    let transfer_amount: u64 = 30_000_000;
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(transfer_amount)
        .fixed_ix();
    world.send_ok(&[ix], &[&bob], "TransferFixed");

    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob received 30 tokens", 30_000_000, bob_balance);

    let delegation_account = world.svm().get_account(&delegation_pda).unwrap();
    let delegation = FixedDelegation::load(&delegation_account.data).unwrap();
    let del_amount = delegation.amount;
    let del_expiry = delegation.expiry_ts;
    world.md().check("the remaining allowance is 20 tokens", 20_000_000, del_amount);
    world.md().check("the expiry is unchanged", expiry_ts, del_expiry);
}

#[test]
fn test_fixed_transfer_token_2022_transfer_fee() {
    let mut world = World::new(make_backend(), 
        "Fixed transfer over a Token-2022 transfer-fee mint",
        "the transfer fee is withheld from the receiver; the allowance decrements by the gross amount",
    );
    let alice = world.actor("alice");
    let bob = world.actor("bob");

    let mint = init_mint(
        world.svm_mut(),
        TOKEN_2022_PROGRAM_ID,
        MINT_DECIMALS,
        1_000_000_000,
        Some(alice.pubkey()),
        &[ExtensionType::TransferFeeConfig],
    );
    world.prop(mint, "USDC mint (T22)");
    let alice_ata = world.fund_ata(mint, &alice, 100_000_000);
    let bob_ata = world.fund_ata(mint, &bob, 0);

    world.md().step("Stage: Alice's authority and a fixed delegation to Bob");
    world.init_authority(&alice, mint, None).0.assert_ok();

    let expiry_ts = world.now() + days(1) as i64;
    let (ix, delegation_pda) =
        CreateDelegation::new(world.svm_mut(), &alice, mint, bob.pubkey()).fixed_ix(50_000_000, expiry_ts);
    world.prop(delegation_pda, "FixedDelegation");
    world.send_ok(&[ix], &[&alice], "CreateFixedDelegation");

    world.md().step("Bob pulls 10 tokens; the transfer fee is withheld");
    let ix =
        TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda).amount(10_000_000).fixed_ix();
    world.send_ok(&[ix], &[&bob], "TransferFixed");

    let alice_balance = token_balance(world.svm(), &alice_ata);
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Alice's ATA debited the gross 10 tokens", 90_000_000, alice_balance);
    world.md().check("Bob received 10 tokens net of the 1% fee", 9_900_000, bob_balance);

    let delegation_account = world.svm().get_account(&delegation_pda).unwrap();
    let remaining_amount = FixedDelegation::load(&delegation_account.data).unwrap().amount;
    world.md().check("the allowance decrements by the gross 10 tokens", 40_000_000, remaining_amount);
}

#[test]
fn test_fixed_transfer_token_2022_confidential_transfer_public_balance() {
    let mut world = World::new(make_backend(), 
        "Fixed transfer over a Token-2022 confidential-transfer mint",
        "a transfer of the public balance over a confidential-transfer mint succeeds",
    );
    let alice = world.actor("alice");
    let bob = world.actor("bob");

    let mint = init_mint(
        world.svm_mut(),
        TOKEN_2022_PROGRAM_ID,
        MINT_DECIMALS,
        1_000_000_000,
        Some(alice.pubkey()),
        &[ExtensionType::ConfidentialTransferMint],
    );
    world.prop(mint, "USDC mint (T22)");
    let alice_ata = world.fund_ata(mint, &alice, 100_000_000);
    let bob_ata = world.fund_ata(mint, &bob, 0);

    world.md().step("Stage: Alice's authority and a fixed delegation to Bob");
    world.init_authority(&alice, mint, None).0.assert_ok();

    let expiry_ts = world.now() + days(1) as i64;
    let (ix, delegation_pda) =
        CreateDelegation::new(world.svm_mut(), &alice, mint, bob.pubkey()).fixed_ix(50_000_000, expiry_ts);
    world.prop(delegation_pda, "FixedDelegation");
    world.send_ok(&[ix], &[&alice], "CreateFixedDelegation");

    world.md().step("Bob pulls 10 tokens of the public balance");
    let ix =
        TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda).amount(10_000_000).fixed_ix();
    world.send_ok(&[ix], &[&bob], "TransferFixed");

    let alice_balance = token_balance(world.svm(), &alice_ata);
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Alice's ATA debited 10 tokens", 90_000_000, alice_balance);
    world.md().check("Bob received 10 tokens", 10_000_000, bob_balance);
}

#[test]
fn test_fixed_transfer_token_2022_unconfigured_transfer_hook() {
    let mut world = World::new(make_backend(), 
        "Fixed transfer over a Token-2022 unconfigured transfer-hook mint",
        "a mint carrying an unconfigured transfer hook still transfers",
    );
    let alice = world.actor("alice");
    let bob = world.actor("bob");

    let mint = init_mint(
        world.svm_mut(),
        TOKEN_2022_PROGRAM_ID,
        MINT_DECIMALS,
        1_000_000_000,
        Some(alice.pubkey()),
        &[ExtensionType::TransferHook],
    );
    world.prop(mint, "USDC mint (T22)");
    let alice_ata = world.fund_ata(mint, &alice, 100_000_000);
    let bob_ata = world.fund_ata(mint, &bob, 0);

    world.md().step("Stage: Alice's authority and a fixed delegation to Bob");
    world.init_authority(&alice, mint, None).0.assert_ok();

    let expiry_ts = world.now() + days(1) as i64;
    let (ix, delegation_pda) =
        CreateDelegation::new(world.svm_mut(), &alice, mint, bob.pubkey()).fixed_ix(50_000_000, expiry_ts);
    world.prop(delegation_pda, "FixedDelegation");
    world.send_ok(&[ix], &[&alice], "CreateFixedDelegation");

    world.md().step("Bob pulls 10 tokens; the unconfigured hook is a no-op");
    let ix =
        TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda).amount(10_000_000).fixed_ix();
    world.send_ok(&[ix], &[&bob], "TransferFixed");

    let alice_balance = token_balance(world.svm(), &alice_ata);
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Alice's ATA debited 10 tokens", 90_000_000, alice_balance);
    world.md().check("Bob received 10 tokens", 10_000_000, bob_balance);
}

#[test]
fn test_fixed_transfer_token_2022_active_transfer_hook() {
    let mut world = World::new(make_backend(), 
        "Fixed transfer over a Token-2022 active transfer-hook mint",
        "a transfer without the hook accounts fails; with them, the hook runs and the transfer succeeds",
    );
    let alice = world.actor("alice");
    load_transfer_hook_example(world.svm_mut());
    let bob = world.actor("bob");

    let mint = init_mint(
        world.svm_mut(),
        TOKEN_2022_PROGRAM_ID,
        MINT_DECIMALS,
        1_000_000_000,
        Some(alice.pubkey()),
        &[ExtensionType::TransferHook],
    );
    world.prop(mint, "USDC mint (T22)");
    set_transfer_hook_config(world.svm_mut(), mint, Some(alice.pubkey()), Some(TRANSFER_HOOK_EXAMPLE_PROGRAM_ID));
    let (validation_pda, counter) = install_transfer_hook_extra_metas(world.svm_mut(), mint);

    let alice_ata = world.fund_ata(mint, &alice, 100_000_000);
    let bob_ata = world.fund_ata(mint, &bob, 0);

    world.md().step("Stage: Alice's authority and a fixed delegation to Bob");
    world.init_authority(&alice, mint, None).0.assert_ok();
    let expiry_ts = world.now() + days(1) as i64;
    let (ix, delegation_pda) =
        CreateDelegation::new(world.svm_mut(), &alice, mint, bob.pubkey()).fixed_ix(50_000_000, expiry_ts);
    world.prop(delegation_pda, "FixedDelegation");
    world.send_ok(&[ix], &[&alice], "CreateFixedDelegation");

    world.md().step("Bob pulls without supplying the hook accounts; the transfer is refused");
    let ix =
        TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda).amount(10_000_000).fixed_ix();
    let missing_accounts = world.send(&[ix], &[&bob], "TransferFixed (missing hook accounts)");
    assert!(!missing_accounts.is_success(), "transfer without hook accounts should fail");
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob's ATA is still empty", 0, bob_balance);

    world.md().step("Bob retries with the hook accounts attached; the hook runs");
    let remaining = vec![
        AccountMeta::new_readonly(TRANSFER_HOOK_EXAMPLE_PROGRAM_ID, false),
        AccountMeta::new_readonly(validation_pda, false),
        AccountMeta::new(counter, false),
    ];
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(10_000_000)
        .remaining(remaining)
        .fixed_ix();
    world.send_ok(&[ix], &[&bob], "TransferFixed (with hook accounts)");

    let alice_balance = token_balance(world.svm(), &alice_ata);
    let bob_balance = token_balance(world.svm(), &bob_ata);
    let hook_runs = world.svm().get_account(&counter).unwrap().data[0];
    world.md().check("Alice's ATA debited 10 tokens", 90_000_000, alice_balance);
    world.md().check("Bob received 10 tokens", 10_000_000, bob_balance);
    world.md().check("the transfer hook ran once", 1, hook_runs);
}

#[test]
fn active_hook_transfer_without_validation_pda_fails() {
    let mut world = World::new(make_backend(), 
        "Active hook transfer without the validation PDA fails",
        "supplying the hook program but not its validation PDA is rejected",
    );
    let alice = world.actor("alice");
    load_transfer_hook_example(world.svm_mut());
    let bob = world.actor("bob");

    let mint = init_mint(
        world.svm_mut(),
        TOKEN_2022_PROGRAM_ID,
        MINT_DECIMALS,
        1_000_000_000,
        Some(alice.pubkey()),
        &[ExtensionType::TransferHook],
    );
    world.prop(mint, "USDC mint (T22)");
    set_transfer_hook_config(world.svm_mut(), mint, Some(alice.pubkey()), Some(TRANSFER_HOOK_EXAMPLE_PROGRAM_ID));
    install_transfer_hook_extra_metas(world.svm_mut(), mint);

    let _alice_ata = world.fund_ata(mint, &alice, 100_000_000);
    let _bob_ata = world.fund_ata(mint, &bob, 0);

    world.md().step("Stage: Alice's authority and a fixed delegation to Bob");
    world.init_authority(&alice, mint, None).0.assert_ok();
    let expiry_ts = world.now() + days(1) as i64;
    let (ix, delegation_pda) =
        CreateDelegation::new(world.svm_mut(), &alice, mint, bob.pubkey()).fixed_ix(50_000_000, expiry_ts);
    world.prop(delegation_pda, "FixedDelegation");
    world.send_ok(&[ix], &[&alice], "CreateFixedDelegation");

    world.md().step("Bob supplies the hook program but omits its validation PDA");
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(10_000_000)
        .remaining(vec![AccountMeta::new_readonly(TRANSFER_HOOK_EXAMPLE_PROGRAM_ID, false)])
        .fixed_ix();
    world.send_err(
        &[ix],
        &[&bob],
        "TransferFixed (missing validation PDA)",
        SubscriptionsError::TransferHookValidationAccountMissing,
    );
}

#[test]
fn test_fixed_transfer_multiple_times() {
    let mut world = World::new(make_backend(), 
        "Fixed transfer cannot exceed the allowance across pulls",
        "after one pull, a second pull for more than the remaining allowance is refused",
    );
    let amount: u64 = 50_000_000;
    let expiry_s: i64 = world.now() + days(1) as i64;
    let nonce = 1;

    let (alice, bob, delegation_pda, mint, _, bob_ata) = setup_fixed_delegation(&mut world, amount, expiry_s, nonce);

    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob's ATA starts empty", 0, bob_balance);

    world.md().step("Bob pulls 30 tokens, leaving 20");
    let transfer_amount: u64 = 30_000_000;
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(transfer_amount)
        .fixed_ix();
    world.send_ok(&[ix], &[&bob], "TransferFixed (first pull)");

    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob received 30 tokens", 30_000_000, bob_balance);

    let delegation_account = world.svm().get_account(&delegation_pda).unwrap();
    let del_amount = FixedDelegation::load(&delegation_account.data).unwrap().amount;
    world.md().check("the remaining allowance is 20 tokens", 20_000_000, del_amount);

    world.md().step("Bob tries to pull another 30 tokens; only 20 remain, so it is refused");
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(transfer_amount)
        .fixed_ix();
    world.send_err(
        &[ix],
        &[&bob],
        "TransferFixed (over the remaining allowance)",
        SubscriptionsError::AmountExceedsLimit,
    );

    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob's balance is unchanged", 30_000_000, bob_balance);

    let delegation_account = world.svm().get_account(&delegation_pda).unwrap();
    let del_amount = FixedDelegation::load(&delegation_account.data).unwrap().amount;
    world.md().check("the remaining allowance is still 20 tokens", 20_000_000, del_amount);
}

#[test]
fn test_fixed_transfer_exceeds_amount() {
    let mut world = World::new(make_backend(), 
        "Fixed transfer exceeding the allowance is refused",
        "a single pull for more than the full allowance is refused and leaves the delegation untouched",
    );
    let amount: u64 = 50_000_000;
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce = 1;

    let (alice, bob, delegation_pda, mint, _, bob_ata) = setup_fixed_delegation(&mut world, amount, expiry_ts, nonce);

    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob's ATA starts empty", 0, bob_balance);

    world.md().step("Bob tries to pull 60 tokens against a 50-token allowance");
    let transfer_amount: u64 = 60_000_000;
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(transfer_amount)
        .fixed_ix();
    world.send_err(&[ix], &[&bob], "TransferFixed (exceeds allowance)", SubscriptionsError::AmountExceedsLimit);

    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob's ATA is still empty", 0, bob_balance);

    let delegation_account = world.svm().get_account(&delegation_pda).unwrap();
    let del_amount = FixedDelegation::load(&delegation_account.data).unwrap().amount;
    world.md().check("the full allowance is intact", 50_000_000, del_amount);
}

#[test]
fn test_fixed_transfer_expired() {
    let mut world = World::new(make_backend(), 
        "Fixed transfer after expiry is refused",
        "a pull within the window succeeds; after the clock passes expiry, a further pull is refused",
    );
    let amount: u64 = 50_000_000;
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce = 1;

    let (alice, bob, delegation_pda, mint, _, bob_ata) = setup_fixed_delegation(&mut world, amount, expiry_ts, nonce);

    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob's ATA starts empty", 0, bob_balance);

    world.md().step("Bob pulls 30 tokens within the window");
    let transfer_amount: u64 = 30_000_000;
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(transfer_amount)
        .fixed_ix();
    world.send_ok(&[ix], &[&bob], "TransferFixed (within window)");
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob received 30 tokens", 30_000_000, bob_balance);

    world.md().step("The clock advances past expiry; a further pull is refused");
    world.warp(days(2));

    let transfer_amount: u64 = 30_000_000;
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(transfer_amount)
        .fixed_ix();
    world.send_err(&[ix], &[&bob], "TransferFixed (expired)", SubscriptionsError::DelegationExpired);
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob's balance is unchanged", 30_000_000, bob_balance);

    let delegation_account = world.svm().get_account(&delegation_pda).unwrap();
    let delegation_amount = FixedDelegation::load(&delegation_account.data).unwrap().amount;
    world.md().check("the remaining allowance is 20 tokens", 20_000_000, delegation_amount);
}

#[test]
fn test_fixed_transfer_wrong_signer() {
    let mut world = World::new(make_backend(), 
        "Fixed transfer by the wrong signer is refused",
        "Mallory, who is not the delegatee, cannot pull against Bob's delegation",
    );
    let amount: u64 = 50_000_000;
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce = 10;

    let (alice, _bob, delegation_pda, mint, _, bob_ata) = setup_fixed_delegation(&mut world, amount, expiry_ts, nonce);

    // Mallory is the unauthorized caller.
    let mallory = world.actor("mallory");

    world.md().step("Mallory, not the delegatee, tries to pull against the delegation");
    let transfer_amount: u64 = 10_000_000;
    let ix = TransferDelegation::new(world.svm_mut(), &mallory, alice.pubkey(), mint, delegation_pda)
        .amount(transfer_amount)
        .to(bob_ata)
        .fixed_ix();
    world.send_err(&[ix], &[&mallory], "TransferFixed (wrong signer)", SubscriptionsError::Unauthorized);
}

#[test]
fn test_fixed_transfer_to_third_party() {
    let mut world = World::new(make_backend(), 
        "Fixed transfer to a third party",
        "Bob, the delegatee, routes a pull to Charlie's account",
    );
    let amount: u64 = 50_000_000;
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce = 0;

    // Alice delegates to Bob.
    let (alice, bob, delegation_pda, mint, _, _) = setup_fixed_delegation(&mut world, amount, expiry_ts, nonce);

    // Charlie is a third party.
    let charlie = world.actor("charlie");
    let charlie_ata = world.fund_ata(mint, &charlie, 0);

    world.md().step("Bob pulls from Alice and routes the funds to Charlie");
    let transfer_amount: u64 = 10_000_000;
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(transfer_amount)
        .to(charlie_ata)
        .fixed_ix();
    world.send_ok(&[ix], &[&bob], "TransferFixed (to third party)");

    let charlie_balance = token_balance(world.svm(), &charlie_ata);
    world.md().check("Charlie received 10 tokens", 10_000_000, charlie_balance);
}

#[test]
fn fixed_delegation_rejects_transfer_with_different_mint_authority() {
    let mut world = World::new(make_backend(), 
        "Fixed delegation rejects a different-mint transfer",
        "a low-value delegation cannot be replayed against a high-value mint's accounts",
    );
    let alice = world.actor("alice");
    let bob = world.actor("bob");

    let low_value_mint =
        world.usdc_mint(&alice);
    world.prop(low_value_mint, "low-value mint");
    let high_value_mint =
        world.usdc_mint(&alice);
    world.prop(high_value_mint, "high-value mint");

    let _alice_low_ata = world.fund_ata(low_value_mint, &alice, 100_000_000);
    let alice_high_ata = world.fund_ata(high_value_mint, &alice, 100_000_000);
    let _bob_low_ata = world.fund_ata(low_value_mint, &bob, 0);
    let bob_high_ata = world.fund_ata(high_value_mint, &bob, 0);

    world.md().step("Stage: Alice's authority over both mints");
    world.init_authority(&alice, low_value_mint, None).0.assert_ok();
    world.init_authority(&alice, high_value_mint, None).0.assert_ok();

    let fixed_allowance = 50_000_000;
    let expiry_ts = world.now() + days(1) as i64;
    let (ix, low_value_delegation_pda) = CreateDelegation::new(world.svm_mut(), &alice, low_value_mint, bob.pubkey())
        .nonce(89)
        .fixed_ix(fixed_allowance, expiry_ts);
    world.prop(low_value_delegation_pda, "low-value delegation");
    world.send_ok(&[ix], &[&alice], "CreateFixedDelegation (low-value)");

    world.md().step("Bob replays the low-value delegation against the high-value mint; it is refused");
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), high_value_mint, low_value_delegation_pda)
        .amount(20_000_000)
        .fixed_ix();
    world.send_err(&[ix], &[&bob], "TransferFixed (mismatched mint)", SubscriptionsError::InvalidDelegatePda);

    let alice_balance = token_balance(world.svm(), &alice_high_ata);
    let bob_balance = token_balance(world.svm(), &bob_high_ata);
    world.md().check("Alice's high-value ATA is untouched", 100_000_000, alice_balance);
    world.md().check("Bob's high-value ATA is empty", 0, bob_balance);

    let delegation_account = world.svm().get_account(&low_value_delegation_pda).unwrap();
    let remaining_allowance = FixedDelegation::load(&delegation_account.data).unwrap().amount;
    world.md().check("the low-value allowance is intact", fixed_allowance, remaining_allowance);
}

#[test]
fn fixed_transfer_rejects_approved_non_canonical_source() {
    let mut world = World::new(make_backend(), 
        "Fixed transfer rejects a non-canonical source",
        "an auxiliary (non-ATA) source the authority was Approve'd over cannot be drained via a delegation",
    );
    let alice = world.actor("alice");
    let bob = world.actor("bob");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let alice_ata = world.fund_ata(mint, &alice, 5_000_000);
    let alice_aux = init_aux_token_account(world.svm_mut(), mint, alice.pubkey(), 100_000_000);
    world.prop(alice_aux, "Alice aux token account");
    let bob_ata = world.fund_ata(mint, &bob, 0);

    world.md().step("Stage: Alice's authority and a fixed delegation to Bob");
    let (res, subscription_authority_pda, _) = world.init_authority(&alice, mint, None);
    res.assert_ok();

    let fixed_allowance = 60_000_000;
    let expiry_ts = world.now() + days(1) as i64;
    let (ix, delegation_pda) =
        CreateDelegation::new(world.svm_mut(), &alice, mint, bob.pubkey()).nonce(87).fixed_ix(fixed_allowance, expiry_ts);
    world.prop(delegation_pda, "FixedDelegation");
    world.send_ok(&[ix], &[&alice], "CreateFixedDelegation");

    world.md().step("Alice Approve's the authority over her auxiliary account");
    let ix = Instruction {
        program_id: TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(alice_aux, false),
            AccountMeta::new(subscription_authority_pda, false),
            AccountMeta::new(alice.pubkey(), true),
        ],
        data: Approve { amount: u64::MAX }.pack(),
    };
    world.send_ok(&[ix], &[&alice], "Approve (aux account)");

    world.md().step("Bob points the delegation at the auxiliary source; it is refused");
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .from(alice_aux)
        .amount(10_000_000)
        .fixed_ix();
    world.send_err(
        &[ix],
        &[&bob],
        "TransferFixed (non-canonical source)",
        SubscriptionsError::InvalidAssociatedTokenAccountDerivedAddress,
    );

    let alice_balance = token_balance(world.svm(), &alice_ata);
    let alice_aux_balance = token_balance(world.svm(), &alice_aux);
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Alice's ATA is untouched", 5_000_000, alice_balance);
    world.md().check("Alice's aux account is untouched", 100_000_000, alice_aux_balance);
    world.md().check("Bob's ATA is empty", 0, bob_balance);

    let delegation_account = world.svm().get_account(&delegation_pda).unwrap();
    let remaining_allowance = FixedDelegation::load(&delegation_account.data).unwrap().amount;
    world.md().check("the allowance is intact", fixed_allowance, remaining_allowance);
}

#[test]
fn writable_accounts_must_be_writable() {
    let writable = idl::writable_account_indices("transferFixed");

    let mut world = World::new(make_backend(), 
        "Writable accounts must be writable (transferFixed)",
        "flipping any account the transfer writes to read-only is rejected",
    );
    let amount: u64 = 50_000_000;
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce = 0;

    let (alice, bob, delegation_pda, mint, _, _) = setup_fixed_delegation(&mut world, amount, expiry_ts, nonce);
    let sponsor = world.actor("sponsor");

    let (subscription_authority_pda, _) = get_subscription_authority_pda(&alice.pubkey(), &mint);
    let delegator_ata = get_associated_token_address_with_program_id(&alice.pubkey(), &mint, &TOKEN_PROGRAM_ID);
    let receiver_ata = get_associated_token_address_with_program_id(&bob.pubkey(), &mint, &TOKEN_PROGRAM_ID);
    let event_authority = Pubkey::new_from_array(event_authority_pda::ID.to_bytes());

    for (idx, name, is_signer) in &writable {
        let mut accounts = vec![
            AccountMeta::new(delegation_pda, false),
            AccountMeta::new_readonly(subscription_authority_pda, false),
            AccountMeta::new(delegator_ata, false),
            AccountMeta::new(receiver_ata, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            AccountMeta::new_readonly(bob.pubkey(), true),
            AccountMeta::new_readonly(event_authority, false),
            AccountMeta::new_readonly(PROGRAM_ID, false),
        ];

        // Flip writable account to readonly, preserving signer flag.
        let pubkey = accounts[*idx].pubkey;
        accounts[*idx] = AccountMeta::new_readonly(pubkey, *is_signer);

        let transfer_amount: u64 = 10_000_000;
        let data = [
            vec![*transfer_fixed_delegation::DISCRIMINATOR],
            transfer_amount.to_le_bytes().to_vec(),
            alice.pubkey().to_bytes().to_vec(),
            mint.to_bytes().to_vec(),
        ]
        .concat();

        let ix = Instruction { program_id: PROGRAM_ID, accounts, data };

        world.send_err(
            &[ix],
            &[&sponsor, &bob],
            &format!("TransferFixed ({name} forced read-only)"),
            SubscriptionsError::AccountNotWritable,
        );
    }
}

#[test]
fn signer_accounts_must_be_signers() {
    let signers = idl::signer_account_indices("transferFixed");

    let mut world = World::new(make_backend(), 
        "Signer accounts must sign (transferFixed)",
        "flipping any required signer to non-signer is rejected",
    );
    let amount: u64 = 50_000_000;
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce = 0;

    let (alice, bob, delegation_pda, mint, _, _) = setup_fixed_delegation(&mut world, amount, expiry_ts, nonce);
    let sponsor = world.actor("sponsor");

    let (subscription_authority_pda, _) = get_subscription_authority_pda(&alice.pubkey(), &mint);
    let delegator_ata = get_associated_token_address_with_program_id(&alice.pubkey(), &mint, &TOKEN_PROGRAM_ID);
    let receiver_ata = get_associated_token_address_with_program_id(&bob.pubkey(), &mint, &TOKEN_PROGRAM_ID);
    let event_authority = Pubkey::new_from_array(event_authority_pda::ID.to_bytes());

    for (idx, name, is_writable) in &signers {
        let mut accounts = vec![
            AccountMeta::new(delegation_pda, false),
            AccountMeta::new_readonly(subscription_authority_pda, false),
            AccountMeta::new(delegator_ata, false),
            AccountMeta::new(receiver_ata, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            AccountMeta::new_readonly(bob.pubkey(), true),
            AccountMeta::new_readonly(event_authority, false),
            AccountMeta::new_readonly(PROGRAM_ID, false),
        ];

        // Flip signer to non-signer, preserving writable flag.
        let pubkey = accounts[*idx].pubkey;
        accounts[*idx] =
            if *is_writable { AccountMeta::new(pubkey, false) } else { AccountMeta::new_readonly(pubkey, false) };

        let transfer_amount: u64 = 10_000_000;
        let data = [
            vec![*transfer_fixed_delegation::DISCRIMINATOR],
            transfer_amount.to_le_bytes().to_vec(),
            alice.pubkey().to_bytes().to_vec(),
            mint.to_bytes().to_vec(),
        ]
        .concat();

        let ix = Instruction { program_id: PROGRAM_ID, accounts, data };

        world.send_err(
            &[ix],
            &[&sponsor],
            &format!("TransferFixed ({name} forced non-signer)"),
            SubscriptionsError::NotSigner,
        );
    }
}

#[test]
fn test_fixed_transfer_delegator_mismatch_exploit() {
    // This test demonstrates the access control vulnerability where a malicious delegatee
    // can use their own delegation to transfer funds from another user's account.
    let mut world = World::new(make_backend(), 
        "Fixed transfer delegator-mismatch exploit is blocked",
        "Bob's self-delegation cannot be used to drain Alice's account by spoofing the delegator",
    );
    let amount: u64 = 50_000_000;
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce = 0;

    // Setup: Alice (victim) with funds and Bob (the malicious delegatee).
    let (alice, bob, _alice_delegation_pda, mint, alice_ata, bob_ata) =
        setup_fixed_delegation(&mut world, amount, expiry_ts, nonce);

    world.md().step("Bob initializes his own authority and a self-delegation");
    world.init_authority(&bob, mint, None).0.assert_ok();

    // Bob creates a self-delegation (Bob -> Bob) with a large allowance.
    let (ix, bob_delegation_pda) =
        CreateDelegation::new(world.svm_mut(), &bob, mint, bob.pubkey()).nonce(nonce).fixed_ix(1_000_000_000, expiry_ts);
    world.prop(bob_delegation_pda, "Bob self-delegation");
    world.send_ok(&[ix], &[&bob], "CreateFixedDelegation (Bob self)");

    world.md().step("Bob spoofs Alice as the delegator while using his own delegation; it is refused");
    let transfer_amount: u64 = 30_000_000;
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, bob_delegation_pda)
        .amount(transfer_amount)
        .to(bob_ata)
        .fixed_ix();
    world.send_err(&[ix], &[&bob], "TransferFixed (delegator mismatch)", SubscriptionsError::Unauthorized);

    let alice_balance = token_balance(world.svm(), &alice_ata);
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Alice's funds are untouched", 100_000_000, alice_balance);
    world.md().check("Bob received no funds", 0, bob_balance);
}

#[test]
fn test_fixed_transfer_version_mismatch() {
    let mut world = World::new(make_backend(), 
        "Fixed transfer over a stale-version delegation is refused",
        "a delegation whose header version byte was zeroed requires explicit migration",
    );
    let amount: u64 = 50_000_000;
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce = 0;

    let (alice, bob, delegation_pda, mint, _, bob_ata) = setup_fixed_delegation(&mut world, amount, expiry_ts, nonce);

    world.md().step("The delegation's header version byte is corrupted to 0");
    let mut account = world.svm().get_account(&delegation_pda).unwrap();
    account.data[VERSION_OFFSET] = 0;
    world.svm_mut().set_account(&delegation_pda, account);

    world.md().step("Bob pulls against the stale delegation; migration is required");
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(10_000_000)
        .fixed_ix();
    world.send_err(&[ix], &[&bob], "TransferFixed (version mismatch)", SubscriptionsError::MigrationRequired);
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob's ATA is empty", 0, bob_balance);
}

#[test]
fn test_fixed_transfer_stale_subscription_authority() {
    let mut world = World::new(make_backend(), 
        "Fixed transfer against a re-initialized authority is refused",
        "closing then re-initializing the authority bumps its init_id, staling the delegation",
    );
    let amount: u64 = 50_000_000;
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce = 0;

    let (alice, bob, delegation_pda, mint, alice_ata, bob_ata) =
        setup_fixed_delegation(&mut world, amount, expiry_ts, nonce);

    world.md().step("Alice closes and re-initializes her authority, bumping its init_id");
    let ix = CloseSubscriptionAuthority::new(world.svm_mut(), &alice, mint).instruction();
    world.send_ok(&[ix], &[&alice], "CloseSubscriptionAuthority");

    world.warp(2);

    world.init_authority(&alice, mint, None).0.assert_ok();

    world.md().step("Bob pulls against the now-stale delegation; it is refused");
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(10_000_000)
        .fixed_ix();
    world.send_err(&[ix], &[&bob], "TransferFixed (stale authority)", SubscriptionsError::StaleSubscriptionAuthority);
    let alice_balance = token_balance(world.svm(), &alice_ata);
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Alice's funds are untouched", 100_000_000, alice_balance);
    world.md().check("Bob's ATA is empty", 0, bob_balance);

    let ix = RevokeDelegation::new(world.svm_mut(), &alice, mint, bob.pubkey(), nonce).instruction();
    world.send_ok(&[ix], &[&alice], "RevokeDelegation");
}

#[test]
fn test_close_subscription_authority_blocks_all_transfers() {
    let mut world = World::new(make_backend(), 
        "Closing the authority blocks all delegations",
        "after the authority is closed, every outstanding delegation's pull is refused",
    );
    let amount: u64 = 50_000_000;

    let alice = world.actor("alice");
    let bob = world.actor("bob");
    let charlie = world.actor("charlie");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let alice_ata = world.fund_ata(mint, &alice, 100_000_000);
    let _bob_ata = world.fund_ata(mint, &bob, 0);
    let _charlie_ata = world.fund_ata(mint, &charlie, 0);

    world.md().step("Stage: Alice's authority and delegations to Bob and Charlie");
    world.init_authority(&alice, mint, None).0.assert_ok();

    let (ix, del_bob) =
        CreateDelegation::new(world.svm_mut(), &alice, mint, bob.pubkey()).nonce(0).fixed_ix(amount, expiry_ts);
    world.prop(del_bob, "Bob delegation");
    world.send_ok(&[ix], &[&alice], "CreateFixedDelegation (Bob)");

    let (ix, del_charlie) =
        CreateDelegation::new(world.svm_mut(), &alice, mint, charlie.pubkey()).nonce(0).fixed_ix(amount, expiry_ts);
    world.prop(del_charlie, "Charlie delegation");
    world.send_ok(&[ix], &[&alice], "CreateFixedDelegation (Charlie)");

    world.md().step("Alice closes her subscription authority");
    let ix = CloseSubscriptionAuthority::new(world.svm_mut(), &alice, mint).instruction();
    world.send_ok(&[ix], &[&alice], "CloseSubscriptionAuthority");

    world.md().step("Both Bob's and Charlie's pulls are now refused");
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, del_bob).amount(10_000_000).fixed_ix();
    world.send_err(
        &[ix],
        &[&bob],
        "TransferFixed (Bob, authority closed)",
        SubscriptionsError::InvalidSubscriptionAuthorityPda,
    );

    let ix =
        TransferDelegation::new(world.svm_mut(), &charlie, alice.pubkey(), mint, del_charlie).amount(10_000_000).fixed_ix();
    world.send_err(
        &[ix],
        &[&charlie],
        "TransferFixed (Charlie, authority closed)",
        SubscriptionsError::InvalidSubscriptionAuthorityPda,
    );

    let alice_balance = token_balance(world.svm(), &alice_ata);
    world.md().check("Alice's funds are untouched", 100_000_000, alice_balance);

    let ix = RevokeDelegation::new(world.svm_mut(), &alice, mint, bob.pubkey(), 0).instruction();
    world.send_ok(&[ix], &[&alice], "RevokeDelegation (Bob)");
    let ix = RevokeDelegation::new(world.svm_mut(), &alice, mint, charlie.pubkey(), 0).instruction();
    world.send_ok(&[ix], &[&alice], "RevokeDelegation (Charlie)");
}

#[test]
fn test_fixed_transfer_within_drift_window() {
    let mut world = World::new(make_backend(), 
        "Fixed transfer within the drift window succeeds",
        "a pull shortly after the nominal expiry, but within the clock-drift tolerance, succeeds",
    );
    let amount: u64 = 50_000_000;
    let expiry_ts: i64 = world.now() + 100;
    let nonce = 0;
    let transfer_amount = 10_000_000;

    let (alice, bob, delegation_pda, mint, _, _) = setup_fixed_delegation(&mut world, amount, expiry_ts, nonce);

    world.md().step("The clock advances just past expiry but within the drift window");
    world.warp(110);

    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(transfer_amount)
        .fixed_ix();
    world.send_ok(&[ix], &[&bob], "TransferFixed (within drift window)");
}

#[test]
fn test_fixed_transfer_past_drift_window() {
    let mut world = World::new(make_backend(), 
        "Fixed transfer past the drift window is refused",
        "a pull well past expiry, beyond the clock-drift tolerance, is refused",
    );
    let amount: u64 = 50_000_000;
    let expiry_ts: i64 = world.now() + 100;
    let nonce = 0;
    let transfer_amount = 10_000_000;

    let (alice, bob, delegation_pda, mint, _, _) = setup_fixed_delegation(&mut world, amount, expiry_ts, nonce);

    world.md().step("The clock advances well past expiry, beyond the drift window");
    world.warp(221);

    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(transfer_amount)
        .fixed_ix();
    world.send_err(&[ix], &[&bob], "TransferFixed (past drift window)", SubscriptionsError::DelegationExpired);
}
