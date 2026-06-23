//! `cancel_subscription`, converted to the World/scenario pattern.
//!
//! Each test builds a `World`, stages a live subscription (or builds one step by
//! step from the cast), and performs the cancel through the observed `send_*`.
//! Every send renders its surface into the test's report under
//! `target/md-reports/`.

use solana_signer::Signer;

use litesvm_utils::TestSVM;

use crate::{
    instructions::create_plan::PlanTerms,
    state::{plan::Plan, subscription_delegation::SubscriptionDelegation},
    tests::utils::{
            days, token_balance, hours, minutes, CancelSubscription, CreatePlan,
            CreateSubscription, DeletePlan, ObservedResultExt, RevokeSubscription, TransferSubscription, UpdatePlan,
            make_backend, World,
        },
    SubscriptionsError,
};

#[test]
fn cancel_subscription_happy_path() {
    let mut world = World::new(make_backend(), 
        "Cancel a subscription (happy path)",
        "Alice cancels her live subscription; it gets an end-of-period expiry",
    );
    let s = world.stage_subscription();

    world.md().step("Alice cancels her subscription");
    let cancel_ix = CancelSubscription::new(world.svm_mut(), &s.alice, s.plan_pda, s.subscription_pda).instruction();
    world.send_ok(&[cancel_ix], &[&s.alice], "CancelSubscription");

    // Verify expires_at_ts is set (end of current period)
    let sub_account = world.svm().get_account(&s.subscription_pda).unwrap();
    let sub = SubscriptionDelegation::load(&sub_account.data).unwrap();
    world.md().check("expires_at_ts is set (non-zero)", true, { sub.expires_at_ts } != 0);
}

#[test]
fn cancel_at_exact_end_ts_keeps_final_period_billable() {
    let mut world = World::new(make_backend(), 
        "Cancel at exact plan end keeps the final period billable",
        "cancelling exactly at end_ts still allows the merchant to pull the final period",
    );
    let subscriber = world.actor("alice");
    let merchant = world.actor("merchant");

    let mint =
        world.usdc_mint(&subscriber);
    world.prop(mint, "USDC mint");
    world.fund_ata(mint, &subscriber, 100_000_000);
    let merchant_ata = world.fund_ata(mint, &merchant, 0);
    world.prop(merchant_ata, "merchant ATA");

    world.md().step("Stage: Alice's authority and the merchant's 2-hour plan");
    world.init_authority(&subscriber, mint, None).0.assert_ok();

    let start_ts = world.now();
    let end_ts = start_ts + hours(2) as i64;

    let (plan_ix, plan_pda) = {
        let builder =
            CreatePlan::new(world.svm_mut(), &merchant, mint).plan_id(1).amount(50_000_000).period_hours(1).end_ts(end_ts);
        (builder.instruction(), builder.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&merchant], "CreatePlan");

    let plan_account = world.svm().get_account(&plan_pda).unwrap();
    let terms = Plan::load(&plan_account.data).unwrap().data.terms;
    let subscription_pda = CreateSubscription::new(world.svm_mut(), plan_pda, subscriber.pubkey(), mint, start_ts)
        .terms(terms)
        .execute();
    world.prop(subscription_pda, "Subscription");

    let now = world.now();
    world.warp(u64::try_from(end_ts - now).unwrap());
    let clock_now = world.now();
    world.md().check("clock is exactly at plan end_ts", end_ts, clock_now);

    world.md().step("Alice cancels exactly at end_ts");
    let cancel_ix = CancelSubscription::new(world.svm_mut(), &subscriber, plan_pda, subscription_pda).instruction();
    world.send_ok(&[cancel_ix], &[&subscriber], "CancelSubscription");

    let sub_account = world.svm().get_account(&subscription_pda).unwrap();
    let sub = SubscriptionDelegation::load(&sub_account.data).unwrap();
    world.md().check("expires_at_ts is just past the inclusive end_ts", end_ts + 1, sub.expires_at_ts);

    world.md().step("Revoke is refused: the subscription is still within its valid period");
    let revoke_ix = RevokeSubscription::new(world.svm_mut(), &subscriber, subscription_pda, plan_pda).instruction();
    world.send_err(
        &[revoke_ix],
        &[&subscriber],
        "RevokeSubscription (still billable)",
        SubscriptionsError::SubscriptionNotCancelled,
    );

    world.md().step("The merchant pulls the final billable period");
    let transfer_ix = TransferSubscription::new(
        world.svm_mut(),
        &merchant,
        subscriber.pubkey(),
        mint,
        subscription_pda,
        plan_pda,
    )
    .amount(20_000_000)
    .to(merchant_ata)
    .instruction();
    world.send_ok(&[transfer_ix], &[&merchant], "TransferSubscription");
    let merchant_balance = token_balance(world.svm(), &merchant_ata);
    world.md().check("the merchant received the final period's tokens", 20_000_000, merchant_balance);
}

