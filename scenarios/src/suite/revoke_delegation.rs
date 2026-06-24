//! `revoke_delegation` (and `revoke_subscription`), converted to the World/scenario pattern.
//!
//! Each test builds its own `World`, draws actors from the cast (`alice` the
//! delegator/subscriber, `sponsor` the payer, `mallory` the adversary), and
//! routes every on-chain action through the observed `send_*` so the surface
//! renders into the test's report under `target/md-reports/`.
//!
//! Every balance assertion here is a tolerance inequality (rent recovered within
//! a 10_000-lamport slack, or a strict gain where the actor recovers their own
//! rent), all of which hold on a fee-less engine: charging no fee can only leave
//! the recovering party with more lamports, never fewer. So none of these need a
//! `world.capabilities().fees` gate.

use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;

use testsvm::TestSVM;

use crate::{
    tests::utils::{
            days, hours, init_mint, CancelSubscription, CreateDelegation, CreateSubscription, ObservedResultExt,
            RevokeDelegation, RevokeSubscription, World,
        },
    AccountDiscriminator, FixedDelegation, RecurringDelegation, SubscriptionsError,
};

pub fn revoke_fixed_delegation<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Revoke a fixed delegation",
        "the delegator revokes her own fixed delegation and recovers the rent",
    );
    let alice = world.actor("alice");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    world.init_authority(&alice, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");
    let nonce: u64 = 0;

    let expiry = world.now() + 1000;
    world.md().step("Alice creates a fixed delegation");
    let (ix, delegation_pda) = CreateDelegation::new(world.svm_mut(), &alice, mint, delegatee).nonce(nonce).fixed_ix(100, expiry);
    world.prop(delegation_pda, "Delegation");
    world.send_ok(&[ix], &[&alice], "CreateFixedDelegation");

    let account_before = world.svm().get_account(&delegation_pda);
    assert!(account_before.is_some());
    let binding = account_before.unwrap();
    let delegation_rent = binding.lamports;
    let delegation = FixedDelegation::load(&binding.data).unwrap();
    world.md().check(
        "the account is tagged FixedDelegation",
        AccountDiscriminator::FixedDelegation as u8,
        delegation.header.discriminator,
    );

    let delegator_balance_before = world.svm().get_account(&alice.pubkey()).unwrap().lamports;

    world.md().step("Alice revokes the delegation");
    let ix = RevokeDelegation::new(world.svm_mut(), &alice, mint, delegatee, nonce).instruction();
    world.send_ok(&[ix], &[&alice], "RevokeDelegation");

    let account_after = world.svm().get_account(&delegation_pda);
    assert!(account_after.is_none() || account_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0);

    let delegator_balance_after = world.svm().get_account(&alice.pubkey()).unwrap().lamports;
    world.md().check("the rent flows back to Alice", true, delegator_balance_after > delegator_balance_before);
    assert!(delegator_balance_after >= delegator_balance_before + delegation_rent - 10000);
}

pub fn revoke_recurring_delegation<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Revoke a recurring delegation",
        "the delegator revokes her own recurring delegation and recovers the rent",
    );
    let alice = world.actor("alice");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    world.init_authority(&alice, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");
    let nonce: u64 = 0;

    let epoch = days(1);
    let start_ts = world.now();
    let expiry_ts = world.now() + days(2) as i64;
    world.md().step("Alice creates a recurring delegation");
    let (ix, delegation_pda) =
        CreateDelegation::new(world.svm_mut(), &alice, mint, delegatee).nonce(nonce).recurring_ix(100, epoch, start_ts, expiry_ts);
    world.prop(delegation_pda, "Delegation");
    world.send_ok(&[ix], &[&alice], "CreateRecurringDelegation");

    let account_before = world.svm().get_account(&delegation_pda);
    assert!(account_before.is_some());
    let binding = account_before.unwrap();
    let delegation_rent = binding.lamports;
    let delegation = RecurringDelegation::load(&binding.data).unwrap();
    world.md().check(
        "the account is tagged RecurringDelegation",
        AccountDiscriminator::RecurringDelegation as u8,
        delegation.header.discriminator,
    );

    let delegator_balance_before = world.svm().get_account(&alice.pubkey()).unwrap().lamports;

    world.md().step("Alice revokes the delegation");
    let ix = RevokeDelegation::new(world.svm_mut(), &alice, mint, delegatee, nonce).instruction();
    world.send_ok(&[ix], &[&alice], "RevokeDelegation");

    let account_after = world.svm().get_account(&delegation_pda);
    assert!(account_after.is_none() || account_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0);

    let delegator_balance_after = world.svm().get_account(&alice.pubkey()).unwrap().lamports;
    world.md().check("the rent flows back to Alice", true, delegator_balance_after > delegator_balance_before);
    assert!(delegator_balance_after >= delegator_balance_before + delegation_rent - 10000);
}

