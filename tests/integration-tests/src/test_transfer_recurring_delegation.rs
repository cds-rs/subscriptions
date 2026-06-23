//! `transfer_recurring_delegation`, converted to the World/scenario pattern.
//!
//! Each test builds its own `World`, draws actors from the cast (`alice` the
//! delegator, `bob` the delegatee, `charlie` a third party, `mallory` the
//! adversary), stages the recurring delegation through `setup_recurring_delegation`,
//! and pulls against it through the observed `send_*`. Every send renders its
//! surface into the test's report under `target/md-reports/`.

use litesvm_utils::{LiteSvmBackend, TestSVM};

use crate::{
    event_engine::event_authority_pda,
    instructions::transfer_recurring_delegation,
    state::{header::VERSION_OFFSET, RecurringDelegation},
    tests::{
        constants::{MINT_DECIMALS, PROGRAM_ID, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID},
        idl,
        pda::get_subscription_authority_pda,
        utils::{
            days, token_balance, hours, init_aux_token_account, init_mint, minutes,
            CloseSubscriptionAuthority, CreateDelegation, ObservedResultExt, TransferDelegation, make_backend, World,
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
use spl_token_interface::instruction::TokenInstruction::{Approve, Revoke};

/// Stage a recurring delegation: Alice initializes her authority over a fresh SPL
/// mint, funds her ATA, and creates a recurring delegation to Bob. Mirrors the
/// suite's `setup_recurring_delegation`, but every send is observed. Returns the
/// cast and the derived accounts (minus the LiteSVM, which the World owns).
#[allow(clippy::too_many_arguments)]
fn setup_recurring_delegation(
    world: &mut World<LiteSvmBackend>,
    amount_per_period: u64,
    period_length_s: u64,
    start_ts: i64,
    expiry_ts: i64,
    nonce: u64,
) -> (Keypair, Keypair, Pubkey, Pubkey, Pubkey, Pubkey, Pubkey) {
    let alice = world.actor("alice");
    let bob = world.actor("bob");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let alice_ata = world.fund_ata(mint, &alice, 100_000_000);
    let bob_ata = world.fund_ata(mint, &bob, 0);

    world.md().step("Stage: Alice's authority and a recurring delegation to Bob");
    let (init_result, init_pda, _) = world.init_authority(&alice, mint, None);
    init_result.assert_ok();

    let (ix, delegation_pda) = CreateDelegation::new(world.svm_mut(), &alice, mint, bob.pubkey())
        .nonce(nonce)
        .recurring_ix(amount_per_period, period_length_s, start_ts, expiry_ts);
    world.prop(delegation_pda, "RecurringDelegation");
    world.send_ok(&[ix], &[&alice], "CreateRecurringDelegation");

    (alice, bob, delegation_pda, mint, alice_ata, bob_ata, init_pda)
}

#[test]
fn test_recurring_transfer_success() {
    let mut world = World::new(make_backend(), 
        "Recurring transfer succeeds across pulls within a period",
        "Bob pulls repeatedly within one period; the amount pulled accumulates toward the period limit",
    );
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = hours(1);
    let start_ts: i64 = world.now();
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce = 0;

    let (alice, bob, delegation_pda, mint, _, bob_ata, _) =
        setup_recurring_delegation(&mut world, amount_per_period, period_length_s, start_ts, expiry_ts, nonce);

    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob's ATA starts empty", 0, bob_balance);

    world.md().step("Bob pulls 10 tokens in period 0");
    let transfer_amount: u64 = 10_000_000;
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(transfer_amount)
        .recurring_ix();
    world.send_ok(&[ix], &[&bob], "TransferRecurring (pull 1)");

    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob received 10 tokens", 10_000_000, bob_balance);

    let delegation_account = world.svm().get_account(&delegation_pda).unwrap();
    let delegation = RecurringDelegation::load(&delegation_account.data).unwrap();
    world.md().check("10 tokens pulled in the period", 10_000_000, delegation.amount_pulled_in_period);
    world.md().check("the period start is the delegation start", start_ts, delegation.current_period_start_ts);
    world.md().check("the period length is unchanged", period_length_s as i64, delegation.period_length_s as i64);

    world.md().step("The clock advances 15 minutes, still within period 0; Bob pulls again");
    world.warp(minutes(15));

    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(transfer_amount)
        .recurring_ix();
    world.send_ok(&[ix], &[&bob], "TransferRecurring (pull 2)");

    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob received another 10 tokens", 20_000_000, bob_balance);

    let delegation_account = world.svm().get_account(&delegation_pda).unwrap();
    let delegation = RecurringDelegation::load(&delegation_account.data).unwrap();
    world.md().check("20 tokens pulled in the period", 20_000_000, delegation.amount_pulled_in_period);

    world.md().step("Another 15 minutes pass, still within period 0; Bob pulls a third time");
    world.warp(minutes(15));

    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(transfer_amount)
        .recurring_ix();
    world.send_ok(&[ix], &[&bob], "TransferRecurring (pull 3)");

    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob received a third 10 tokens", 30_000_000, bob_balance);

    let delegation_account = world.svm().get_account(&delegation_pda).unwrap();
    let delegation = RecurringDelegation::load(&delegation_account.data).unwrap();
    world.md().check("30 tokens pulled in the period", 30_000_000, delegation.amount_pulled_in_period);
}

#[test]
fn test_recurring_transfer_exceeds_period_limit() {
    let mut world = World::new(make_backend(), 
        "Recurring transfer exceeding the period limit is refused",
        "a single pull for more than the per-period allowance is refused",
    );
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = hours(1);
    let start_ts: i64 = world.now();
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce = 1;

    let (alice, bob, delegation_pda, mint, _, bob_ata, _) =
        setup_recurring_delegation(&mut world, amount_per_period, period_length_s, start_ts, expiry_ts, nonce);

    world.md().step("The clock advances past one period; Bob tries to pull 60 against a 50 limit");
    world.warp(period_length_s + 1);

    let transfer_amount: u64 = 60_000_000;
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(transfer_amount)
        .recurring_ix();
    world.send_err(
        &[ix],
        &[&bob],
        "TransferRecurring (exceeds period limit)",
        SubscriptionsError::AmountExceedsPeriodLimit,
    );

    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob's ATA is still empty", 0, bob_balance);
}

#[test]
fn test_recurring_transfer_expired() {
    let mut world = World::new(make_backend(), 
        "Recurring transfer after expiry is refused",
        "a pull within the window succeeds; once the clock passes expiry, a further pull is refused",
    );
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = hours(1);
    let start_ts: i64 = world.now();
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce = 1;

    let (alice, bob, delegation_pda, mint, _, bob_ata, _) =
        setup_recurring_delegation(&mut world, amount_per_period, period_length_s, start_ts, expiry_ts, nonce);

    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob's ATA starts empty", 0, bob_balance);

    world.md().step("Bob pulls 30 tokens within the window");
    let transfer_amount: u64 = 30_000_000;
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(transfer_amount)
        .recurring_ix();
    world.send_ok(&[ix], &[&bob], "TransferRecurring (within window)");
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob received 30 tokens", 30_000_000, bob_balance);

    world.md().step("The clock advances past expiry; a further pull is refused");
    let warp_secs = (world.now() + (days(2) as i64)) as u64;
    world.warp(warp_secs);

    let transfer_amount: u64 = 30_000_000;
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(transfer_amount)
        .recurring_ix();
    world.send_err(&[ix], &[&bob], "TransferRecurring (expired)", SubscriptionsError::DelegationExpired);
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob's balance is unchanged", 30_000_000, bob_balance);
}

#[test]
fn test_recurring_transfer_multiple_periods() {
    let mut world = World::new(make_backend(), 
        "Recurring transfer resets the allowance each period",
        "after a full period elapses, the amount pulled resets and Bob can pull again",
    );
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = hours(1);
    let start_ts: i64 = world.now();
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce = 1;

    let (alice, bob, delegation_pda, mint, _, bob_ata, _) =
        setup_recurring_delegation(&mut world, amount_per_period, period_length_s, start_ts, expiry_ts, nonce);

    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob's ATA starts empty", 0, bob_balance);

    world.md().step("Bob pulls 30 tokens in period 0");
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(30_000_000)
        .recurring_ix();
    world.send_ok(&[ix], &[&bob], "TransferRecurring (period 0)");

    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob received 30 tokens", 30_000_000, bob_balance);

    let delegation_account = world.svm().get_account(&delegation_pda).unwrap();
    let delegation_amount_pulled_in_period =
        RecurringDelegation::load(&delegation_account.data).unwrap().amount_pulled_in_period;
    world.md().check("30 tokens pulled in period 0", 30_000_000, delegation_amount_pulled_in_period);

    world.md().step("A full period elapses; the allowance resets and Bob pulls 30 again");
    world.warp(period_length_s);

    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(30_000_000)
        .recurring_ix();
    world.send_ok(&[ix], &[&bob], "TransferRecurring (period 1)");

    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob received another 30 tokens", 60_000_000, bob_balance);

    let delegation_account = world.svm().get_account(&delegation_pda).unwrap();
    let delegation_amount_pulled_in_period =
        RecurringDelegation::load(&delegation_account.data).unwrap().amount_pulled_in_period;
    world.md().check("the pulled amount reset to 30 in the new period", 30_000_000, delegation_amount_pulled_in_period);
}

#[test]
fn test_recurring_transfer_skip_multiple_periods() {
    let mut world = World::new(make_backend(), 
        "Recurring transfer aligns the period start when periods are skipped",
        "after skipping three periods, the period start jumps forward by a whole number of periods",
    );
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = hours(1);
    let start_ts: i64 = world.now();
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce = 2;

    let (alice, bob, delegation_pda, mint, _, bob_ata, _) =
        setup_recurring_delegation(&mut world, amount_per_period, period_length_s, start_ts, expiry_ts, nonce);

    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob's ATA starts empty", 0, bob_balance);

    world.md().step("Period 0: Bob pulls 10 tokens");
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(10_000_000)
        .recurring_ix();
    world.send_ok(&[ix], &[&bob], "TransferRecurring (period 0)");
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob received 10 tokens", 10_000_000, bob_balance);

    world.md().step("Three periods are skipped; Bob pulls again in period 3");
    world.warp(period_length_s * 3);

    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(10_000_000)
        .recurring_ix();
    world.send_ok(&[ix], &[&bob], "TransferRecurring (period 3)");

    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob received another 10 tokens", 20_000_000, bob_balance);

    let delegation_account = world.svm().get_account(&delegation_pda).unwrap();
    let delegation = RecurringDelegation::load(&delegation_account.data).unwrap();

    // New start should be start_ts + 3 * period
    let expected_start = start_ts + (period_length_s * 3) as i64;
    world.md().check("the period start jumped forward 3 periods", expected_start, delegation.current_period_start_ts);
    world.md().check("only 10 tokens pulled in the new period", 10_000_000, delegation.amount_pulled_in_period);
}

#[test]
fn test_recurring_transfer_skip_period_cannot_double_claim() {
    // Bug hypothesis: after skipping one period with no claims, the delegatee
    // can claim twice (2x amount_per_period) in the next period.
    //
    // Scenario:
    //   Period 0: claim full allowance
    //   Period 1: no claims (skipped)
    //   Period 2 start: claim full allowance, then immediately try again
    //
    // Expected: second claim in period 2 should fail — skipped periods
    // do not accumulate allowance.
    let mut world = World::new(make_backend(), 
        "Skipped periods do not accumulate allowance",
        "after skipping a period, the delegatee cannot claim twice the per-period allowance in the next",
    );
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = hours(1);
    let start_ts: i64 = world.now();
    let expiry_ts: i64 = world.now() + days(7) as i64;
    let nonce = 3;

    let (alice, bob, delegation_pda, mint, _, bob_ata, _) =
        setup_recurring_delegation(&mut world, amount_per_period, period_length_s, start_ts, expiry_ts, nonce);

    world.md().step("Period 0: Bob claims the full allowance");
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(amount_per_period)
        .recurring_ix();
    world.send_ok(&[ix], &[&bob], "TransferRecurring (period 0, full)");
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob received the full 50 tokens", 50_000_000, bob_balance);

    world.md().step("Period 1 is skipped entirely; advance to the start of period 2");
    world.warp(period_length_s * 2);

    world.md().step("Period 2: Bob claims the full allowance again");
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(amount_per_period)
        .recurring_ix();
    world.send_ok(&[ix], &[&bob], "TransferRecurring (period 2, full)");
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob received another full 50 tokens", 100_000_000, bob_balance);

    world.md().step("Period 2: Bob immediately tries to claim again; it is refused");
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(amount_per_period)
        .recurring_ix();
    world.send_err(
        &[ix],
        &[&bob],
        "TransferRecurring (period 2, second claim)",
        SubscriptionsError::AmountExceedsPeriodLimit,
    );

    // Balances unchanged after failed transfer
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob's balance is unchanged after the refused claim", 100_000_000, bob_balance);

    // Verify delegation state
    let delegation_account = world.svm().get_account(&delegation_pda).unwrap();
    let delegation = RecurringDelegation::load(&delegation_account.data).unwrap();
    world.md().check("the full allowance is marked pulled", amount_per_period, delegation.amount_pulled_in_period);
    let expected_start = start_ts + (period_length_s * 2) as i64;
    world.md().check("the period start is aligned to period 2", expected_start, delegation.current_period_start_ts);
}

#[test]
fn recurring_delegation_rejects_transfer_with_different_mint_authority() {
    let mut world = World::new(make_backend(), 
        "Recurring delegation rejects a different-mint transfer",
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

    let amount_per_period = 50_000_000;
    let period_length_s = hours(1);
    let start_ts = world.now();
    let expiry_ts = start_ts + days(1) as i64;
    let (ix, low_value_delegation_pda) = CreateDelegation::new(world.svm_mut(), &alice, low_value_mint, bob.pubkey())
        .nonce(7)
        .recurring_ix(amount_per_period, period_length_s, start_ts, expiry_ts);
    world.prop(low_value_delegation_pda, "low-value delegation");
    world.send_ok(&[ix], &[&alice], "CreateRecurringDelegation (low-value)");

    world.md().step("Bob replays the low-value delegation against the high-value mint; it is refused");
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), high_value_mint, low_value_delegation_pda)
        .amount(10_000_000)
        .recurring_ix();
    world.send_err(&[ix], &[&bob], "TransferRecurring (mismatched mint)", SubscriptionsError::InvalidDelegatePda);

    let alice_balance = token_balance(world.svm(), &alice_high_ata);
    let bob_balance = token_balance(world.svm(), &bob_high_ata);
    world.md().check("Alice's high-value ATA is untouched", 100_000_000, alice_balance);
    world.md().check("Bob's high-value ATA is empty", 0, bob_balance);

    let delegation_account = world.svm().get_account(&low_value_delegation_pda).unwrap();
    let amount_pulled = RecurringDelegation::load(&delegation_account.data).unwrap().amount_pulled_in_period;
    world.md().check("the low-value allowance is intact", 0, amount_pulled);
}

#[test]
fn recurring_transfer_rejects_approved_non_canonical_source() {
    let mut world = World::new(make_backend(), 
        "Recurring transfer rejects a non-canonical source",
        "an auxiliary (non-ATA) source the authority was Approve'd over cannot be drained via a delegation",
    );
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = hours(1);
    let start_ts: i64 = world.now();
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce = 0;

    let (alice, bob, delegation_pda, mint, alice_ata, bob_ata, subscription_authority_pda) =
        setup_recurring_delegation(&mut world, amount_per_period, period_length_s, start_ts, expiry_ts, nonce);

    let alice_aux = init_aux_token_account(world.svm_mut(), mint, alice.pubkey(), 100_000_000);
    world.prop(alice_aux, "Alice aux token account");

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
        .recurring_ix();
    world.send_err(
        &[ix],
        &[&bob],
        "TransferRecurring (non-canonical source)",
        SubscriptionsError::InvalidAssociatedTokenAccountDerivedAddress,
    );

    let alice_balance = token_balance(world.svm(), &alice_ata);
    let alice_aux_balance = token_balance(world.svm(), &alice_aux);
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Alice's ATA is untouched", 100_000_000, alice_balance);
    world.md().check("Alice's aux account is untouched", 100_000_000, alice_aux_balance);
    world.md().check("Bob's ATA is empty", 0, bob_balance);

    let delegation_account = world.svm().get_account(&delegation_pda).unwrap();
    let amount_pulled = RecurringDelegation::load(&delegation_account.data).unwrap().amount_pulled_in_period;
    world.md().check("the allowance is intact", 0, amount_pulled);
}

#[test]
fn writable_accounts_must_be_writable() {
    let writable = idl::writable_account_indices("transferRecurring");

    let mut world = World::new(make_backend(), 
        "Writable accounts must be writable (transferRecurring)",
        "flipping any account the transfer writes to read-only is rejected",
    );
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = hours(1);
    let start_ts: i64 = world.now();
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce = 0;

    let (alice, bob, delegation_pda, mint, _, _, _) =
        setup_recurring_delegation(&mut world, amount_per_period, period_length_s, start_ts, expiry_ts, nonce);
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

        // Flip writable account to readonly, preserving signer flag
        let pubkey = accounts[*idx].pubkey;
        accounts[*idx] = AccountMeta::new_readonly(pubkey, *is_signer);

        let transfer_amount: u64 = 10_000_000;
        let data = [
            vec![*transfer_recurring_delegation::DISCRIMINATOR],
            transfer_amount.to_le_bytes().to_vec(),
            alice.pubkey().to_bytes().to_vec(),
            mint.to_bytes().to_vec(),
        ]
        .concat();

        let ix = Instruction { program_id: PROGRAM_ID, accounts, data };

        world.send_err(
            &[ix],
            &[&sponsor, &bob],
            &format!("TransferRecurring ({name} forced read-only)"),
            SubscriptionsError::AccountNotWritable,
        );
    }
}

#[test]
fn signer_accounts_must_be_signers() {
    let signers = idl::signer_account_indices("transferRecurring");

    let mut world = World::new(make_backend(), 
        "Signer accounts must sign (transferRecurring)",
        "flipping any required signer to non-signer is rejected",
    );
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = hours(1);
    let start_ts: i64 = world.now();
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce = 0;

    let (alice, bob, delegation_pda, mint, _, _, _) =
        setup_recurring_delegation(&mut world, amount_per_period, period_length_s, start_ts, expiry_ts, nonce);
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

        // Flip signer to non-signer, preserving writable flag
        let pubkey = accounts[*idx].pubkey;
        accounts[*idx] =
            if *is_writable { AccountMeta::new(pubkey, false) } else { AccountMeta::new_readonly(pubkey, false) };

        let transfer_amount: u64 = 10_000_000;
        let data = [
            vec![*transfer_recurring_delegation::DISCRIMINATOR],
            transfer_amount.to_le_bytes().to_vec(),
            alice.pubkey().to_bytes().to_vec(),
            mint.to_bytes().to_vec(),
        ]
        .concat();

        let ix = Instruction { program_id: PROGRAM_ID, accounts, data };

        world.send_err(
            &[ix],
            &[&sponsor],
            &format!("TransferRecurring ({name} forced non-signer)"),
            SubscriptionsError::NotSigner,
        );
    }
}

