//! `create_fixed_delegation`, converted to the World/scenario pattern.
//!
//! Each test builds a `World`, draws its actors from the cast (`alice` the
//! delegator, `sponsor` the payer, `bob` the delegatee where it must sign),
//! stands up Alice's authority through the `init_authority` verb, and performs
//! the create-delegation action through the observed `send_*`. Every send renders
//! its surface into the test's report under `target/md-reports/`.
//!
//! In `create_fixed_delegation_with_sponsor` Alice revokes a sponsor-funded
//! delegation, routing the rent to the sponsor; her only balance movement is the
//! transaction fee, so the "Alice paid the revoke fee" check (a strict
//! `final < after`) is gated on `world.capabilities().fees`. On a fee-less engine
//! Alice's balance is unchanged after the revoke, which is correct, so the strict
//! inequality would not hold there. The companion "sponsor was refunded the rent"
//! check (a tolerance inequality) holds regardless and stays ungated.

use solana_pubkey::Pubkey;
use solana_signer::Signer;

use testsvm::TestSVM;

use crate::{
    tests::{
        pda::get_delegation_pda,
        utils::{as_pubkey,
            days, token_balance, CloseSubscriptionAuthority, CreateDelegation,
            ObservedResultExt, RevokeDelegation, TransferDelegation, World,
        },
    },
    AccountDiscriminator, FixedDelegation, SubscriptionsError,
};

pub fn create_fixed_delegation_with_sponsor<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Create a fixed delegation with a sponsor",
        "a sponsor pays the delegation rent; revoking refunds it to the sponsor",
    );
    let delegator = world.actor("alice");
    let sponsor = world.actor("sponsor");

    let amount: u64 = 100_000_000;
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce: u64 = 0;

    let mint =
        world.usdc_mint(&delegator);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &delegator, 1_000_000);

    world.init_authority(&delegator, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");

    let delegator_balance_before = world.svm().get_account(&delegator.pubkey()).unwrap().lamports;
    let sponsor_balance_before = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;

    world.md().step("Alice creates a fixed delegation; the sponsor pays the rent");
    let (ix, delegation_pda) = {
        CreateDelegation::new(world.svm_mut(), &delegator, mint, delegatee)
            .payer(&sponsor)
            .nonce(nonce)
            .fixed_ix(amount, expiry_ts)
    };
    world.prop(delegation_pda, "FixedDelegation");
    world.send_ok(&[ix], &[&sponsor, &delegator], "CreateFixedDelegation (sponsored)");

    let delegator_balance_after = world.svm().get_account(&delegator.pubkey()).unwrap().lamports;
    let sponsor_balance_after = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;

    // Delegator shouldn't spend anything.
    world.md().check("Alice's lamports are untouched", delegator_balance_before, delegator_balance_after);
    world.md().check("the sponsor was charged", true, sponsor_balance_after < sponsor_balance_before);

    let account = world.svm().get_account(&delegation_pda).unwrap();
    let delegation_rent = account.lamports;
    let delegation = FixedDelegation::load(&account.data).unwrap();

    world.md().check("the delegation's payer is the sponsor", sponsor.pubkey(), as_pubkey(delegation.header.payer.to_bytes()));

    // Now revoke and check refund.
    world.md().step("Alice revokes the delegation, refunding the rent to the sponsor");
    let revoke_ix =
        RevokeDelegation::new(world.svm_mut(), &delegator, mint, delegatee, nonce).receiver(sponsor.pubkey()).instruction();
    world.send_ok(&[revoke_ix], &[&delegator], "RevokeDelegation (refund to sponsor)");

    let sponsor_balance_final = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;
    world.md().check(
        "the sponsor was refunded the delegation rent",
        true,
        sponsor_balance_final >= sponsor_balance_after + delegation_rent,
    );

    // Check delegator paid for revoke. The rent routes to the sponsor, so Alice's
    // only balance movement here is the transaction fee; on a fee-less engine her
    // balance is unchanged, so gate the strict drop on the fees capability.
    if world.capabilities().fees {
        let delegator_balance_final = world.svm().get_account(&delegator.pubkey()).unwrap().lamports;
        world.md().check("Alice paid the revoke fee", true, delegator_balance_final < delegator_balance_after);
    }
}