pub fn non_delegator_cannot_revoke<B: TestSVM>(backend: B) {
    use solana_instruction::{AccountMeta, Instruction};

    use crate::{instructions::revoke_delegation, tests::constants::PROGRAM_ID};

    let mut world = World::new(backend,
        "A non-delegator cannot revoke",
        "Mallory, who is neither delegator nor payer, cannot revoke a live delegation",
    );
    let alice = world.actor("alice");
    let mallory = world.actor("mallory");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    world.init_authority(&alice, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");
    let nonce: u64 = 0;

    let epoch = days(1);
    let start_ts = world.now();
    let expiry_ts = world.now() + days(2) as i64;
    world.md().step("Alice creates a recurring delegation");
    let (ix, delegation_pda) =
        CreateDelegation::new(world.svm_mut(), &alice, mint, delegatee).nonce(nonce).recurring_ix(100, epoch, start_ts, expiry_ts);
    world.prop(delegation_pda, "Delegation");
    world.send_ok(&[ix], &[&alice], "CreateRecurringDelegation");

    // Mallory hand-builds a revoke and tries to close the delegation.
    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![AccountMeta::new(mallory.pubkey(), true), AccountMeta::new(delegation_pda, false)],
        data: vec![*revoke_delegation::DISCRIMINATOR],
    };
    world.md().step("Mallory tries to revoke Alice's delegation");
    let res = world.send(&[ix], &[&mallory], "RevokeDelegation (by Mallory)");
    assert!(!res.is_success());

    let account_after = world.svm().get_account(&delegation_pda);
    assert!(account_after.is_some());
    assert!(account_after.as_ref().map(|a| a.lamports).unwrap_or(0) > 0);
}

pub fn closed_account_is_zeroed<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "A closed delegation account is zeroed",
        "after revoke, any residual delegation bytes are all zero",
    );
    let alice = world.actor("alice");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    world.init_authority(&alice, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");
    let nonce: u64 = 0;

    let expiry = world.now() + 1000;
    let (ix, delegation_pda) = CreateDelegation::new(world.svm_mut(), &alice, mint, delegatee).nonce(nonce).fixed_ix(100, expiry);
    world.prop(delegation_pda, "Delegation");
    world.send_ok(&[ix], &[&alice], "CreateFixedDelegation");

    let account_before = world.svm().get_account(&delegation_pda);
    let _before_data = account_before.as_ref().unwrap().data.clone();

    world.md().step("Alice revokes the delegation");
    let ix = RevokeDelegation::new(world.svm_mut(), &alice, mint, delegatee, nonce).instruction();
    world.send_ok(&[ix], &[&alice], "RevokeDelegation");

    let account_after = world.svm().get_account(&delegation_pda);

    if let Some(account) = account_after {
        assert!(account.data.iter().all(|&byte| byte == 0), "All data should be zeroed after close");
    }
}

pub fn revoke_with_wrong_receiver_returns_unauthorized<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Revoke with a wrong receiver is unauthorized",
        "a sponsor-funded delegation cannot route rent to an arbitrary receiver",
    );
    let alice = world.actor("alice");
    let sponsor = world.actor("sponsor");
    let wrong_receiver = Pubkey::new_unique();
    world.prop(wrong_receiver, "wrong receiver");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    world.init_authority(&alice, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");
    let nonce: u64 = 0;

    let expiry = world.now() + 1000;
    world.md().step("Alice creates a sponsor-funded fixed delegation");
    let (ix, _) = CreateDelegation::new(world.svm_mut(), &alice, mint, delegatee)
        .payer(&sponsor)
        .nonce(nonce)
        .fixed_ix(100, expiry);
    world.send_ok(&[ix], &[&sponsor, &alice], "CreateFixedDelegation (sponsored)");

    world.md().step("Alice revokes but points the rent at a wrong receiver");
    let ix = RevokeDelegation::new(world.svm_mut(), &alice, mint, delegatee, nonce).receiver(wrong_receiver).instruction();
    world.send_err(&[ix], &[&alice], "RevokeDelegation (wrong receiver)", SubscriptionsError::Unauthorized);
}

pub fn writable_accounts_must_be_writable<B: TestSVM>(backend: B) {
    use solana_instruction::{AccountMeta, Instruction};

    use crate::{
        instructions::revoke_delegation,
        tests::{constants::PROGRAM_ID, idl},
    };

    let writable = idl::writable_account_indices("revokeDelegation");

    let mut world = World::new(backend,
        "Revoke: writable accounts must be writable",
        "flipping any account the revoke writes to read-only is rejected",
    );
    let alice = world.actor("alice");
    let fee_payer = world.actor("sponsor");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    world.init_authority(&alice, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");
    let nonce: u64 = 0;

    let expiry = world.now() + 1000;
    let (ix, delegation_pda) = CreateDelegation::new(world.svm_mut(), &alice, mint, delegatee).nonce(nonce).fixed_ix(100, expiry);
    world.prop(delegation_pda, "Delegation");
    world.send_ok(&[ix], &[&alice], "CreateFixedDelegation");

    for (idx, name, is_signer) in &writable {
        let mut accounts = vec![AccountMeta::new(alice.pubkey(), true), AccountMeta::new(delegation_pda, false)];

        // Flip writable account to readonly, preserving signer flag.
        let pubkey = accounts[*idx].pubkey;
        accounts[*idx] = AccountMeta::new_readonly(pubkey, *is_signer);

        let ix = Instruction { program_id: PROGRAM_ID, accounts, data: vec![*revoke_delegation::DISCRIMINATOR] };

        world.send_err(
            &[ix],
            &[&fee_payer, &alice],
            &format!("RevokeDelegation ({name} forced read-only)"),
            SubscriptionsError::AccountNotWritable,
        );
    }
}

pub fn signer_accounts_must_be_signers<B: TestSVM>(backend: B) {
    use solana_instruction::{AccountMeta, Instruction};

    use crate::{
        instructions::revoke_delegation,
        tests::{constants::PROGRAM_ID, idl},
    };

    let signers = idl::signer_account_indices("revokeDelegation");

    let mut world = World::new(backend,
        "Revoke: signer accounts must sign",
        "flipping any required signer to non-signer is rejected",
    );
    let alice = world.actor("alice");
    let fee_payer = world.actor("sponsor");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    world.init_authority(&alice, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");
    let nonce: u64 = 0;

    let expiry = world.now() + 1000;
    let (ix, delegation_pda) = CreateDelegation::new(world.svm_mut(), &alice, mint, delegatee).nonce(nonce).fixed_ix(100, expiry);
    world.prop(delegation_pda, "Delegation");
    world.send_ok(&[ix], &[&alice], "CreateFixedDelegation");

    for (idx, name, is_writable) in &signers {
        let mut accounts = vec![AccountMeta::new(alice.pubkey(), true), AccountMeta::new(delegation_pda, false)];

        // Flip signer to non-signer, preserving writable flag.
        let pubkey = accounts[*idx].pubkey;
        accounts[*idx] =
            if *is_writable { AccountMeta::new(pubkey, false) } else { AccountMeta::new_readonly(pubkey, false) };

        let ix = Instruction { program_id: PROGRAM_ID, accounts, data: vec![*revoke_delegation::DISCRIMINATOR] };

        world.send_err(
            &[ix],
            &[&fee_payer],
            &format!("RevokeDelegation ({name} forced non-signer)"),
            SubscriptionsError::NotSigner,
        );
    }
}

pub fn revoke_subscription_without_cancel_rejected<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Revoke a subscription without cancelling is rejected",
        "a live subscription cannot be revoked before it is cancelled",
    );
    let s = world.stage_subscription();

    world.md().step("Alice tries to revoke without cancelling first");
    let ix = RevokeSubscription::new(world.svm_mut(), &s.alice, s.subscription_pda, s.plan_pda).instruction();
    world.send_err(&[ix], &[&s.alice], "RevokeSubscription (not cancelled)", SubscriptionsError::SubscriptionNotCancelled);

    // Account should still exist.
    let account = world.svm().get_account(&s.subscription_pda);
    assert!(account.is_some());
}