#[test]
fn test_recurring_transfer_delegator_mismatch_exploit() {
    // This test demonstrates the access control vulnerability where a malicious delegatee
    // can use their own delegation to transfer funds from another user's account
    let mut world = World::new(make_backend(), 
        "Recurring transfer delegator-mismatch exploit is blocked",
        "Bob's self-delegation cannot be used to drain Alice's account by spoofing the delegator",
    );
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = hours(1);
    let start_ts: i64 = world.now();
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce = 0;

    // Setup: Alice (victim) with funds and Bob (the malicious delegatee)
    let (alice, bob, _alice_delegation_pda, mint, alice_ata, bob_ata, _) =
        setup_recurring_delegation(&mut world, amount_per_period, period_length_s, start_ts, expiry_ts, nonce);

    world.md().step("Bob initializes his own authority and a self-delegation");
    world.init_authority(&bob, mint, None).0.assert_ok();

    // Bob creates a self-delegation (Bob -> Bob) with a large allowance
    let (ix, bob_delegation_pda) = CreateDelegation::new(world.svm_mut(), &bob, mint, bob.pubkey())
        .nonce(nonce)
        .recurring_ix(1_000_000_000, period_length_s, start_ts, expiry_ts);
    world.prop(bob_delegation_pda, "Bob self-delegation");
    world.send_ok(&[ix], &[&bob], "CreateRecurringDelegation (Bob self)");

    world.md().step("Bob spoofs Alice as the delegator while using his own delegation; it is refused");
    let transfer_amount: u64 = 30_000_000;

    // Exploit: Bob tries to transfer from Alice's ATA using his own delegation
    // by passing Alice's delegator_pubkey in the instruction data
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, bob_delegation_pda)
        .amount(transfer_amount)
        .to(bob_ata)
        .recurring_ix();

    // After the fix, this should fail with Unauthorized error
    world.send_err(&[ix], &[&bob], "TransferRecurring (delegator mismatch)", SubscriptionsError::Unauthorized);

    let alice_balance = token_balance(world.svm(), &alice_ata);
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Alice's funds are untouched", 100_000_000, alice_balance);
    world.md().check("Bob received no funds", 0, bob_balance);
}