pub fn create_fixed_delegation<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Create a fixed delegation",
        "Alice grants a fixed-amount delegation to a delegatee",
    );
    let payer = world.actor("alice");
    let amount: u64 = 100_000_000;
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce: u64 = 0;

    let mint = world.usdc_mint(&payer);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &payer, 1_000_000);

    world.init_authority(&payer, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");

    world.md().step("Alice creates the fixed delegation");
    let (ix, delegation_pda) = {
        CreateDelegation::new(world.svm_mut(), &payer, mint, delegatee).nonce(nonce).fixed_ix(amount, expiry_ts)
    };
    world.prop(delegation_pda, "FixedDelegation");
    world.send_ok(&[ix], &[&payer], "CreateFixedDelegation");

    let account = world.svm().get_account(&delegation_pda).unwrap();
    let delegation = FixedDelegation::load(&account.data).unwrap();

    let header = delegation.header;
    let del_amount = delegation.amount;
    let del_expiry_s = delegation.expiry_ts;
    world.md().check("the delegator is Alice", payer.pubkey(), as_pubkey(header.delegator.to_bytes()));
    world.md().check("the delegatee matches", delegatee, as_pubkey(header.delegatee.to_bytes()));
    world.md().check(
        "the account is tagged FixedDelegation",
        AccountDiscriminator::FixedDelegation as u8,
        header.discriminator,
    );
    world.md().check("the delegated amount matches", amount, del_amount);
    world.md().check("the expiry matches", expiry_ts, del_expiry_s);
}

pub fn create_fixed_delegation_rejects_stale_subscription_authority_generation<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Reject a stale subscription-authority generation",
        "a delegation pinned to a closed authority's init_id is rejected after re-init",
    );
    let payer = world.actor("alice");
    let amount: u64 = 100_000_000;
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce: u64 = 0;

    let mint = world.usdc_mint(&payer);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &payer, 1_000_000);

    let (_, subscription_authority_pda, _) = world.init_authority(&payer, mint, None);
    let old_init_id =
        crate::state::SubscriptionAuthority::load(&world.svm().get_account(&subscription_authority_pda).unwrap().data)
            .unwrap()
            .init_id;

    world.md().step("Alice closes and re-initializes her authority, bumping its init_id");
    let close_ix = CloseSubscriptionAuthority::new(world.svm_mut(), &payer, mint).instruction();
    world.send_ok(&[close_ix], &[&payer], "CloseSubscriptionAuthority");
    world.warp(1);
    world.init_authority(&payer, mint, None).0.assert_ok();

    let new_init_id =
        crate::state::SubscriptionAuthority::load(&world.svm().get_account(&subscription_authority_pda).unwrap().data)
            .unwrap()
            .init_id;
    assert_ne!(old_init_id, new_init_id);

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");

    world.md().step("Alice pins the delegation to the old init_id; the program rejects it");
    let (ix, _) = {
        CreateDelegation::new(world.svm_mut(), &payer, mint, delegatee)
            .expected_subscription_authority_init_id(old_init_id)
            .nonce(nonce)
            .fixed_ix(amount, expiry_ts)
    };
    world.send_err(&[ix], &[&payer], "CreateFixedDelegation (stale init_id)", SubscriptionsError::StaleSubscriptionAuthority);
}

/// Verify that pre-funding a delegation PDA with lamports (DOS attack)
/// does not prevent the legitimate user from creating the delegation.
pub fn create_fixed_delegation_with_prefunded_pda<B: TestSVM>(backend: B) {
    use solana_account::Account;

    let mut world = World::new(backend,
        "Survive a pre-funded delegation PDA",
        "a griefer pre-funds the delegation PDA; Alice can still create it",
    );
    let payer = world.actor("alice");
    let amount: u64 = 100_000_000;
    let expiry_ts: i64 = world.now() + days(1) as i64;
    let nonce: u64 = 0;

    let mint = world.usdc_mint(&payer);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &payer, 1_000_000);

    world.init_authority(&payer, mint, None).0.assert_ok();

    let delegatee = solana_pubkey::Pubkey::new_unique();
    world.prop(delegatee, "delegatee");

    // Simulate Mallory pre-funding the delegation PDA address with lamports.
    let (subscription_authority_pda, _) = get_delegation_pda(
        &crate::tests::pda::get_subscription_authority_pda(&payer.pubkey(), &mint).0,
        &payer.pubkey(),
        &delegatee,
        nonce,
    );
    world.svm_mut()
        .set_account(
            &subscription_authority_pda,
            Account {
                lamports: 1_000,
                data: vec![],
                owner: solana_pubkey::Pubkey::default(), // system program
                executable: false,
                rent_epoch: 0,
            },
        );

    // The user should still be able to create the delegation PDA.
    world.md().step("Despite the pre-funded PDA, Alice creates the fixed delegation");
    let (ix, delegation_pda) = {
        CreateDelegation::new(world.svm_mut(), &payer, mint, delegatee).nonce(nonce).fixed_ix(amount, expiry_ts)
    };
    world.prop(delegation_pda, "FixedDelegation");
    world.send_ok(&[ix], &[&payer], "CreateFixedDelegation (pre-funded PDA)");

    let account = world.svm().get_account(&delegation_pda).unwrap();
    let delegation = FixedDelegation::load(&account.data).unwrap();

    let header = delegation.header;
    let del_amount = delegation.amount;
    let del_expiry_ts = delegation.expiry_ts;
    world.md().check("the delegator is Alice", payer.pubkey(), as_pubkey(header.delegator.to_bytes()));
    world.md().check("the delegatee matches", delegatee, as_pubkey(header.delegatee.to_bytes()));
    world.md().check(
        "the account is tagged FixedDelegation",
        AccountDiscriminator::FixedDelegation as u8,
        header.discriminator,
    );
    world.md().check("the delegated amount matches", amount, del_amount);
    world.md().check("the expiry matches", expiry_ts, del_expiry_ts);
}