pub fn revoke_subscription_after_cancel_succeeds<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Revoke a subscription after cancel",
        "once cancelled and past its period, a subscription can be revoked and the rent returns",
    );
    let s = world.stage_subscription();

    let balance_before = world.svm().get_account(&s.alice.pubkey()).unwrap().lamports;

    world.md().step("Alice cancels the subscription");
    let ix = CancelSubscription::new(world.svm_mut(), &s.alice, s.plan_pda, s.subscription_pda).instruction();
    world.send_ok(&[ix], &[&s.alice], "CancelSubscription");

    // Advance clock past the expiration (plan has 1h period).
    world.warp(hours(1));

    world.md().step("Alice revokes the cancelled subscription");
    let ix = RevokeSubscription::new(world.svm_mut(), &s.alice, s.subscription_pda, s.plan_pda).instruction();
    world.send_ok(&[ix], &[&s.alice], "RevokeSubscription");

    // Account should be closed.
    let account = world.svm().get_account(&s.subscription_pda);
    assert!(
        account.is_none() || account.as_ref().map(|a| a.lamports).unwrap_or(0) == 0,
        "Subscription PDA should be closed"
    );

    // Rent should be returned.
    let balance_after = world.svm().get_account(&s.alice.pubkey()).unwrap().lamports;
    assert!(balance_after > balance_before - 10000);
}

pub fn revoke_subscription_with_future_expires_at_ts_rejected<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Revoke a subscription with a future expiry is rejected",
        "a subscription whose expires_at_ts is in the future is not yet revocable",
    );
    let s = world.stage_subscription();

    // Manually inject a subscription with expires_at_ts in the future.
    let now = world.now();
    let subscription_pda = CreateSubscription::new(world.svm_mut(), s.plan_pda, s.alice.pubkey(), s.mint, now)
        .expires_at_ts(now + days(1) as i64)
        .execute();
    world.prop(subscription_pda, "Subscription");

    world.md().step("Alice tries to revoke a subscription whose period hasn't ended");
    let ix = RevokeSubscription::new(world.svm_mut(), &s.alice, subscription_pda, s.plan_pda).instruction();
    world.send_err(&[ix], &[&s.alice], "RevokeSubscription (future expiry)", SubscriptionsError::SubscriptionNotCancelled);

    // Account should still exist.
    let account = world.svm().get_account(&subscription_pda);
    assert!(account.is_some());
}

