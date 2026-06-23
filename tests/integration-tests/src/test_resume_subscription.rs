//! `resume_subscription`, converted to the World/scenario pattern.
//!
//! Each test builds a `World`, stages a cancelled subscription (or builds one
//! step by step from the cast), and performs the resume through the observed
//! `send_*`. Every send renders its surface into the test's report under
//! `target/md-reports/`.

use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;

use litesvm_utils::TestSVM;

use crate::{
    state::{common::PlanStatus, header::VERSION_OFFSET, subscription_delegation::SubscriptionDelegation},
    tests::{
        pda::get_subscription_pda,
        utils::{
            days, hours, init_ata, CancelSubscription, CreatePlan, DeletePlan, ObservedResultExt,
            ResumeSubscription, Subscribe, TransferSubscription, UpdatePlan, World,
        },
    },
    SubscriptionsError,
};

/// Stages a subscription whose plan ends exactly one period after creation, so
/// the cancellation `expires_at_ts` is pinned to `plan.end_ts` and the
/// `PlanExpired`/`PlanClosed` guards in resume become reachable. Builds the
/// world's state through the observed verbs and returns the owned cast plus the
/// derived PDAs.
fn setup_subscription_with_tight_plan_end(world: &mut World) -> (Keypair, Keypair, Pubkey, Pubkey) {
    let alice = world.actor("alice");
    let merchant = world.actor("merchant");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    world.fund_ata(mint, &alice, 100_000_000);

    world.md().step("Stage: Alice's authority, the merchant's tight 1-hour plan, Alice subscribed");
    world.init_authority(&alice, mint, None).0.assert_ok();

    let end_ts = world.now() + hours(1) as i64;
    let (plan_ix, plan_pda) = {
        let builder =
            CreatePlan::new(world.svm_mut(), &merchant, mint).plan_id(1).amount(50_000_000).period_hours(1).end_ts(end_ts);
        (builder.instruction(), builder.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&merchant], "CreatePlan");

    let plan_bump = crate::tests::pda::get_plan_pda(&merchant.pubkey(), 1).1;
    let sub_ix = Subscribe::new(world.svm_mut(), &alice, merchant.pubkey(), plan_pda, 1, plan_bump, mint).instruction();
    let (subscription_pda, _) = get_subscription_pda(&plan_pda, &alice.pubkey());
    world.prop(subscription_pda, "Subscription");
    world.send_ok(&[sub_ix], &[&alice], "Subscribe");

    (alice, merchant, plan_pda, subscription_pda)
}

#[test]
fn resume_subscription_happy_path() {
    let mut world = World::new(
        "Resume a subscription (happy path)",
        "Alice cancels then resumes; the expiry is cleared and the period state is preserved",
    );
    let s = world.stage_subscription();

    world.md().step("Alice cancels her subscription");
    let cancel_ix = CancelSubscription::new(world.svm_mut(), &s.alice, s.plan_pda, s.subscription_pda).instruction();
    world.send_ok(&[cancel_ix], &[&s.alice], "CancelSubscription");

    let sub_account = world.svm().get_account(&s.subscription_pda).unwrap();
    let sub = SubscriptionDelegation::load(&sub_account.data).unwrap();
    let period_start = sub.current_period_start_ts;
    let amount_pulled = sub.amount_pulled_in_period;
    world.md().check("the cancelled subscription has a non-zero expiry", true, { sub.expires_at_ts } != 0);

    world.md().step("Alice resumes her subscription");
    let resume_ix = ResumeSubscription::new(world.svm_mut(), &s.alice, s.plan_pda, s.subscription_pda).instruction();
    world.send_ok(&[resume_ix], &[&s.alice], "ResumeSubscription");

    let sub_account = world.svm().get_account(&s.subscription_pda).unwrap();
    let sub = SubscriptionDelegation::load(&sub_account.data).unwrap();
    world.md().check("the expiry is cleared", 0, sub.expires_at_ts);
    world.md().check("the period start is preserved", period_start, sub.current_period_start_ts);
    world.md().check("the amount pulled is preserved", amount_pulled, sub.amount_pulled_in_period);
}