// NOTE: These error tests use FixedDelegation but validate shared code paths.
// The same checks apply to RecurringDelegation via shared helpers.

pub fn create_delegation_without_subscription_authority<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Reject a delegation without a subscription authority",
        "creating a delegation before initializing the authority is refused",
    );
    let payer = world.actor("alice");

    let mint = world.usdc_mint(&payer);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &payer, 1_000_000);

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");

    world.md().step("Alice tries to create a delegation with no authority in place");
    let (ix, _) = { CreateDelegation::new(world.svm_mut(), &payer, mint, delegatee).nonce(0).fixed_ix(100, 1000) };
    let res = world.send(&[ix], &[&payer], "CreateFixedDelegation (no authority)");
    assert!(!res.is_success());
}

pub fn create_delegation_wrong_pda<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Reject a delegation at the wrong PDA",
        "an instruction pointed at a non-canonical delegation PDA is refused",
    );
    let payer = world.actor("alice");

    let mint = world.usdc_mint(&payer);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &payer, 1_000_000);

    world.init_authority(&payer, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");
    let wrong_pda = Pubkey::new_unique();

    world.md().step("Alice points the instruction at the wrong delegation PDA");
    let (ix, _) =
        { CreateDelegation::new(world.svm_mut(), &payer, mint, delegatee).pda(wrong_pda).nonce(0).fixed_ix(100, 1000) };
    let res = world.send(&[ix], &[&payer], "CreateFixedDelegation (wrong PDA)");
    assert!(!res.is_success());
}

pub fn create_delegation_duplicate_nonce<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Reject a duplicate delegation nonce",
        "creating a second delegation at the same nonce is refused",
    );
    let payer = world.actor("alice");

    let mint = world.usdc_mint(&payer);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &payer, 1_000_000);

    world.init_authority(&payer, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");

    world.md().step("Alice creates the first delegation at nonce 0");
    let now = world.now();
    let (ix1, _) =
        { CreateDelegation::new(world.svm_mut(), &payer, mint, delegatee).nonce(0).fixed_ix(100, now + 1000) };
    world.send_ok(&[ix1], &[&payer], "CreateFixedDelegation (nonce 0)");

    world.md().step("Alice retries at nonce 0; the program rejects the duplicate");
    let (ix2, _) =
        { CreateDelegation::new(world.svm_mut(), &payer, mint, delegatee).nonce(0).fixed_ix(200, now + 2000) };
    world.send_err(&[ix2], &[&payer], "CreateFixedDelegation (duplicate nonce)", SubscriptionsError::DelegationAlreadyExists);
}