pub fn test_revoke_fixed_version_agnostic<B: TestSVM>(backend: B) {
    use crate::state::header::VERSION_OFFSET;

    let mut world = World::new(backend,
        "Revoke a fixed delegation across versions",
        "revoke works regardless of the stored header version byte",
    );
    let alice = world.actor("alice");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    world.init_authority(&alice, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");
    let nonce: u64 = 0;

    let expiry = world.now() + 1000;
    let (ix, delegation_pda) = CreateDelegation::new(world.svm_mut(), &alice, mint, delegatee).nonce(nonce).fixed_ix(100, expiry);
    world.prop(delegation_pda, "Delegation");
    world.send_ok(&[ix], &[&alice], "CreateFixedDelegation");

    let mut account = world.svm().get_account(&delegation_pda).unwrap();
    account.data[VERSION_OFFSET] = 0;
    world.svm_mut().set_account(&delegation_pda, account);

    world.md().step("Alice revokes a delegation with a zeroed version byte");
    let ix = RevokeDelegation::new(world.svm_mut(), &alice, mint, delegatee, nonce).instruction();
    world.send_ok(&[ix], &[&alice], "RevokeDelegation");

    let account_after = world.svm().get_account(&delegation_pda);
    assert!(account_after.is_none() || account_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0);
}

pub fn test_revoke_recurring_version_agnostic<B: TestSVM>(backend: B) {
    use crate::state::header::VERSION_OFFSET;

    let mut world = World::new(backend,
        "Revoke a recurring delegation across versions",
        "revoke works regardless of the stored header version byte",
    );
    let alice = world.actor("alice");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    world.init_authority(&alice, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");
    let nonce: u64 = 0;

    let start_ts = world.now();
    let expiry_ts = world.now() + days(2) as i64;
    let (ix, delegation_pda) =
        CreateDelegation::new(world.svm_mut(), &alice, mint, delegatee).nonce(nonce).recurring_ix(100, days(1), start_ts, expiry_ts);
    world.prop(delegation_pda, "Delegation");
    world.send_ok(&[ix], &[&alice], "CreateRecurringDelegation");

    let mut account = world.svm().get_account(&delegation_pda).unwrap();
    account.data[VERSION_OFFSET] = 0;
    world.svm_mut().set_account(&delegation_pda, account);

    world.md().step("Alice revokes a delegation with a zeroed version byte");
    let ix = RevokeDelegation::new(world.svm_mut(), &alice, mint, delegatee, nonce).instruction();
    world.send_ok(&[ix], &[&alice], "RevokeDelegation");

    let account_after = world.svm().get_account(&delegation_pda);
    assert!(account_after.is_none() || account_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0);
}

pub fn test_revoke_subscription_version_mismatch<B: TestSVM>(backend: B) {
    use crate::state::header::VERSION_OFFSET;

    let mut world = World::new(backend,
        "Revoke a subscription with a version mismatch",
        "a subscription whose version byte was tampered requires migration before revoke",
    );
    let s = world.stage_subscription();

    world.md().step("Alice cancels the subscription");
    let ix = CancelSubscription::new(world.svm_mut(), &s.alice, s.plan_pda, s.subscription_pda).instruction();
    world.send_ok(&[ix], &[&s.alice], "CancelSubscription");

    world.warp(hours(1));

    let mut account = world.svm().get_account(&s.subscription_pda).unwrap();
    account.data[VERSION_OFFSET] = 0;
    world.svm_mut().set_account(&s.subscription_pda, account);

    world.md().step("Alice tries to revoke a subscription with a zeroed version byte");
    let ix = RevokeSubscription::new(world.svm_mut(), &s.alice, s.subscription_pda, s.plan_pda).instruction();
    world.send_err(&[ix], &[&s.alice], "RevokeSubscription (version mismatch)", SubscriptionsError::MigrationRequired);
}

pub fn sponsor_can_revoke_expired_fixed_delegation<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "A sponsor can revoke an expired fixed delegation",
        "once the fixed delegation has expired past the drift window, the sponsor recovers its rent",
    );
    let delegator = world.actor("alice");
    let sponsor = world.actor("sponsor");

    let mint = world.usdc_mint(&delegator);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &delegator, 1_000_000);

    world.init_authority(&delegator, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");
    let nonce: u64 = 0;
    let expiry_ts = world.now() + hours(1) as i64;

    world.md().step("Alice creates a sponsor-funded fixed delegation");
    let (ix, delegation_pda) =
        CreateDelegation::new(world.svm_mut(), &delegator, mint, delegatee).payer(&sponsor).nonce(nonce).fixed_ix(100, expiry_ts);
    world.prop(delegation_pda, "Delegation");
    world.send_ok(&[ix], &[&sponsor, &delegator], "CreateFixedDelegation (sponsored)");

    let delegation_rent = world.svm().get_account(&delegation_pda).unwrap().lamports;

    world.warp(hours(2));

    let sponsor_balance_before = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;

    world.md().step("The sponsor revokes the expired delegation");
    let ix = RevokeDelegation::new(world.svm_mut(), &delegator, mint, delegatee, nonce).signer(&sponsor).instruction();
    world.send_ok(&[ix], &[&sponsor], "RevokeDelegation (by sponsor)");

    let account_after = world.svm().get_account(&delegation_pda);
    assert!(account_after.is_none() || account_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0);

    let sponsor_balance_after = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;
    assert!(sponsor_balance_after >= sponsor_balance_before + delegation_rent - 10000);
}

pub fn sponsor_can_revoke_expired_recurring_delegation<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "A sponsor can revoke an expired recurring delegation",
        "once the recurring delegation has expired past the drift window, the sponsor recovers its rent",
    );
    let delegator = world.actor("alice");
    let sponsor = world.actor("sponsor");

    let mint = world.usdc_mint(&delegator);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &delegator, 1_000_000);

    world.init_authority(&delegator, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");
    let nonce: u64 = 0;
    let start_ts = world.now();
    let expiry_ts = world.now() + days(2) as i64;

    world.md().step("Alice creates a sponsor-funded recurring delegation");
    let (ix, delegation_pda) = CreateDelegation::new(world.svm_mut(), &delegator, mint, delegatee)
        .payer(&sponsor)
        .nonce(nonce)
        .recurring_ix(100, days(1), start_ts, expiry_ts);
    world.prop(delegation_pda, "Delegation");
    world.send_ok(&[ix], &[&sponsor, &delegator], "CreateRecurringDelegation (sponsored)");

    let delegation_rent = world.svm().get_account(&delegation_pda).unwrap().lamports;

    world.warp(days(3));

    let sponsor_balance_before = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;

    world.md().step("The sponsor revokes the expired delegation");
    let ix = RevokeDelegation::new(world.svm_mut(), &delegator, mint, delegatee, nonce).signer(&sponsor).instruction();
    world.send_ok(&[ix], &[&sponsor], "RevokeDelegation (by sponsor)");

    let account_after = world.svm().get_account(&delegation_pda);
    assert!(account_after.is_none() || account_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0);

    let sponsor_balance_after = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;
    assert!(sponsor_balance_after >= sponsor_balance_before + delegation_rent - 10000);
}

