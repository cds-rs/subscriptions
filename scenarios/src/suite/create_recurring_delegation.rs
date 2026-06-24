//! `create_recurring_delegation`, converted to the World/scenario pattern.
//!
//! Each test builds a `World`, draws Alice (the delegator/payer) from the cast,
//! stages her mint, ATA, and authority through the fabrication helpers, then
//! performs the create-recurring-delegation action through the observed `send_*`.
//! The delegatee is a prop (a bare `Pubkey`) unless it must actually pull, in
//! which case it is `bob` from the cast.

use solana_pubkey::Pubkey;
use solana_signer::Signer;

use testsvm::TestSVM;

use crate::{
    tests::utils::{as_pubkey,
            days, token_balance, CloseSubscriptionAuthority, CreateDelegation,
            ObservedResultExt, TransferDelegation, World,
        },
    AccountDiscriminator, RecurringDelegation, SubscriptionsError,
};

pub fn create_recurring_delegation<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Create a recurring delegation",
        "Alice grants a recurring pull delegation and its header and terms are recorded",
    );
    let alice = world.actor("alice");
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = 86400;
    let start_ts: i64 = world.now();
    let expiry_ts = start_ts + days(7) as i64;
    let nonce: u64 = 0;

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    world.md().step("Alice initializes her subscription authority");
    world.init_authority(&alice, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");

    world.md().step("Alice creates the recurring delegation");
    let (ix, delegation_pda) = CreateDelegation::new(world.svm_mut(), &alice, mint, delegatee)
        .nonce(nonce)
        .recurring_ix(amount_per_period, period_length_s, start_ts, expiry_ts);
    world.prop(delegation_pda, "RecurringDelegation");
    world.send_ok(&[ix], &[&alice], "CreateRecurringDelegation");

    let account = world.svm().get_account(&delegation_pda).unwrap();
    let delegation = RecurringDelegation::load(&account.data).unwrap();

    let header = delegation.header;
    let del_amount_per_period = delegation.amount_per_period;
    let del_period_length_s = delegation.period_length_s;
    let del_expiry_s = delegation.expiry_ts;
    let del_amount_pulled_in_period = delegation.amount_pulled_in_period;
    let del_current_period_start_ts = delegation.current_period_start_ts;

    world.md().check("the delegator is Alice", alice.pubkey(), as_pubkey(header.delegator.to_bytes()));
    world.md().check("the delegatee is recorded", delegatee, as_pubkey(header.delegatee.to_bytes()));
    world.md().check("the payer is Alice", alice.pubkey(), as_pubkey(header.payer.to_bytes()));
    world.md().check(
        "the account is tagged RecurringDelegation",
        AccountDiscriminator::RecurringDelegation as u8,
        header.discriminator,
    );
    world.md().check("the per-period amount is recorded", amount_per_period, del_amount_per_period);
    world.md().check("the period length is recorded", period_length_s, del_period_length_s);
    world.md().check("the expiry is recorded", expiry_ts, del_expiry_s);
    world.md().check("nothing has been pulled yet", 0, del_amount_pulled_in_period);
    world.md().check("the current period starts at the requested start", start_ts, del_current_period_start_ts);
}

pub fn create_recurring_delegation_rejects_stale_subscription_authority_generation<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Reject a stale subscription-authority generation",
        "a recurring delegation pinned to a closed-then-reinitialized authority's old init_id is rejected",
    );
    let alice = world.actor("alice");
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = 86400;
    let start_ts: i64 = world.now();
    let expiry_ts = start_ts + days(7) as i64;
    let nonce: u64 = 0;

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    world.md().step("Alice initializes her authority for the first time");
    let (res, subscription_authority_pda, _) = world.init_authority(&alice, mint, None);
    res.assert_ok();
    let old_init_id = crate::state::SubscriptionAuthority::load(
        &world.svm().get_account(&subscription_authority_pda).unwrap().data,
    )
    .unwrap()
    .init_id;

    world.md().step("Alice closes and reinitializes her authority, minting a fresh generation");
    let close_ix = CloseSubscriptionAuthority::new(world.svm_mut(), &alice, mint).instruction();
    world.send_ok(&[close_ix], &[&alice], "CloseSubscriptionAuthority");
    world.warp(1);
    world.init_authority(&alice, mint, None).0.assert_ok();

    let new_init_id = crate::state::SubscriptionAuthority::load(
        &world.svm().get_account(&subscription_authority_pda).unwrap().data,
    )
    .unwrap()
    .init_id;
    world.md().check("the generation changed", true, old_init_id != new_init_id);

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");

    world.md().step("Alice pins the delegation to the stale generation");
    let (ix, _) = CreateDelegation::new(world.svm_mut(), &alice, mint, delegatee)
        .expected_subscription_authority_init_id(old_init_id)
        .nonce(nonce)
        .recurring_ix(amount_per_period, period_length_s, start_ts, expiry_ts);
    world.send_err(
        &[ix],
        &[&alice],
        "CreateRecurringDelegation (stale authority)",
        SubscriptionsError::StaleSubscriptionAuthority,
    );
}