#[test]
fn resume_subscription_rejected_at_cancelled_period_end() {
    let mut world = World::new(
        "Resume rejected at the cancelled period end",
        "once the cancelled period has elapsed, both a pull and a resume are refused as cancelled",
    );
    let s = world.stage_subscription();
    init_ata(world.svm_mut(), s.mint, s.merchant.pubkey(), 0);

    world.md().step("Alice cancels her subscription");
    let cancel_ix = CancelSubscription::new(world.svm_mut(), &s.alice, s.plan_pda, s.subscription_pda).instruction();
    world.send_ok(&[cancel_ix], &[&s.alice], "CancelSubscription");
    world.warp(hours(1));

    world.md().step("The merchant's pull is refused: the cancelled period has ended");
    let transfer_ix = TransferSubscription::new(
        world.svm_mut(),
        &s.merchant,
        s.alice.pubkey(),
        s.mint,
        s.subscription_pda,
        s.plan_pda,
    )
    .amount(10_000_000)
    .instruction();
    world.send_err(
        &[transfer_ix],
        &[&s.merchant],
        "TransferSubscription (after cancelled period)",
        SubscriptionsError::SubscriptionCancelled,
    );

    world.md().step("Alice's resume is refused for the same reason");
    let resume_ix = ResumeSubscription::new(world.svm_mut(), &s.alice, s.plan_pda, s.subscription_pda).instruction();
    world.send_err(
        &[resume_ix],
        &[&s.alice],
        "ResumeSubscription (after cancelled period)",
        SubscriptionsError::SubscriptionCancelled,
    );
}

#[test]
fn resume_subscription_not_cancelled_rejected() {
    let mut world = World::new(
        "Resume rejects a live subscription",
        "resuming a subscription that was never cancelled is refused",
    );
    let s = world.stage_subscription();

    world.md().step("Alice resumes a subscription she never cancelled");
    let resume_ix = ResumeSubscription::new(world.svm_mut(), &s.alice, s.plan_pda, s.subscription_pda).instruction();
    world.send_err(
        &[resume_ix],
        &[&s.alice],
        "ResumeSubscription (not cancelled)",
        SubscriptionsError::SubscriptionNotCancelled,
    );
}

#[test]
fn resume_subscription_non_subscriber_rejected() {
    let mut world = World::new(
        "Resume rejects a non-subscriber",
        "Mallory cannot resume a subscription she does not own",
    );
    let s = world.stage_subscription();

    world.md().step("Alice cancels her subscription");
    let cancel_ix = CancelSubscription::new(world.svm_mut(), &s.alice, s.plan_pda, s.subscription_pda).instruction();
    world.send_ok(&[cancel_ix], &[&s.alice], "CancelSubscription");

    let mallory = world.actor("mallory");
    world.md().step("Mallory attempts to resume Alice's subscription");
    let resume_ix = ResumeSubscription::new(world.svm_mut(), &mallory, s.plan_pda, s.subscription_pda).instruction();
    world.send_err(
        &[resume_ix],
        &[&mallory],
        "ResumeSubscription (non-subscriber)",
        SubscriptionsError::Unauthorized,
    );
}

#[test]
fn resume_subscription_plan_mismatch_rejected() {
    let mut world = World::new(
        "Resume rejects a mismatched plan",
        "resuming against a different plan than the subscription's is refused",
    );
    let s = world.stage_subscription();

    world.md().step("Alice cancels her subscription");
    let cancel_ix = CancelSubscription::new(world.svm_mut(), &s.alice, s.plan_pda, s.subscription_pda).instruction();
    world.send_ok(&[cancel_ix], &[&s.alice], "CancelSubscription");

    world.md().step("The merchant creates a second, unrelated plan");
    let end_ts = world.now() + days(30) as i64;
    let (wrong_plan_ix, wrong_plan) = {
        let builder =
            CreatePlan::new(world.svm_mut(), &s.merchant, s.mint).plan_id(2).amount(50_000_000).period_hours(1).end_ts(end_ts);
        (builder.instruction(), builder.plan_pda())
    };
    world.prop(wrong_plan, "Wrong Plan");
    world.send_ok(&[wrong_plan_ix], &[&s.merchant], "CreatePlan (second plan)");

    world.md().step("Alice resumes against the wrong plan");
    let resume_ix = ResumeSubscription::new(world.svm_mut(), &s.alice, wrong_plan, s.subscription_pda).instruction();
    world.send_err(
        &[resume_ix],
        &[&s.alice],
        "ResumeSubscription (plan mismatch)",
        SubscriptionsError::SubscriptionPlanMismatch,
    );
}