pub fn sponsor_cannot_revoke_non_expired_fixed_delegation<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "A sponsor cannot revoke a non-expired fixed delegation",
        "while the fixed delegation is live, only the delegator may revoke it",
    );
    let delegator = world.actor("alice");
    let sponsor = world.actor("sponsor");

    let mint = world.usdc_mint(&delegator);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &delegator, 1_000_000);

    world.init_authority(&delegator, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");
    let nonce: u64 = 0;
    let expiry_ts = world.now() + hours(2) as i64;

    world.md().step("Alice creates a sponsor-funded fixed delegation");
    let (ix, _) =
        CreateDelegation::new(world.svm_mut(), &delegator, mint, delegatee).payer(&sponsor).nonce(nonce).fixed_ix(100, expiry_ts);
    world.send_ok(&[ix], &[&sponsor, &delegator], "CreateFixedDelegation (sponsored)");

    world.md().step("The sponsor tries to revoke before expiry");
    let ix = RevokeDelegation::new(world.svm_mut(), &delegator, mint, delegatee, nonce).signer(&sponsor).instruction();
    world.send_err(&[ix], &[&sponsor], "RevokeDelegation (by sponsor, premature)", SubscriptionsError::Unauthorized);
}

pub fn sponsor_cannot_revoke_non_expired_recurring_delegation<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "A sponsor cannot revoke a non-expired recurring delegation",
        "while the recurring delegation is live, only the delegator may revoke it",
    );
    let delegator = world.actor("alice");
    let sponsor = world.actor("sponsor");

    let mint = world.usdc_mint(&delegator);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &delegator, 1_000_000);

    world.init_authority(&delegator, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");
    let nonce: u64 = 0;
    let start_ts = world.now();
    let expiry_ts = world.now() + days(2) as i64;

    world.md().step("Alice creates a sponsor-funded recurring delegation");
    let (ix, _) = CreateDelegation::new(world.svm_mut(), &delegator, mint, delegatee).payer(&sponsor).nonce(nonce).recurring_ix(
        100,
        days(1),
        start_ts,
        expiry_ts,
    );
    world.send_ok(&[ix], &[&sponsor, &delegator], "CreateRecurringDelegation (sponsored)");

    world.md().step("The sponsor tries to revoke before expiry");
    let ix = RevokeDelegation::new(world.svm_mut(), &delegator, mint, delegatee, nonce).signer(&sponsor).instruction();
    world.send_err(&[ix], &[&sponsor], "RevokeDelegation (by sponsor, premature)", SubscriptionsError::Unauthorized);
}

pub fn sponsor_cannot_revoke_no_expiry_delegation<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "A sponsor cannot revoke a no-expiry delegation",
        "a delegation with no expiry never becomes sponsor-revocable, even far in the future",
    );
    let delegator = world.actor("alice");
    let sponsor = world.actor("sponsor");

    let mint = world.usdc_mint(&delegator);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &delegator, 1_000_000);

    world.init_authority(&delegator, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");
    let nonce: u64 = 0;

    world.md().step("Alice creates a sponsor-funded delegation with no expiry");
    let (ix, _) =
        CreateDelegation::new(world.svm_mut(), &delegator, mint, delegatee).payer(&sponsor).nonce(nonce).fixed_ix(100, 0);
    world.send_ok(&[ix], &[&sponsor, &delegator], "CreateFixedDelegation (sponsored, no expiry)");

    world.warp(days(365));

    world.md().step("The sponsor tries to revoke a no-expiry delegation a year later");
    let ix = RevokeDelegation::new(world.svm_mut(), &delegator, mint, delegatee, nonce).signer(&sponsor).instruction();
    world.send_err(&[ix], &[&sponsor], "RevokeDelegation (by sponsor, no expiry)", SubscriptionsError::Unauthorized);
}

pub fn sponsor_cannot_revoke_within_drift_window<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "A sponsor cannot revoke within the drift window",
        "the sponsor is held off until 120s past expiry, then allowed",
    );
    let delegator = world.actor("alice");
    let sponsor = world.actor("sponsor");

    let mint = world.usdc_mint(&delegator);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &delegator, 1_000_000);

    world.init_authority(&delegator, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");
    let nonce: u64 = 0;
    let expiry_ts = world.now() + 100;

    world.md().step("Alice creates a sponsor-funded fixed delegation expiring in 100s");
    let (ix, _) =
        CreateDelegation::new(world.svm_mut(), &delegator, mint, delegatee).payer(&sponsor).nonce(nonce).fixed_ix(100, expiry_ts);
    world.send_ok(&[ix], &[&sponsor, &delegator], "CreateFixedDelegation (sponsored)");

    // 110s after creation: past expiry but still within 120s drift window.
    world.warp(110);

    world.md().step("Within the drift window, the sponsor is refused");
    let ix = RevokeDelegation::new(world.svm_mut(), &delegator, mint, delegatee, nonce).signer(&sponsor).instruction();
    world.send_err(&[ix], &[&sponsor], "RevokeDelegation (within drift window)", SubscriptionsError::Unauthorized);

    // Past the drift window: sponsor can revoke.
    world.warp(121);

    world.md().step("Past the drift window, the sponsor can revoke");
    let ix = RevokeDelegation::new(world.svm_mut(), &delegator, mint, delegatee, nonce).signer(&sponsor).instruction();
    world.send_ok(&[ix], &[&sponsor], "RevokeDelegation (past drift window)");
}