#[test]
fn test_recurring_transfer_token_revoke() {
    let mut world = World::new(make_backend(), 
        "Recurring transfer after a token Revoke is refused until re-approved for the max",
        "revoking the SPL approval breaks the pull; a partial re-approval still fails; only a max approval restores it",
    );
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = hours(1);
    let start_ts: i64 = world.now();
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce = 0;

    let (alice, bob, delegation_pda, mint, alice_ata, bob_ata, subscription_authority_pda) =
        setup_recurring_delegation(&mut world, amount_per_period, period_length_s, start_ts, expiry_ts, nonce);

    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob's ATA starts empty", 0, bob_balance);

    world.md().step("Bob pulls the full 50 tokens in period 0");
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(50_000_000)
        .recurring_ix();
    world.send_ok(&[ix], &[&bob], "TransferRecurring (period 0)");
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob received 50 tokens", 50_000_000, bob_balance);

    world.md().step("Alice revokes the SPL token approval over her ATA");
    let ix = Instruction {
        program_id: TOKEN_PROGRAM_ID,
        accounts: vec![AccountMeta::new(alice_ata, false), AccountMeta::new(alice.pubkey(), true)],
        data: Revoke.pack(),
    };
    world.send_ok(&[ix], &[&alice], "Revoke");

    // Now let's move the clock and try to fetch recurring delegation again
    world.md().step("A full period elapses; Bob's pull now fails with an owner mismatch");
    world.warp(period_length_s);

    // Now, let's try again
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(50_000_000)
        .recurring_ix();
    let result = world.send(&[ix], &[&bob], "TransferRecurring (after revoke)");
    assert!(!result.is_success());
    result.assert_error_code(spl_token_interface::error::TokenError::OwnerMismatch as u32);

    // Doing approval once again fixes it, but it has to be max possible for it to work

    // Scenario 1: We approve, but less amount
    world.md().step("Alice re-approves, but for too small an amount; the pull still fails");
    let ix = Instruction {
        program_id: TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(alice_ata, false),
            AccountMeta::new(subscription_authority_pda, false),
            AccountMeta::new(alice.pubkey(), true),
        ],
        data: Approve { amount: 100000 }.pack(),
    };
    world.send_ok(&[ix], &[&alice], "Approve (partial)");

    // Since the approval amount is less than what is needed, we fail again
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(50_000_000)
        .recurring_ix();
    let result = world.send(&[ix], &[&bob], "TransferRecurring (partial approval)");
    assert!(!result.is_success());
    result.assert_error_code(spl_token_interface::error::TokenError::InsufficientFunds as u32);

    // Scenario 2: We approve for max amount. Now it should work as usual
    world.md().step("Alice re-approves for the max amount; the pull succeeds again");
    let ix = Instruction {
        program_id: TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(alice_ata, false),
            AccountMeta::new(subscription_authority_pda, false),
            AccountMeta::new(alice.pubkey(), true),
        ],
        data: Approve { amount: u64::MAX }.pack(),
    };
    world.send_ok(&[ix], &[&alice], "Approve (max)");

    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(50_000_000)
        .recurring_ix();
    world.send_ok(&[ix], &[&bob], "TransferRecurring (after max approval)");
}