pub fn create_multiple_delegations_different_nonces<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Create multiple delegations at different nonces",
        "distinct nonces derive distinct delegation PDAs",
    );
    let payer = world.actor("alice");

    let mint = world.usdc_mint(&payer);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &payer, 1_000_000);

    let (_, subscription_authority_pda, _) = world.init_authority(&payer, mint, None);

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");

    world.md().step("Alice creates three delegations at nonces 0, 1, 2");
    let now = world.now();
    let (ix0, pda0) =
        { CreateDelegation::new(world.svm_mut(), &payer, mint, delegatee).nonce(0).fixed_ix(100, now + 1000) };
    let tx = world.send_ok(&[ix0], &[&payer], "CreateFixedDelegation (nonce 0)");
    println!("Create Fixed delegation consumed: {} CUs", tx.compute_units);

    let (ix1, pda1) =
        { CreateDelegation::new(world.svm_mut(), &payer, mint, delegatee).nonce(1).fixed_ix(200, now + 2000) };
    let tx = world.send_ok(&[ix1], &[&payer], "CreateFixedDelegation (nonce 1)");
    println!("Create Fixed delegation consumed: {} CUs", tx.compute_units);

    let (ix2, pda2) =
        { CreateDelegation::new(world.svm_mut(), &payer, mint, delegatee).nonce(2).fixed_ix(300, now + 3000) };
    let tx = world.send_ok(&[ix2], &[&payer], "CreateFixedDelegation (nonce 2)");
    println!("Create Fixed delegation consumed: {} CUs", tx.compute_units);

    assert_ne!(pda0, pda1);
    assert_ne!(pda1, pda2);
    assert_ne!(pda0, pda2);

    let (expected_pda0, _) = get_delegation_pda(&subscription_authority_pda, &payer.pubkey(), &delegatee, 0);
    let (expected_pda1, _) = get_delegation_pda(&subscription_authority_pda, &payer.pubkey(), &delegatee, 1);
    let (expected_pda2, _) = get_delegation_pda(&subscription_authority_pda, &payer.pubkey(), &delegatee, 2);

    world.md().check("nonce 0 lands at the derived PDA", expected_pda0, pda0);
    world.md().check("nonce 1 lands at the derived PDA", expected_pda1, pda1);
    world.md().check("nonce 2 lands at the derived PDA", expected_pda2, pda2);
}

pub fn writable_accounts_must_be_writable<B: TestSVM>(backend: B) {
    use solana_instruction::{AccountMeta, Instruction};

    use crate::{
        instructions::create_fixed_delegation,
        tests::{constants::PROGRAM_ID, idl, pda::get_subscription_authority_pda},
    };

    let writable = idl::writable_account_indices("createFixedDelegation");

    let mut world = World::new(backend,
        "Create fixed delegation: writable accounts must be writable",
        "flipping any account the instruction writes to read-only is rejected",
    );
    let user = world.actor("alice");
    let fee_payer = world.actor("sponsor");

    let mint = world.usdc_mint(&user);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &user, 1_000_000);

    world.init_authority(&user, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");
    let nonce: u64 = 0;
    let (subscription_authority_pda, _) = get_subscription_authority_pda(&user.pubkey(), &mint);
    let expected_subscription_authority_init_id =
        crate::state::SubscriptionAuthority::load(&world.svm().get_account(&subscription_authority_pda).unwrap().data)
            .unwrap()
            .init_id;
    let (delegation_pda, _) =
        crate::tests::pda::get_delegation_pda(&subscription_authority_pda, &user.pubkey(), &delegatee, nonce);

    let now = world.now();
    for (idx, name, is_signer) in &writable {
        let mut accounts = vec![
            AccountMeta::new(user.pubkey(), true),
            AccountMeta::new_readonly(subscription_authority_pda, false),
            AccountMeta::new(delegation_pda, false),
            AccountMeta::new_readonly(delegatee, false),
            AccountMeta::new_readonly(crate::tests::constants::SYSTEM_PROGRAM_ID, false),
        ];

        // Flip writable account to readonly, preserving signer flag.
        let pubkey = accounts[*idx].pubkey;
        accounts[*idx] = AccountMeta::new_readonly(pubkey, *is_signer);

        let data = [
            vec![*create_fixed_delegation::DISCRIMINATOR],
            nonce.to_le_bytes().to_vec(),
            100u64.to_le_bytes().to_vec(),
            (now + 1000i64).to_le_bytes().to_vec(),
            expected_subscription_authority_init_id.to_le_bytes().to_vec(),
        ]
        .concat();

        let ix = Instruction { program_id: PROGRAM_ID, accounts, data };

        world.send_err(
            &[ix],
            &[&fee_payer, &user],
            &format!("CreateFixedDelegation ({name} forced read-only)"),
            SubscriptionsError::AccountNotWritable,
        );
    }
}