pub fn delegator_can_revoke_sponsor_funded_before_expiry<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "The delegator can revoke a sponsor-funded delegation before expiry",
        "the delegator revokes early and rent flows to the sponsor (the recorded payer)",
    );
    let delegator = world.actor("alice");
    let sponsor = world.actor("sponsor");

    let mint = world.usdc_mint(&delegator);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &delegator, 1_000_000);

    world.init_authority(&delegator, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");
    let nonce: u64 = 0;
    let expiry_ts = world.now() + hours(2) as i64;

    world.md().step("Alice creates a sponsor-funded fixed delegation");
    let (ix, delegation_pda) =
        CreateDelegation::new(world.svm_mut(), &delegator, mint, delegatee).payer(&sponsor).nonce(nonce).fixed_ix(100, expiry_ts);
    world.prop(delegation_pda, "Delegation");
    world.send_ok(&[ix], &[&sponsor, &delegator], "CreateFixedDelegation (sponsored)");

    let delegation_rent = world.svm().get_account(&delegation_pda).unwrap().lamports;
    let sponsor_balance_before = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;

    world.md().step("Alice revokes early, routing the rent to the sponsor");
    let ix = RevokeDelegation::new(world.svm_mut(), &delegator, mint, delegatee, nonce).receiver(sponsor.pubkey()).instruction();
    world.send_ok(&[ix], &[&delegator], "RevokeDelegation (rent to sponsor)");

    let account_after = world.svm().get_account(&delegation_pda);
    assert!(account_after.is_none() || account_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0);

    let sponsor_balance_after = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;
    assert!(sponsor_balance_after >= sponsor_balance_before + delegation_rent - 10000);
}

pub fn attacker_cannot_revoke_sponsor_funded_delegation<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Mallory cannot revoke a sponsor-funded delegation",
        "Mallory cannot revoke even after expiry, despite naming the sponsor as receiver",
    );
    let delegator = world.actor("alice");
    let sponsor = world.actor("sponsor");
    let mallory = world.actor("mallory");

    let mint = world.usdc_mint(&delegator);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &delegator, 1_000_000);

    world.init_authority(&delegator, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");
    let nonce: u64 = 0;
    let expiry_ts = world.now() + hours(1) as i64;

    world.md().step("Alice creates a sponsor-funded fixed delegation");
    let (ix, _) =
        CreateDelegation::new(world.svm_mut(), &delegator, mint, delegatee).payer(&sponsor).nonce(nonce).fixed_ix(100, expiry_ts);
    world.send_ok(&[ix], &[&sponsor, &delegator], "CreateFixedDelegation (sponsored)");

    world.warp(hours(2));

    // Mallory passes sponsor as receiver to try to close the account.
    world.md().step("Mallory tries to revoke, naming the sponsor as receiver");
    let ix = RevokeDelegation::new(world.svm_mut(), &delegator, mint, delegatee, nonce)
        .signer(&mallory)
        .receiver(sponsor.pubkey())
        .instruction();
    world.send_err(&[ix], &[&mallory], "RevokeDelegation (by Mallory)", SubscriptionsError::Unauthorized);
}

/// Helper: spin up a sponsor-funded subscription, returning everything callers
/// need to drive subsequent revoke-subscription tests. Builds the world's state
/// through the observed sends so each staging action renders into the report.
fn setup_sponsored_subscription<B: TestSVM>(
    world: &mut World<B>,
    plan_end_ts: i64,
) -> (
    Keypair, // alice (subscriber)
    Keypair, // merchant
    Keypair, // sponsor
    Pubkey,  // plan_pda
    Pubkey,  // subscription_pda
) {
    use crate::tests::{
        pda::{get_plan_pda, get_subscription_pda},
        utils::{CreatePlan, Subscribe},
    };

    let alice = world.actor("alice");
    let merchant = world.actor("merchant");
    let sponsor = world.actor("sponsor");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _alice_ata = world.fund_ata(mint, &alice, 100_000_000);

    world.md().step("Stage: Alice's authority, the merchant's plan, Alice subscribed (sponsored)");
    world.init_authority(&alice, mint, None).0.assert_ok();

    let plan_ix = CreatePlan::new(world.svm_mut(), &merchant, mint)
        .plan_id(1)
        .amount(50_000_000)
        .period_hours(1)
        .end_ts(plan_end_ts)
        .instruction();
    let (plan_pda, plan_bump) = get_plan_pda(&merchant.pubkey(), 1);
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&merchant], "CreatePlan");

    let sub_ix = Subscribe::new(world.svm_mut(), &alice, merchant.pubkey(), plan_pda, 1, plan_bump, mint)
        .payer(&sponsor)
        .instruction();
    let (subscription_pda, _) = get_subscription_pda(&plan_pda, &alice.pubkey());
    world.prop(subscription_pda, "Subscription");
    world.send_ok(&[sub_ix], &[&sponsor, &alice], "Subscribe (sponsored)");

    (alice, merchant, sponsor, plan_pda, subscription_pda)
}

pub fn sponsor_revoke_subscription_when_plan_ended<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "A sponsor can revoke a subscription once the plan ended",
        "once the plan end_ts passes, the sponsor recovers the subscription rent",
    );
    let plan_end_ts = world.now() + hours(2) as i64;
    let (_alice, _merchant, sponsor, plan_pda, subscription_pda) = setup_sponsored_subscription(&mut world, plan_end_ts);

    let sub_rent = world.svm().get_account(&subscription_pda).unwrap().lamports;
    let sponsor_balance_before = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;

    // Move past plan end.
    world.warp(hours(3));

    world.md().step("The sponsor revokes the subscription whose plan has ended");
    let ix = RevokeSubscription::new(world.svm_mut(), &sponsor, subscription_pda, plan_pda).instruction();
    world.send_ok(&[ix], &[&sponsor], "RevokeSubscription (plan ended)");

    let account_after = world.svm().get_account(&subscription_pda);
    assert!(account_after.is_none() || account_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0);

    let sponsor_balance_after = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;
    assert!(sponsor_balance_after >= sponsor_balance_before + sub_rent - 10_000);
}