#[test]
fn test_recurring_transfer_to_third_party() {
    let mut world = World::new(make_backend(), 
        "Recurring transfer to a third party",
        "Bob, the delegatee, routes a pull to Charlie's account",
    );
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = hours(1);
    let start_ts: i64 = world.now();
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce = 0;

    // Alice delegates to Bob
    let (alice, bob, delegation_pda, mint, _, _, _) =
        setup_recurring_delegation(&mut world, amount_per_period, period_length_s, start_ts, expiry_ts, nonce);

    // Charlie is a third party
    let charlie = world.actor("charlie");
    let charlie_ata = world.fund_ata(mint, &charlie, 0);

    world.md().step("Bob pulls from Alice and routes the funds to Charlie");
    let transfer_amount: u64 = 10_000_000;

    // Bob transfers from Alice -> Charlie
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(transfer_amount)
        .to(charlie_ata)
        .recurring_ix();
    world.send_ok(&[ix], &[&bob], "TransferRecurring (to third party)");

    // Verify Charlie received funds
    let charlie_balance = token_balance(world.svm(), &charlie_ata);
    world.md().check("Charlie received 10 tokens", 10_000_000, charlie_balance);
}

#[test]
fn test_recurring_transfer_version_mismatch() {
    let mut world = World::new(make_backend(), 
        "Recurring transfer over a stale-version delegation is refused",
        "a delegation whose header version byte was zeroed requires explicit migration",
    );
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = hours(1);
    let start_ts: i64 = world.now();
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce = 0;

    let (alice, bob, delegation_pda, mint, _, bob_ata, _) =
        setup_recurring_delegation(&mut world, amount_per_period, period_length_s, start_ts, expiry_ts, nonce);

    world.md().step("The delegation's header version byte is corrupted to 0");
    let mut account = world.svm().get_account(&delegation_pda).unwrap();
    account.data[VERSION_OFFSET] = 0;
    world.svm_mut().set_account(&delegation_pda, account);

    world.md().step("Bob pulls against the stale delegation; migration is required");
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(10_000_000)
        .recurring_ix();
    world.send_err(&[ix], &[&bob], "TransferRecurring (version mismatch)", SubscriptionsError::MigrationRequired);
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob's ATA is empty", 0, bob_balance);
}