#[test]
fn cancel_subscription_non_subscriber_rejected() {
    let mut world = World::new(make_backend(), 
        "Cancel rejects a non-subscriber",
        "Mallory cannot cancel a subscription she does not own",
    );
    let s = world.stage_subscription();
    let mallory = world.actor("mallory");

    world.md().step("Mallory attempts to cancel Alice's subscription");
    let cancel_ix = CancelSubscription::new(world.svm_mut(), &mallory, s.plan_pda, s.subscription_pda).instruction();
    world.send_err(
        &[cancel_ix],
        &[&mallory],
        "CancelSubscription (non-subscriber)",
        SubscriptionsError::Unauthorized,
    );
}

#[test]
fn cancel_subscription_already_cancelled_rejected() {
    let mut world = World::new(make_backend(), 
        "Cancel rejects an already-cancelled subscription",
        "cancelling twice is rejected",
    );
    let s = world.stage_subscription();

    // Cancel once
    world.md().step("Alice cancels her subscription");
    let cancel_ix = CancelSubscription::new(world.svm_mut(), &s.alice, s.plan_pda, s.subscription_pda).instruction();
    world.send_ok(&[cancel_ix], &[&s.alice], "CancelSubscription");

    // Cancel again should fail
    world.md().step("Alice cancels again; it is rejected");
    let cancel_again_ix =
        CancelSubscription::new(world.svm_mut(), &s.alice, s.plan_pda, s.subscription_pda).instruction();
    world.send_err(
        &[cancel_again_ix],
        &[&s.alice],
        "CancelSubscription (already cancelled)",
        SubscriptionsError::SubscriptionAlreadyCancelled,
    );
}

#[test]
fn test_cancel_subscription_version_mismatch() {
    use crate::state::header::VERSION_OFFSET;

    let mut world = World::new(make_backend(), 
        "Cancel rejects a stale account version",
        "a downgraded version byte forces a MigrationRequired error",
    );
    let s = world.stage_subscription();

    world.md().step("Downgrade the subscription's version byte");
    let mut account = world.svm().get_account(&s.subscription_pda).unwrap();
    account.data[VERSION_OFFSET] = 0;
    world.svm_mut().set_account(&s.subscription_pda, account);

    world.md().step("Alice cancels; the stale version is refused");
    let cancel_ix = CancelSubscription::new(world.svm_mut(), &s.alice, s.plan_pda, s.subscription_pda).instruction();
    world.send_err(
        &[cancel_ix],
        &[&s.alice],
        "CancelSubscription (version mismatch)",
        SubscriptionsError::MigrationRequired,
    );
}

#[test]
fn cancel_subscription_ghost_plan_expires_immediately() {
    use crate::state::common::PlanStatus;

    let mut world = World::new(make_backend(), 
        "Cancel on a ghost plan expires immediately",
        "a recreated plan (same id, new terms) gives a cancelled subscription no grace period",
    );
    let s = world.stage_subscription();

    // Get current time before any clock manipulation
    let ts_before = world.now();

    // Sunset, expire, and delete the plan
    world.md().step("The merchant sunsets the plan");
    let end_ts = world.now() + days(2) as i64;
    let sunset_ix = UpdatePlan::new(world.svm_mut(), &s.merchant, s.plan_pda)
        .status(PlanStatus::Sunset)
        .end_ts(end_ts)
        .instruction();
    world.send_ok(&[sunset_ix], &[&s.merchant], "UpdatePlan (sunset)");

    world.warp(days(3));

    world.md().step("The merchant deletes the expired plan");
    let delete_ix = DeletePlan::new(world.svm_mut(), &s.merchant, s.plan_pda).instruction();
    world.send_ok(&[delete_ix], &[&s.merchant], "DeletePlan");

    // Recreate plan with same plan_id but different terms
    world.md().step("The merchant recreates a plan with the same id but new terms (a ghost plan)");
    let new_end_ts = world.now() + days(60) as i64;
    let (recreate_ix, new_plan_pda) = {
        let builder = CreatePlan::new(world.svm_mut(), &s.merchant, s.mint)
            .plan_id(1)
            .amount(999_000_000)
            .period_hours(720)
            .end_ts(new_end_ts);
        (builder.instruction(), builder.plan_pda())
    };
    world.send_ok(&[recreate_ix], &[&s.merchant], "CreatePlan (ghost)");
    world.md().check("the recreated plan resolves to the same PDA", s.plan_pda, new_plan_pda);

    // Cancel should succeed but expire immediately (no grace period)
    world.md().step("Alice cancels against the ghost plan");
    let cancel_ix = CancelSubscription::new(world.svm_mut(), &s.alice, s.plan_pda, s.subscription_pda).instruction();
    world.send_ok(&[cancel_ix], &[&s.alice], "CancelSubscription (ghost)");

    let sub_account = world.svm().get_account(&s.subscription_pda).unwrap();
    let sub = SubscriptionDelegation::load(&sub_account.data).unwrap();
    let expires = sub.expires_at_ts;
    // Should be immediate (current_ts), not end-of-period
    assert!(expires > ts_before);
    // Verify it's NOT a grace period (which would be period_start + period_length)
    // Ghost plan expires at current_ts, which is much less than period_start + 720h
    let svm_ts = world.now();
    world.md().check("expires immediately at the current clock (no grace period)", svm_ts, expires);
}