pub fn sponsor_revoke_subscription_when_plan_closed<B: TestSVM>(backend: B) {
    use crate::{state::common::PlanStatus, tests::utils::{DeletePlan, UpdatePlan}};

    let mut world = World::new(backend,
        "A sponsor can revoke a subscription once the plan is closed",
        "after the merchant sunsets and deletes the plan, the sponsor can revoke the subscription",
    );
    let plan_end_ts = world.now() + hours(2) as i64;
    let (_alice, merchant, sponsor, plan_pda, subscription_pda) = setup_sponsored_subscription(&mut world, plan_end_ts);

    // Sunset, expire, and delete the plan.
    world.md().step("The merchant sunsets the plan");
    let ix = UpdatePlan::new(world.svm_mut(), &merchant, plan_pda).status(PlanStatus::Sunset).end_ts(plan_end_ts).instruction();
    world.send_ok(&[ix], &[&merchant], "UpdatePlan (sunset)");

    world.warp(hours(3));

    world.md().step("The merchant deletes the expired plan");
    let ix = DeletePlan::new(world.svm_mut(), &merchant, plan_pda).instruction();
    world.send_ok(&[ix], &[&merchant], "DeletePlan");

    // Plan account is now system-owned (closed). Sponsor can revoke.
    world.md().step("The sponsor revokes against the closed plan");
    let ix = RevokeSubscription::new(world.svm_mut(), &sponsor, subscription_pda, plan_pda).instruction();
    world.send_ok(&[ix], &[&sponsor], "RevokeSubscription (plan closed)");

    let account_after = world.svm().get_account(&subscription_pda);
    assert!(account_after.is_none() || account_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0);
}

pub fn sponsor_revoke_subscription_when_plan_recreated_with_different_terms<B: TestSVM>(backend: B) {
    // Same-address ghost plan: merchant deletes the expired plan and
    // recreates it under the same `plan_id` with different terms. The
    // subscription is no longer pull-eligible (transfers fail via
    // `check_plan_terms`), and the sponsor should be able to recover rent
    // unilaterally even though `plan_closed` is false on the recreated PDA.
    use crate::{state::common::PlanStatus, tests::utils::{CreatePlan, DeletePlan, UpdatePlan}};

    let mut world = World::new(backend,
        "A sponsor can revoke against a recreated ghost plan",
        "a plan recreated under the same id with new terms still lets the sponsor recover rent",
    );
    let plan_end_ts = world.now() + hours(2) as i64;
    let (_alice, merchant, sponsor, plan_pda, subscription_pda) = setup_sponsored_subscription(&mut world, plan_end_ts);

    // Sunset, expire, delete.
    world.md().step("The merchant sunsets the plan");
    let ix = UpdatePlan::new(world.svm_mut(), &merchant, plan_pda).status(PlanStatus::Sunset).end_ts(plan_end_ts).instruction();
    world.send_ok(&[ix], &[&merchant], "UpdatePlan (sunset)");

    world.warp(hours(3));

    world.md().step("The merchant deletes the expired plan");
    let ix = DeletePlan::new(world.svm_mut(), &merchant, plan_pda).instruction();
    world.send_ok(&[ix], &[&merchant], "DeletePlan");

    // Recreate the same plan_id with different terms (ghost plan). End_ts
    // is in the future so neither plan_ended nor plan_closed would fire.
    let new_end_ts = world.now() + days(60) as i64;
    let mint = init_mint(
        world.svm_mut(),
        crate::tests::constants::TOKEN_PROGRAM_ID,
        crate::tests::constants::MINT_DECIMALS,
        1_000_000_000,
        None,
        &[],
    );
    world.md().step("The merchant recreates the plan with different terms (a ghost plan)");
    let plan_ix = CreatePlan::new(world.svm_mut(), &merchant, mint)
        .plan_id(1)
        .amount(999_000_000)
        .period_hours(720)
        .end_ts(new_end_ts)
        .instruction();
    let recreated_plan_pda = crate::tests::pda::get_plan_pda(&merchant.pubkey(), 1).0;
    world.send_ok(&[plan_ix], &[&merchant], "CreatePlan (recreated)");
    world.md().check("the recreated plan reuses the same PDA", plan_pda, recreated_plan_pda);

    let sub_rent = world.svm().get_account(&subscription_pda).unwrap().lamports;
    let sponsor_balance_before = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;

    world.md().step("The sponsor revokes against the ghost plan");
    let ix = RevokeSubscription::new(world.svm_mut(), &sponsor, subscription_pda, plan_pda).instruction();
    world.send_ok(&[ix], &[&sponsor], "RevokeSubscription (ghost plan)");

    let account_after = world.svm().get_account(&subscription_pda);
    assert!(account_after.is_none() || account_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0);

    let sponsor_balance_after = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;
    assert!(sponsor_balance_after >= sponsor_balance_before + sub_rent - 10_000);
}