#[test]
fn test_recurring_transfer_stale_subscription_authority() {
    let mut world = World::new(make_backend(), 
        "Recurring transfer against a re-initialized authority is refused",
        "closing then re-initializing the authority bumps its init_id, staling the delegation",
    );
    let amount_per_period: u64 = 50_000_000;
    let period_length_s = days(1);
    let start_ts = world.now();
    let expiry_ts = world.now() + days(30) as i64;
    let nonce = 0;

    let (alice, bob, delegation_pda, mint, alice_ata, bob_ata, _) =
        setup_recurring_delegation(&mut world, amount_per_period, period_length_s, start_ts, expiry_ts, nonce);

    world.md().step("Alice closes and re-initializes her authority, bumping its init_id");
    let ix = CloseSubscriptionAuthority::new(world.svm_mut(), &alice, mint).instruction();
    world.send_ok(&[ix], &[&alice], "CloseSubscriptionAuthority");

    world.warp(2);

    world.init_authority(&alice, mint, None).0.assert_ok();

    world.md().step("Bob pulls against the now-stale delegation; it is refused");
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(10_000_000)
        .recurring_ix();
    world.send_err(&[ix], &[&bob], "TransferRecurring (stale authority)", SubscriptionsError::StaleSubscriptionAuthority);
    let alice_balance = token_balance(world.svm(), &alice_ata);
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Alice's funds are untouched", 100_000_000, alice_balance);
    world.md().check("Bob's ATA is empty", 0, bob_balance);
}