#[test]
fn cancel_subscription_caps_at_plan_end_ts() {
    let mut world = World::new(make_backend(), 
        "Cancel caps expiry at the plan end_ts",
        "expiry is capped just past the inclusive plan end_ts, not the period end",
    );
    let alice = world.actor("alice");
    let merchant = world.actor("merchant");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    world.fund_ata(mint, &alice, 100_000_000);

    world.md().step("Stage: Alice's authority and a plan ending in 90 minutes");
    world.init_authority(&alice, mint, None).0.assert_ok();

    let end_ts = world.now() + minutes(90) as i64;
    let (plan_ix, plan_pda) = {
        let builder =
            CreatePlan::new(world.svm_mut(), &merchant, mint).plan_id(1).amount(50_000_000).period_hours(1).end_ts(end_ts);
        (builder.instruction(), builder.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&merchant], "CreatePlan");

    let svm_ts = world.now();
    let subscription_pda = CreateSubscription::new(world.svm_mut(), plan_pda, alice.pubkey(), mint, svm_ts)
        .terms(PlanTerms { amount: 50_000_000, period_hours: 1, created_at: svm_ts })
        .execute();
    world.prop(subscription_pda, "Subscription");

    world.warp(hours(1) + minutes(5));

    world.md().step("Alice cancels after the plan end has passed within the period");
    let cancel_ix = CancelSubscription::new(world.svm_mut(), &alice, plan_pda, subscription_pda).instruction();
    world.send_ok(&[cancel_ix], &[&alice], "CancelSubscription");

    let sub_account = world.svm().get_account(&subscription_pda).unwrap();
    let sub = SubscriptionDelegation::load(&sub_account.data).unwrap();
    world.md().check(
        "expires_at_ts is capped just past the inclusive plan end_ts, not period end",
        end_ts + 1,
        sub.expires_at_ts,
    );
}

#[test]
fn cancel_subscription_after_plan_expired_allows_immediate_revoke() {
    let mut world = World::new(make_backend(), 
        "Cancel after plan expiry allows an immediate revoke",
        "cancelling a subscription whose plan already expired sets expiry at-or-before now",
    );
    let alice = world.actor("alice");
    let merchant = world.actor("merchant");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    world.fund_ata(mint, &alice, 100_000_000);

    world.md().step("Stage: Alice's authority and a plan ending in 2 hours");
    world.init_authority(&alice, mint, None).0.assert_ok();

    let end_ts = world.now() + hours(2) as i64;
    let (plan_ix, plan_pda) = {
        let builder =
            CreatePlan::new(world.svm_mut(), &merchant, mint).plan_id(1).amount(50_000_000).period_hours(1).end_ts(end_ts);
        (builder.instruction(), builder.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&merchant], "CreatePlan");

    let svm_ts = world.now();
    let subscription_pda = CreateSubscription::new(world.svm_mut(), plan_pda, alice.pubkey(), mint, svm_ts)
        .terms(PlanTerms { amount: 50_000_000, period_hours: 1, created_at: svm_ts })
        .execute();
    world.prop(subscription_pda, "Subscription");

    world.warp(hours(3));

    world.md().step("Alice cancels after the plan has fully expired");
    let cancel_ix = CancelSubscription::new(world.svm_mut(), &alice, plan_pda, subscription_pda).instruction();
    world.send_ok(&[cancel_ix], &[&alice], "CancelSubscription");

    let sub_account = world.svm().get_account(&subscription_pda).unwrap();
    let sub = SubscriptionDelegation::load(&sub_account.data).unwrap();
    let current_clock = world.now();
    assert!(
        { sub.expires_at_ts } <= current_clock,
        "expires_at_ts ({}) should be <= current time ({}) so subscriber can revoke immediately",
        { sub.expires_at_ts },
        current_clock
    );
    world.md().check("expiry is at or before now (immediate revoke is allowed)", true, { sub.expires_at_ts } <= current_clock);
}