pub fn sponsor_revoke_subscription_when_cancelled_and_expired<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "A sponsor can revoke a cancelled, expired subscription",
        "after the subscriber cancels and the period ends, the sponsor recovers the rent",
    );
    let plan_end_ts = world.now() + days(30) as i64;
    let (alice, _merchant, sponsor, plan_pda, subscription_pda) = setup_sponsored_subscription(&mut world, plan_end_ts);

    // Subscriber cancels.
    world.md().step("Alice cancels the subscription");
    let ix = CancelSubscription::new(world.svm_mut(), &alice, plan_pda, subscription_pda).instruction();
    world.send_ok(&[ix], &[&alice], "CancelSubscription");

    // Wait for the cancellation period to end.
    world.warp(hours(2));

    let sub_rent = world.svm().get_account(&subscription_pda).unwrap().lamports;
    let sponsor_balance_before = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;

    world.md().step("The sponsor revokes the cancelled subscription");
    let ix = RevokeSubscription::new(world.svm_mut(), &sponsor, subscription_pda, plan_pda).instruction();
    world.send_ok(&[ix], &[&sponsor], "RevokeSubscription (cancelled and expired)");

    let sponsor_balance_after = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;
    assert!(sponsor_balance_after >= sponsor_balance_before + sub_rent - 10_000);
}

pub fn sponsor_revoke_active_subscription_rejected<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "A sponsor cannot revoke an active subscription",
        "while the plan is active and the subscription is not cancelled, the sponsor cannot revoke",
    );
    let plan_end_ts = world.now() + days(30) as i64;
    let (_alice, _merchant, sponsor, plan_pda, subscription_pda) = setup_sponsored_subscription(&mut world, plan_end_ts);

    // Plan still active, subscription not cancelled. Sponsor cannot revoke.
    world.md().step("The sponsor tries to revoke an active subscription");
    let ix = RevokeSubscription::new(world.svm_mut(), &sponsor, subscription_pda, plan_pda).instruction();
    world.send_err(&[ix], &[&sponsor], "RevokeSubscription (active)", SubscriptionsError::Unauthorized);
}

pub fn sponsor_revoke_subscription_with_wrong_plan_pda_rejected<B: TestSVM>(backend: B) {
    use crate::tests::utils::CreatePlan;

    let mut world = World::new(backend,
        "A sponsor cannot revoke with the wrong plan PDA",
        "pointing revoke at an unrelated plan is rejected as a subscription/plan mismatch",
    );
    let plan_end_ts = world.now() + hours(2) as i64;
    let (_alice, merchant, sponsor, _plan_pda, subscription_pda) = setup_sponsored_subscription(&mut world, plan_end_ts);

    // Create a second, unrelated plan.
    let mint = init_mint(
        world.svm_mut(),
        crate::tests::constants::TOKEN_PROGRAM_ID,
        crate::tests::constants::MINT_DECIMALS,
        1_000_000_000,
        None,
        &[],
    );
    let other_plan_end = world.now() + days(60) as i64;
    world.md().step("The merchant creates a second, unrelated plan");
    let other_plan_ix = CreatePlan::new(world.svm_mut(), &merchant, mint)
        .plan_id(99)
        .amount(1_000)
        .period_hours(24)
        .end_ts(other_plan_end)
        .instruction();
    let other_plan_pda = crate::tests::pda::get_plan_pda(&merchant.pubkey(), 99).0;
    world.prop(other_plan_pda, "Other plan");
    world.send_ok(&[other_plan_ix], &[&merchant], "CreatePlan (unrelated)");

    world.warp(hours(3));

    world.md().step("The sponsor revokes against the wrong plan PDA");
    let ix = RevokeSubscription::new(world.svm_mut(), &sponsor, subscription_pda, other_plan_pda).instruction();
    world.send_err(&[ix], &[&sponsor], "RevokeSubscription (wrong plan)", SubscriptionsError::SubscriptionPlanMismatch);
}

pub fn attacker_cannot_revoke_sponsor_funded_subscription<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Mallory cannot revoke a sponsor-funded subscription",
        "Mallory cannot revoke even after the plan expires; she is neither delegator nor payer",
    );
    let plan_end_ts = world.now() + hours(2) as i64;
    let (_alice, _merchant, _sponsor, plan_pda, subscription_pda) = setup_sponsored_subscription(&mut world, plan_end_ts);

    let mallory = world.actor("mallory");

    // Even after the plan expires, Mallory (neither delegator nor payer)
    // must not be able to revoke.
    world.warp(hours(3));

    world.md().step("Mallory tries to revoke the sponsor-funded subscription");
    let ix = RevokeSubscription::new(world.svm_mut(), &mallory, subscription_pda, plan_pda).instruction();
    world.send_err(&[ix], &[&mallory], "RevokeSubscription (by Mallory)", SubscriptionsError::Unauthorized);
}

pub fn subscriber_revoke_routes_rent_to_sponsor<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "A subscriber revoke routes rent to the sponsor",
        "when the subscriber revokes, the rent flows to the recorded payer (the sponsor)",
    );
    let plan_end_ts = world.now() + days(30) as i64;
    let (alice, _merchant, sponsor, plan_pda, subscription_pda) = setup_sponsored_subscription(&mut world, plan_end_ts);

    // Subscriber cancels and waits.
    world.md().step("Alice cancels the subscription");
    let ix = CancelSubscription::new(world.svm_mut(), &alice, plan_pda, subscription_pda).instruction();
    world.send_ok(&[ix], &[&alice], "CancelSubscription");
    world.warp(hours(2));

    let sub_rent = world.svm().get_account(&subscription_pda).unwrap().lamports;
    let sponsor_balance_before = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;

    // Subscriber revokes but receiver = sponsor (because header.payer = sponsor).
    world.md().step("Alice revokes, routing the rent back to the sponsor");
    let ix = RevokeSubscription::new(world.svm_mut(), &alice, subscription_pda, plan_pda).receiver(sponsor.pubkey()).instruction();
    world.send_ok(&[ix], &[&alice], "RevokeSubscription (rent to sponsor)");

    let sponsor_balance_after = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;
    assert!(sponsor_balance_after >= sponsor_balance_before + sub_rent - 10_000);
}