#[test]
fn test_recurring_transfer_not_started() {
    let mut world = World::new(make_backend(), 
        "Recurring transfer before the start time is refused",
        "a pull before the delegation's start is refused; once the start passes, it succeeds",
    );
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = hours(1);
    let start_ts: i64 = world.now() + hours(1) as i64;
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce = 0;

    let (alice, bob, delegation_pda, mint, _, bob_ata, _) =
        setup_recurring_delegation(&mut world, amount_per_period, period_length_s, start_ts, expiry_ts, nonce);

    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob's ATA starts empty", 0, bob_balance);

    world.md().step("Bob pulls before the start time; it is refused");
    let transfer_amount: u64 = 10_000_000;
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(transfer_amount)
        .recurring_ix();
    world.send_err(&[ix], &[&bob], "TransferRecurring (not started)", SubscriptionsError::DelegationNotStarted);
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob's ATA is still empty", 0, bob_balance);

    world.md().step("The clock passes the start time; Bob's pull now succeeds");
    world.warp(hours(1) + 1);

    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(transfer_amount)
        .recurring_ix();
    world.send_ok(&[ix], &[&bob], "TransferRecurring (after start)");
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob received 10 tokens", transfer_amount, bob_balance);
}

#[test]
fn test_recurring_transfer_within_drift_window() {
    let mut world = World::new(make_backend(), 
        "Recurring transfer within the drift window succeeds",
        "a pull shortly after the nominal expiry, but within the clock-drift tolerance, succeeds",
    );
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = hours(1);
    let start_ts: i64 = world.now();
    let expiry_ts: i64 = world.now() + hours(1) as i64;
    let nonce = 0;
    let transfer_amount = 10_000_000;

    let (alice, bob, delegation_pda, mint, _, _, _) =
        setup_recurring_delegation(&mut world, amount_per_period, period_length_s, start_ts, expiry_ts, nonce);

    world.md().step("The clock advances just past expiry but within the drift window");
    world.warp(hours(1) + 60);

    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(transfer_amount)
        .recurring_ix();
    world.send_ok(&[ix], &[&bob], "TransferRecurring (within drift window)");
}