#[test]
fn resume_subscription_rejected_after_cancelled_period_elapsed() {
    let mut world = World::new(
        "Resume rejected after the cancelled period elapsed",
        "once past the cancelled period boundary, resume is refused as cancelled",
    );
    let s = world.stage_subscription();

    world.md().step("Alice cancels her subscription");
    let cancel_ix = CancelSubscription::new(world.svm_mut(), &s.alice, s.plan_pda, s.subscription_pda).instruction();
    world.send_ok(&[cancel_ix], &[&s.alice], "CancelSubscription");
    world.warp(hours(1) + 1);

    world.md().step("Alice resumes one second past the cancelled period end");
    let resume_ix = ResumeSubscription::new(world.svm_mut(), &s.alice, s.plan_pda, s.subscription_pda).instruction();
    world.send_err(
        &[resume_ix],
        &[&s.alice],
        "ResumeSubscription (past cancelled period)",
        SubscriptionsError::SubscriptionCancelled,
    );
}

#[test]
fn resume_subscription_rejected_when_plan_expired() {
    let mut world = World::new(
        "Resume rejected when the plan has expired",
        "resume is refused because the plan no longer supports active subscriptions",
    );
    let (alice, _merchant, plan_pda, subscription_pda) = setup_subscription_with_tight_plan_end(&mut world);

    world.md().step("Alice cancels her subscription");
    let cancel_ix = CancelSubscription::new(world.svm_mut(), &alice, plan_pda, subscription_pda).instruction();
    world.send_ok(&[cancel_ix], &[&alice], "CancelSubscription");

    // Advance just past plan.end_ts. Resume is rejected because the plan no
    // longer supports active subscriptions.
    world.warp(hours(1) + 1);

    world.md().step("Alice resumes after the plan has expired");
    let resume_ix = ResumeSubscription::new(world.svm_mut(), &alice, plan_pda, subscription_pda).instruction();
    world.send_err(
        &[resume_ix],
        &[&alice],
        "ResumeSubscription (plan expired)",
        SubscriptionsError::PlanExpired,
    );
}

#[test]
fn resume_subscription_allows_when_plan_sunset() {
    let mut world = World::new(
        "Resume allowed while the plan is sunset",
        "a sunset (but not yet expired) plan still admits a resume",
    );
    let s = world.stage_subscription();

    world.md().step("Alice cancels her subscription");
    let cancel_ix = CancelSubscription::new(world.svm_mut(), &s.alice, s.plan_pda, s.subscription_pda).instruction();
    world.send_ok(&[cancel_ix], &[&s.alice], "CancelSubscription");

    world.md().step("The merchant sunsets the plan with a 7-day end");
    let sunset_end = world.now() + days(7) as i64;
    let sunset_ix = UpdatePlan::new(world.svm_mut(), &s.merchant, s.plan_pda)
        .status(PlanStatus::Sunset)
        .end_ts(sunset_end)
        .instruction();
    world.send_ok(&[sunset_ix], &[&s.merchant], "UpdatePlan (sunset)");

    world.md().step("Alice resumes while the plan is sunset");
    let resume_ix = ResumeSubscription::new(world.svm_mut(), &s.alice, s.plan_pda, s.subscription_pda).instruction();
    world.send_ok(&[resume_ix], &[&s.alice], "ResumeSubscription");

    let account = world.svm().get_account(&s.subscription_pda).unwrap();
    let sub = SubscriptionDelegation::load(&account.data).unwrap();
    world.md().check("the expiry is cleared", 0, sub.expires_at_ts);
}