pub fn signer_accounts_must_be_signers<B: TestSVM>(backend: B) {
    use solana_instruction::{AccountMeta, Instruction};

    use crate::{
        instructions::create_fixed_delegation,
        tests::{constants::PROGRAM_ID, idl, pda::get_subscription_authority_pda},
    };

    let signers = idl::signer_account_indices("createFixedDelegation");

    let mut world = World::new(backend,
        "Create fixed delegation: signer accounts must sign",
        "flipping any required signer to non-signer is rejected",
    );
    let user = world.actor("alice");
    let fee_payer = world.actor("sponsor");

    let mint = world.usdc_mint(&user);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &user, 1_000_000);

    world.init_authority(&user, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");
    let nonce: u64 = 0;
    let (subscription_authority_pda, _) = get_subscription_authority_pda(&user.pubkey(), &mint);
    let expected_subscription_authority_init_id =
        crate::state::SubscriptionAuthority::load(&world.svm().get_account(&subscription_authority_pda).unwrap().data)
            .unwrap()
            .init_id;
    let (delegation_pda, _) =
        crate::tests::pda::get_delegation_pda(&subscription_authority_pda, &user.pubkey(), &delegatee, nonce);

    let now = world.now();
    for (idx, name, is_writable) in &signers {
        let mut accounts = vec![
            AccountMeta::new(user.pubkey(), true),
            AccountMeta::new_readonly(subscription_authority_pda, false),
            AccountMeta::new(delegation_pda, false),
            AccountMeta::new_readonly(delegatee, false),
            AccountMeta::new_readonly(crate::tests::constants::SYSTEM_PROGRAM_ID, false),
        ];

        // Flip signer to non-signer, preserving writable flag.
        let pubkey = accounts[*idx].pubkey;
        accounts[*idx] =
            if *is_writable { AccountMeta::new(pubkey, false) } else { AccountMeta::new_readonly(pubkey, false) };

        let data = [
            vec![*create_fixed_delegation::DISCRIMINATOR],
            nonce.to_le_bytes().to_vec(),
            100u64.to_le_bytes().to_vec(),
            (now + 1000i64).to_le_bytes().to_vec(),
            expected_subscription_authority_init_id.to_le_bytes().to_vec(),
        ]
        .concat();

        let ix = Instruction { program_id: PROGRAM_ID, accounts, data };

        world.send_err(
            &[ix],
            &[&fee_payer],
            &format!("CreateFixedDelegation ({name} forced non-signer)"),
            SubscriptionsError::NotSigner,
        );
    }
}

pub fn create_fixed_delegation_with_expiry_in_past<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Reject a fixed delegation with a past expiry",
        "an expiry timestamp before now is refused",
    );
    let payer = world.actor("alice");
    let amount: u64 = 100_000_000;
    let expiry_ts: i64 = -10000000;
    let nonce: u64 = 0;

    let mint = world.usdc_mint(&payer);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &payer, 1_000_000);

    world.init_authority(&payer, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");

    world.md().step("Alice creates a delegation with an expiry in the past; rejected");
    let (ix, _delegation_pda) = {
        CreateDelegation::new(world.svm_mut(), &payer, mint, delegatee).nonce(nonce).fixed_ix(amount, expiry_ts)
    };
    world.send_err(&[ix], &[&payer], "CreateFixedDelegation (past expiry)", SubscriptionsError::FixedDelegationExpiryInPast);
}

pub fn create_fixed_delegation_with_zero_expiry<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Create a fixed delegation with a zero expiry",
        "a zero expiry means no expiry; the delegatee can still pull later",
    );
    let payer = world.actor("alice");
    let amount: u64 = 100_000_000;
    let expiry_ts: i64 = 0;
    let nonce: u64 = 0;

    let mint = world.usdc_mint(&payer);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &payer, 100_000_000);

    world.init_authority(&payer, mint, None).0.assert_ok();

    let delegatee = world.actor("bob");
    let delegatee_ata = world.fund_ata(mint, &delegatee, 0);
    world.prop(delegatee_ata, "Bob's ATA");

    world.md().step("Alice creates a fixed delegation with a zero (no-expiry) timestamp");
    let (ix, delegation_pda) = {
        CreateDelegation::new(world.svm_mut(), &payer, mint, delegatee.pubkey()).nonce(nonce).fixed_ix(amount, expiry_ts)
    };
    world.prop(delegation_pda, "FixedDelegation");
    world.send_ok(&[ix], &[&payer], "CreateFixedDelegation (zero expiry)");

    let account = world.svm().get_account(&delegation_pda).unwrap();
    let delegation = FixedDelegation::load(&account.data).unwrap();
    let del_expiry_ts = delegation.expiry_ts;
    world.md().check("the stored expiry is zero", 0, del_expiry_ts);

    world.warp(days(30));

    world.md().step("Thirty days on, Bob pulls from the never-expiring delegation");
    let transfer_amount: u64 = 10_000_000;
    let transfer_ix = {
        TransferDelegation::new(world.svm_mut(), &delegatee, payer.pubkey(), mint, delegation_pda)
            .amount(transfer_amount)
            .fixed_ix()
    };
    world.send_ok(&[transfer_ix], &[&delegatee], "TransferFixed (after 30 days)");

    let bob_balance = token_balance(world.svm(), &delegatee_ata);
    world.md().check("Bob received the pulled amount", transfer_amount, bob_balance);
}