#[test]
fn test_recurring_rollover_blocked_at_expiry_boundary() {
    let mut world = World::new(make_backend(), 
        "Recurring rollover blocked at the expiry boundary",
        "when a period would roll over exactly at expiry, the rollover pull is refused",
    );
    let amount_per_period: u64 = 1_000_000;
    let period_length_s: u64 = 1;
    let start_ts: i64 = world.now();
    let expiry_ts: i64 = start_ts + period_length_s as i64;
    let nonce = 0;

    let (alice, bob, delegation_pda, mint, _, bob_ata, _) =
        setup_recurring_delegation(&mut world, amount_per_period, period_length_s, start_ts, expiry_ts, nonce);

    world.md().step("Bob pulls the full allowance in period 0");
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(amount_per_period)
        .recurring_ix();
    world.send_ok(&[ix], &[&bob], "TransferRecurring (period 0)");
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob received the full allowance", amount_per_period, bob_balance);

    world.md().step("A period elapses, landing exactly at expiry; the rollover pull is refused");
    world.warp(period_length_s);

    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(amount_per_period)
        .recurring_ix();
    world.send_err(
        &[ix],
        &[&bob],
        "TransferRecurring (rollover at expiry)",
        SubscriptionsError::AmountExceedsPeriodLimit,
    );
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Bob's balance is unchanged", amount_per_period, bob_balance);
}