pub fn create_recurring_delegation_with_past_start_ts<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Reject a recurring delegation with a past start",
        "a start_ts in the past (i64::MIN sentinel of past) is rejected",
    );
    let alice = world.actor("alice");
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = 86400;
    let start_ts: i64 = i64::MIN;
    let expiry_ts = world.now() + 100000000;
    let nonce: u64 = 0;

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    world.md().step("Alice initializes her subscription authority");
    world.init_authority(&alice, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");

    world.md().step("Alice requests a start in the past");
    let (ix, _delegation_pda) = CreateDelegation::new(world.svm_mut(), &alice, mint, delegatee)
        .nonce(nonce)
        .recurring_ix(amount_per_period, period_length_s, start_ts, expiry_ts);
    world.send_err(
        &[ix],
        &[&alice],
        "CreateRecurringDelegation (past start)",
        SubscriptionsError::RecurringDelegationStartTimeInPast,
    );
}

pub fn create_recurring_delegation_with_zero_period<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Reject a recurring delegation with a zero period",
        "a period_length_s of zero is rejected",
    );
    let alice = world.actor("alice");
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = 0;
    let start_ts: i64 = world.now() + 10000;
    let expiry_ts = world.now() + 100000000;
    let nonce: u64 = 0;

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    world.md().step("Alice initializes her subscription authority");
    world.init_authority(&alice, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");

    world.md().step("Alice requests a zero-length period");
    let (ix, _delegation_pda) = CreateDelegation::new(world.svm_mut(), &alice, mint, delegatee)
        .nonce(nonce)
        .recurring_ix(amount_per_period, period_length_s, start_ts, expiry_ts);
    world.send_err(
        &[ix],
        &[&alice],
        "CreateRecurringDelegation (zero period)",
        SubscriptionsError::InvalidPeriodLength,
    );
}

pub fn create_recurring_delegation_with_start_ts_greater_than_expiry_ts<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Reject a recurring delegation whose start exceeds its expiry",
        "a start_ts greater than expiry_ts is rejected",
    );
    let alice = world.actor("alice");
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = 1;
    let start_ts: i64 = world.now() + 100000000;
    let expiry_ts = world.now() + 10000;
    let nonce: u64 = 0;

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    world.md().step("Alice initializes her subscription authority");
    world.init_authority(&alice, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");

    world.md().step("Alice requests a start after the expiry");
    let (ix, _delegation_pda) = CreateDelegation::new(world.svm_mut(), &alice, mint, delegatee)
        .nonce(nonce)
        .recurring_ix(amount_per_period, period_length_s, start_ts, expiry_ts);
    world.send_err(
        &[ix],
        &[&alice],
        "CreateRecurringDelegation (start past expiry)",
        SubscriptionsError::RecurringDelegationStartTimeGreaterThanExpiry,
    );
}

pub fn create_recurring_delegation_with_period_exceeding_max<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Reject a recurring delegation whose period exceeds the maximum",
        "a period_length_s beyond the one-year cap is rejected",
    );
    let alice = world.actor("alice");
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = 31_536_001;
    let start_ts: i64 = world.now();
    let expiry_ts = world.now() + days(365) as i64;
    let nonce: u64 = 0;

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    world.md().step("Alice initializes her subscription authority");
    world.init_authority(&alice, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");

    world.md().step("Alice requests a period beyond the maximum");
    let (ix, _delegation_pda) = CreateDelegation::new(world.svm_mut(), &alice, mint, delegatee)
        .nonce(nonce)
        .recurring_ix(amount_per_period, period_length_s, start_ts, expiry_ts);
    world.send_err(
        &[ix],
        &[&alice],
        "CreateRecurringDelegation (period exceeds max)",
        SubscriptionsError::InvalidPeriodLength,
    );
}

pub fn create_recurring_delegation_with_sentinel_start_starts_at_landing<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Sentinel start begins at landing",
        "a start_ts of 0 anchors the first period to the on-chain landing time, and the delegatee can pull",
    );
    let alice = world.actor("alice");
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = 86400;
    let start_ts: i64 = 0;
    let expiry_ts = world.now() + days(7) as i64;
    let nonce: u64 = 0;

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 100_000_000);

    world.md().step("Alice initializes her subscription authority");
    world.init_authority(&alice, mint, None).0.assert_ok();

    let bob = world.actor("bob");
    let delegatee_ata = world.fund_ata(mint, &bob, 0);
    world.prop(delegatee_ata, "delegatee ATA");

    world.md().step("Alice creates the delegation with a sentinel (landing) start");
    let (ix, delegation_pda) = CreateDelegation::new(world.svm_mut(), &alice, mint, bob.pubkey())
        .nonce(nonce)
        .recurring_ix(amount_per_period, period_length_s, start_ts, expiry_ts);
    world.prop(delegation_pda, "RecurringDelegation");
    world.send_ok(&[ix], &[&alice], "CreateRecurringDelegation (sentinel start)");

    let clock_ts = world.now();
    let account = world.svm().get_account(&delegation_pda).unwrap();
    let delegation = RecurringDelegation::load(&account.data).unwrap();
    let del_current_period_start_ts = delegation.current_period_start_ts;
    world.md().check("the period starts at the landing clock", clock_ts, del_current_period_start_ts);
    assert_ne!(del_current_period_start_ts, 0);

    let transfer_amount: u64 = 10_000_000;
    world.md().step("Bob pulls against the delegation");
    let transfer_ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(transfer_amount)
        .recurring_ix();
    world.send_ok(&[transfer_ix], &[&bob], "TransferRecurring");

    let bob_balance = token_balance(world.svm(), &delegatee_ata);
    world.md().check("the pulled amount landed in Bob's ATA", transfer_amount, bob_balance);
}