#[test]
fn resume_subscription_rejected_when_plan_deleted() {
    let mut world = World::new(
        "Resume rejected when the plan was deleted",
        "resuming against a closed (deleted) plan is refused",
    );
    let (alice, merchant, plan_pda, subscription_pda) = setup_subscription_with_tight_plan_end(&mut world);

    world.md().step("Alice cancels her subscription");
    let cancel_ix = CancelSubscription::new(world.svm_mut(), &alice, plan_pda, subscription_pda).instruction();
    world.send_ok(&[cancel_ix], &[&alice], "CancelSubscription");
    world.warp(hours(1) + 1);

    world.md().step("The merchant deletes the expired plan");
    let delete_ix = DeletePlan::new(world.svm_mut(), &merchant, plan_pda).instruction();
    world.send_ok(&[delete_ix], &[&merchant], "DeletePlan");

    world.md().step("Alice resumes against the deleted plan");
    let resume_ix = ResumeSubscription::new(world.svm_mut(), &alice, plan_pda, subscription_pda).instruction();
    world.send_err(
        &[resume_ix],
        &[&alice],
        "ResumeSubscription (plan closed)",
        SubscriptionsError::PlanClosed,
    );
}

#[test]
fn resume_subscription_cancel_resume_cancel_across_period_boundary() {
    let mut world = World::new(
        "Cancel, resume, then cancel across a period boundary",
        "the second cancel computes a fresh period boundary rather than reusing the stale one",
    );
    let s = world.stage_subscription();

    world.md().step("Alice cancels her subscription");
    let cancel_ix = CancelSubscription::new(world.svm_mut(), &s.alice, s.plan_pda, s.subscription_pda).instruction();
    world.send_ok(&[cancel_ix], &[&s.alice], "CancelSubscription (first)");
    let first_expires_at = {
        let account = world.svm().get_account(&s.subscription_pda).unwrap();
        let sub = SubscriptionDelegation::load(&account.data).unwrap();
        sub.expires_at_ts
    };

    world.md().step("Alice resumes her subscription");
    let resume_ix = ResumeSubscription::new(world.svm_mut(), &s.alice, s.plan_pda, s.subscription_pda).instruction();
    world.send_ok(&[resume_ix], &[&s.alice], "ResumeSubscription");

    // Advance past the original period end so the second cancel must compute a
    // new period boundary, not reuse the stale one.
    world.warp(hours(2));

    world.md().step("Alice cancels again past the original period boundary");
    let cancel_again_ix =
        CancelSubscription::new(world.svm_mut(), &s.alice, s.plan_pda, s.subscription_pda).instruction();
    world.send_ok(&[cancel_again_ix], &[&s.alice], "CancelSubscription (second)");
    let account = world.svm().get_account(&s.subscription_pda).unwrap();
    let sub = SubscriptionDelegation::load(&account.data).unwrap();
    assert!(
        { sub.expires_at_ts } > first_expires_at,
        "second cancel should advance expires_at_ts past the prior period boundary",
    );
    world.md().check(
        "the second cancel advances the expiry past the prior boundary",
        true,
        { sub.expires_at_ts } > first_expires_at,
    );
}

#[test]
fn resume_subscription_version_mismatch() {
    let mut world = World::new(
        "Resume rejects a stale account version",
        "a downgraded version byte forces a MigrationRequired error on resume",
    );
    let s = world.stage_subscription();

    world.md().step("Alice cancels her subscription");
    let cancel_ix = CancelSubscription::new(world.svm_mut(), &s.alice, s.plan_pda, s.subscription_pda).instruction();
    world.send_ok(&[cancel_ix], &[&s.alice], "CancelSubscription");

    world.md().step("Downgrade the subscription's version byte");
    let mut account = world.svm().get_account(&s.subscription_pda).unwrap();
    account.data[VERSION_OFFSET] = 0;
    world.svm_mut().set_account(&s.subscription_pda, account);

    world.md().step("Alice resumes; the stale version is refused");
    let resume_ix = ResumeSubscription::new(world.svm_mut(), &s.alice, s.plan_pda, s.subscription_pda).instruction();
    world.send_err(
        &[resume_ix],
        &[&s.alice],
        "ResumeSubscription (version mismatch)",
        SubscriptionsError::MigrationRequired,
    );
}