#[test]
fn test_recurring_transfer_past_drift_window() {
    let mut world = World::new(make_backend(), 
        "Recurring transfer past the drift window is refused",
        "a pull well past expiry, beyond the clock-drift tolerance, is refused",
    );
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = hours(1);
    let start_ts: i64 = world.now();
    let expiry_ts: i64 = world.now() + hours(1) as i64;
    let nonce = 0;
    let transfer_amount = 10_000_000;

    let (alice, bob, delegation_pda, mint, _, _, _) =
        setup_recurring_delegation(&mut world, amount_per_period, period_length_s, start_ts, expiry_ts, nonce);

    world.md().step("The clock advances well past expiry, beyond the drift window");
    world.warp(hours(1) + 121);

    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(transfer_amount)
        .recurring_ix();
    world.send_err(&[ix], &[&bob], "TransferRecurring (past drift window)", SubscriptionsError::DelegationExpired);
}

#[test]
fn test_recurring_transfer_token_2022_transfer_fee() {
    let mut world = World::new(make_backend(), 
        "Recurring transfer over a Token-2022 transfer-fee mint",
        "the transfer fee is withheld from the receiver; the amount pulled tracks the gross amount",
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

    world.md().step("Stage: Alice's authority and a recurring delegation to Bob");
    world.init_authority(&alice, mint, None).0.assert_ok();

    let start_ts = world.now();
    let expiry_ts = world.now() + days(1) as i64;
    let (ix, delegation_pda) = CreateDelegation::new(world.svm_mut(), &alice, mint, bob.pubkey()).recurring_ix(
        50_000_000,
        hours(1),
        start_ts,
        expiry_ts,
    );
    world.prop(delegation_pda, "RecurringDelegation");
    world.send_ok(&[ix], &[&alice], "CreateRecurringDelegation");

    world.md().step("Bob pulls 10 tokens; the transfer fee is withheld");
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(10_000_000)
        .recurring_ix();
    world.send_ok(&[ix], &[&bob], "TransferRecurring");

    let alice_balance = token_balance(world.svm(), &alice_ata);
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Alice's ATA debited the gross 10 tokens", 90_000_000, alice_balance);
    world.md().check("Bob received 10 tokens net of the 1% fee", 9_900_000, bob_balance);

    let delegation_account = world.svm().get_account(&delegation_pda).unwrap();
    let delegation = RecurringDelegation::load(&delegation_account.data).unwrap();
    world.md().check("the amount pulled tracks the gross 10 tokens", 10_000_000, delegation.amount_pulled_in_period);
}

#[test]
fn test_recurring_transfer_token_2022_confidential_transfer_public_balance() {
    let mut world = World::new(make_backend(), 
        "Recurring transfer over a Token-2022 confidential-transfer mint",
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

    world.md().step("Stage: Alice's authority and a recurring delegation to Bob");
    world.init_authority(&alice, mint, None).0.assert_ok();

    let start_ts = world.now();
    let (ix, delegation_pda) = CreateDelegation::new(world.svm_mut(), &alice, mint, bob.pubkey()).recurring_ix(
        50_000_000,
        hours(1),
        start_ts,
        start_ts + days(1) as i64,
    );
    world.prop(delegation_pda, "RecurringDelegation");
    world.send_ok(&[ix], &[&alice], "CreateRecurringDelegation");

    world.md().step("Bob pulls 10 tokens of the public balance");
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(10_000_000)
        .recurring_ix();
    world.send_ok(&[ix], &[&bob], "TransferRecurring");

    let alice_balance = token_balance(world.svm(), &alice_ata);
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Alice's ATA debited 10 tokens", 90_000_000, alice_balance);
    world.md().check("Bob received 10 tokens", 10_000_000, bob_balance);
}

#[test]
fn test_recurring_transfer_token_2022_unconfigured_transfer_hook() {
    let mut world = World::new(make_backend(), 
        "Recurring transfer over a Token-2022 unconfigured transfer-hook mint",
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

    world.md().step("Stage: Alice's authority and a recurring delegation to Bob");
    world.init_authority(&alice, mint, None).0.assert_ok();

    let start_ts = world.now();
    let (ix, delegation_pda) = CreateDelegation::new(world.svm_mut(), &alice, mint, bob.pubkey()).recurring_ix(
        50_000_000,
        hours(1),
        start_ts,
        start_ts + days(1) as i64,
    );
    world.prop(delegation_pda, "RecurringDelegation");
    world.send_ok(&[ix], &[&alice], "CreateRecurringDelegation");

    world.md().step("Bob pulls 10 tokens; the unconfigured hook is a no-op");
    let ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(10_000_000)
        .recurring_ix();
    world.send_ok(&[ix], &[&bob], "TransferRecurring");

    let alice_balance = token_balance(world.svm(), &alice_ata);
    let bob_balance = token_balance(world.svm(), &bob_ata);
    world.md().check("Alice's ATA debited 10 tokens", 90_000_000, alice_balance);
    world.md().check("Bob received 10 tokens", 10_000_000, bob_balance);
}