pub fn create_recurring_delegation_sentinel_start_requires_expiry<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Sentinel start requires an expiry",
        "a start_ts of 0 with a zero expiry is rejected",
    );
    let alice = world.actor("alice");
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = 86400;
    let start_ts: i64 = 0;
    let expiry_ts: i64 = 0;
    let nonce: u64 = 0;

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    world.md().step("Alice initializes her subscription authority");
    world.init_authority(&alice, mint, None).0.assert_ok();

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");

    world.md().step("Alice requests a sentinel start with no expiry");
    let (ix, _delegation_pda) = CreateDelegation::new(world.svm_mut(), &alice, mint, delegatee)
        .nonce(nonce)
        .recurring_ix(amount_per_period, period_length_s, start_ts, expiry_ts);
    world.send_err(
        &[ix],
        &[&alice],
        "CreateRecurringDelegation (sentinel start, no expiry)",
        SubscriptionsError::RecurringDelegationStartOnLandingRequiresExpiry,
    );
}

pub fn create_recurring_delegation_sentinel_start_rejects_elapsed_expiry<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Sentinel start rejects an already-elapsed expiry",
        "a start_ts of 0 with an expiry the landing clock has already passed is rejected",
    );
    let alice = world.actor("alice");
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = 86400;
    let start_ts: i64 = 0;
    let expiry_ts = world.now() + days(7) as i64;
    let nonce: u64 = 0;

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    world.md().step("Alice initializes her subscription authority");
    world.init_authority(&alice, mint, None).0.assert_ok();

    world.md().step("The clock advances past the chosen expiry");
    world.warp(days(8));

    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");

    world.md().step("Alice requests a sentinel start whose expiry has already elapsed");
    let (ix, _delegation_pda) = CreateDelegation::new(world.svm_mut(), &alice, mint, delegatee)
        .nonce(nonce)
        .recurring_ix(amount_per_period, period_length_s, start_ts, expiry_ts);
    world.send_err(
        &[ix],
        &[&alice],
        "CreateRecurringDelegation (sentinel start, elapsed expiry)",
        SubscriptionsError::RecurringDelegationStartTimeGreaterThanExpiry,
    );
}

pub fn create_recurring_delegation_with_zero_expiry<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "A zero expiry means no expiry",
        "a zero expiry_ts records an open-ended delegation that still pulls after a long warp",
    );
    let alice = world.actor("alice");
    let amount_per_period: u64 = 50_000_000;
    let period_length_s: u64 = 86400;
    let start_ts: i64 = world.now();
    let expiry_ts: i64 = 0;
    let nonce: u64 = 0;

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 100_000_000);

    world.md().step("Alice initializes her subscription authority");
    world.init_authority(&alice, mint, None).0.assert_ok();

    let bob = world.actor("bob");
    let delegatee_ata = world.fund_ata(mint, &bob, 0);
    world.prop(delegatee_ata, "delegatee ATA");

    world.md().step("Alice creates an open-ended (zero expiry) delegation");
    let (ix, delegation_pda) = CreateDelegation::new(world.svm_mut(), &alice, mint, bob.pubkey())
        .nonce(nonce)
        .recurring_ix(amount_per_period, period_length_s, start_ts, expiry_ts);
    world.prop(delegation_pda, "RecurringDelegation");
    world.send_ok(&[ix], &[&alice], "CreateRecurringDelegation (zero expiry)");

    let account = world.svm().get_account(&delegation_pda).unwrap();
    let delegation = RecurringDelegation::load(&account.data).unwrap();
    let del_expiry_ts = delegation.expiry_ts;
    world.md().check("the expiry is recorded as zero (open-ended)", 0, del_expiry_ts);

    world.md().step("The clock advances 30 days; the delegation is still pullable");
    world.warp(days(30));

    let transfer_amount: u64 = 10_000_000;
    let transfer_ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(transfer_amount)
        .recurring_ix();
    world.send_ok(&[transfer_ix], &[&bob], "TransferRecurring");

    let bob_balance = token_balance(world.svm(), &delegatee_ata);
    world.md().check("the pulled amount landed in Bob's ATA", transfer_amount, bob_balance);
}
