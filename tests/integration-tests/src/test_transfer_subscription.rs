//! `transfer_subscription`, converted to the World/scenario pattern.
//!
//! A merchant (or an authorized puller) pulls funds against a subscriber's
//! standing subscription. Each test builds its own `World`, stages a plan and a
//! subscription via the local `setup_plan_and_subscription` helper (rewritten to
//! drive the world's state through the observed backend), then performs the pull
//! through `TransferSubscription` routed through `world.send_*`. Every send
//! renders its surface into the test's report under `target/md-reports/`.

use std::vec::Vec;

use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use spl_associated_token_account_interface::address::get_associated_token_address_with_program_id;

use crate::{
    event_engine::event_authority_pda,
    state::{plan::Plan, subscription_delegation::SubscriptionDelegation},
    tests::{
        constants::{PROGRAM_ID, TOKEN_PROGRAM_ID},
        pda::{get_plan_pda, get_subscription_authority_pda},
        utils::{
            days, get_ata_balance, hours, init_ata, CancelSubscription, CreatePlan, CreateSubscription,
            DeletePlan, ObservedResultExt, TransferSubscription, UpdatePlan, World,
        },
    },
    SubscriptionsError,
};

/// Stage a plan (owned by the merchant) and a live subscription for Alice,
/// driving every on-chain action through the observed backend. Mirrors the file's
/// original `setup_plan_and_subscription`, but takes a `&mut World` and returns
/// owned values (minus the LiteSVM the world now owns). The subscription is
/// injected directly with the world's clock as the period start.
#[allow(clippy::type_complexity)]
fn setup_plan_and_subscription(
    world: &mut World,
    amount_per_period: u64,
    period_hours: u64,
    end_ts: i64,
    destinations: Vec<Pubkey>,
    pullers: Vec<Pubkey>,
) -> (
    Keypair, // alice (subscriber)
    Keypair, // merchant (plan owner)
    Pubkey,  // mint
    Pubkey,  // plan_pda
    u8,      // plan_bump
    Pubkey,  // subscription_pda
    Pubkey,  // alice_ata
    Pubkey,  // merchant_ata
) {
    let alice = world.actor("alice");
    let merchant = world.actor("merchant");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let alice_ata = world.fund_ata(mint, &alice, 100_000_000);
    let merchant_ata = world.fund_ata(mint, &merchant, 0);

    // Initialize subscription_authority for alice.
    world.md().step("Stage: Alice's authority, the merchant's plan, Alice subscribed");
    world.init_authority(&alice, mint, None).0.assert_ok();

    // Create plan.
    let plan_ix = CreatePlan::new(world.svm_mut(), &merchant, mint)
        .plan_id(1)
        .amount(amount_per_period)
        .period_hours(period_hours)
        .end_ts(end_ts)
        .destinations(destinations)
        .pullers(pullers)
        .instruction();
    let (plan_pda, plan_bump) = get_plan_pda(&merchant.pubkey(), 1);
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&merchant], "CreatePlan");

    // Manually inject subscription delegation (use the world clock as period start).
    let svm_ts = world.now();
    let plan_account = world.svm().get_account(&plan_pda).unwrap();
    let plan = Plan::load(&plan_account.data).unwrap();
    let plan_terms = plan.data.terms;
    let subscription_pda =
        CreateSubscription::new(world.svm_mut(), plan_pda, alice.pubkey(), mint, svm_ts).terms(plan_terms).execute();
    world.prop(subscription_pda, "Subscription");

    (alice, merchant, mint, plan_pda, plan_bump, subscription_pda, alice_ata, merchant_ata)
}

#[test]
fn test_transfer_subscription_success() {
    let mut world = World::new(
        "Transfer subscription: a merchant pulls",
        "the merchant pulls funds against Alice's standing subscription",
    );

    let amount_per_period = 50_000_000u64;
    let period_hours = 1u64;
    let end_ts = world.now() + days(30) as i64;

    let (alice, merchant, mint, plan_pda, _, subscription_pda, _, merchant_ata) =
        setup_plan_and_subscription(&mut world, amount_per_period, period_hours, end_ts, vec![], vec![]);

    let bal = get_ata_balance(world.svm(), &merchant_ata);
    world.md().check("the merchant ATA starts empty", 0, bal);

    let transfer_amount = 10_000_000u64;
    world.md().step("The merchant pulls 10 tokens");
    let ix = TransferSubscription::new(world.svm_mut(), &merchant, alice.pubkey(), mint, subscription_pda, plan_pda)
        .amount(transfer_amount)
        .instruction();
    world.send_ok(&[ix], &[&merchant], "TransferSubscription");

    let bal = get_ata_balance(world.svm(), &merchant_ata);
    world.md().check("the merchant received 10 tokens", 10_000_000, bal);

    // Verify subscription state was updated.
    let sub_account = world.svm().get_account(&subscription_pda).unwrap();
    let sub = SubscriptionDelegation::load(&sub_account.data).unwrap();
    world.md().check("the pulled-in-period is 10 tokens", 10_000_000, sub.amount_pulled_in_period);
}

#[test]
fn test_transfer_subscription_puller_authorized() {
    let mut world = World::new(
        "Transfer subscription: an authorized puller pulls",
        "a whitelisted puller (not the merchant) pulls against the subscription",
    );

    let amount_per_period = 50_000_000u64;
    let period_hours = 1u64;
    let end_ts = world.now() + days(30) as i64;

    let puller = world.actor("puller");

    let (alice, _merchant, mint, plan_pda, _, subscription_pda, _, merchant_ata) = setup_plan_and_subscription(
        &mut world,
        amount_per_period,
        period_hours,
        end_ts,
        vec![],
        vec![puller.pubkey()],
    );

    let transfer_amount = 10_000_000u64;
    world.md().step("The whitelisted puller pulls 10 tokens to the merchant ATA");
    let ix = TransferSubscription::new(world.svm_mut(), &puller, alice.pubkey(), mint, subscription_pda, plan_pda)
        .amount(transfer_amount)
        .to(merchant_ata)
        .instruction();
    world.send_ok(&[ix], &[&puller], "TransferSubscription (authorized puller)");

    let bal = get_ata_balance(world.svm(), &merchant_ata);
    world.md().check("the merchant received 10 tokens", 10_000_000, bal);
}

#[test]
fn test_transfer_subscription_unauthorized_caller() {
    let mut world = World::new(
        "Transfer subscription: an unauthorized caller is refused",
        "Mallory (neither merchant nor whitelisted puller) cannot pull",
    );

    let amount_per_period = 50_000_000u64;
    let period_hours = 1u64;
    let end_ts = world.now() + days(30) as i64;

    let (alice, _merchant, mint, plan_pda, _, subscription_pda, _, merchant_ata) =
        setup_plan_and_subscription(&mut world, amount_per_period, period_hours, end_ts, vec![], vec![]);

    let mallory = world.actor("mallory");

    let transfer_amount = 10_000_000u64;
    world.md().step("Mallory, neither merchant nor puller, attempts a pull");
    let ix = TransferSubscription::new(world.svm_mut(), &mallory, alice.pubkey(), mint, subscription_pda, plan_pda)
        .amount(transfer_amount)
        .to(merchant_ata)
        .instruction();
    world.send_err(&[ix], &[&mallory], "TransferSubscription (unauthorized)", SubscriptionsError::Unauthorized);
}

#[test]
fn test_transfer_subscription_multiple_pulls_within_period() {
    let mut world = World::new(
        "Transfer subscription: multiple pulls within a period",
        "two pulls within the same period accumulate up to the period limit",
    );

    let amount_per_period = 50_000_000u64;
    let period_hours = 1u64;
    let end_ts = world.now() + days(30) as i64;

    let (alice, merchant, mint, plan_pda, _, subscription_pda, _, merchant_ata) =
        setup_plan_and_subscription(&mut world, amount_per_period, period_hours, end_ts, vec![], vec![]);

    // First pull.
    world.md().step("The merchant pulls 20 tokens (first pull)");
    let ix = TransferSubscription::new(world.svm_mut(), &merchant, alice.pubkey(), mint, subscription_pda, plan_pda)
        .amount(20_000_000)
        .instruction();
    world.send_ok(&[ix], &[&merchant], "TransferSubscription (first pull)");

    let bal = get_ata_balance(world.svm(), &merchant_ata);
    world.md().check("the merchant has 20 tokens", 20_000_000, bal);

    // Second pull.
    world.md().step("The merchant pulls another 20 tokens (second pull)");
    let ix = TransferSubscription::new(world.svm_mut(), &merchant, alice.pubkey(), mint, subscription_pda, plan_pda)
        .amount(20_000_000)
        .instruction();
    world.send_ok(&[ix], &[&merchant], "TransferSubscription (second pull)");

    let bal = get_ata_balance(world.svm(), &merchant_ata);
    world.md().check("the merchant has 40 tokens", 40_000_000, bal);

    // Verify pulled amount.
    let sub_account = world.svm().get_account(&subscription_pda).unwrap();
    let sub = SubscriptionDelegation::load(&sub_account.data).unwrap();
    world.md().check("the pulled-in-period is 40 tokens", 40_000_000, sub.amount_pulled_in_period);
}

#[test]
fn test_transfer_subscription_exceeds_period_limit() {
    let mut world = World::new(
        "Transfer subscription: exceeding the period limit is refused",
        "a single pull over the per-period amount is rejected",
    );

    let amount_per_period = 50_000_000u64;
    let period_hours = 1u64;
    let end_ts = world.now() + days(30) as i64;

    let (alice, merchant, mint, plan_pda, _, subscription_pda, _, merchant_ata) =
        setup_plan_and_subscription(&mut world, amount_per_period, period_hours, end_ts, vec![], vec![]);

    world.md().step("The merchant attempts to pull 60 tokens over a 50-token period limit");
    let ix = TransferSubscription::new(world.svm_mut(), &merchant, alice.pubkey(), mint, subscription_pda, plan_pda)
        .amount(60_000_000)
        .instruction();
    world.send_err(
        &[ix],
        &[&merchant],
        "TransferSubscription (exceeds period limit)",
        SubscriptionsError::AmountExceedsPeriodLimit,
    );
    let bal = get_ata_balance(world.svm(), &merchant_ata);
    world.md().check("the merchant ATA is still empty", 0, bal);
}

#[test]
fn test_transfer_subscription_period_rollover() {
    let mut world = World::new(
        "Transfer subscription: the period rolls over",
        "advancing past the period boundary resets the pulled amount",
    );

    let amount_per_period = 50_000_000u64;
    let period_hours = 1u64;
    let end_ts = world.now() + days(30) as i64;

    let (alice, merchant, mint, plan_pda, _, subscription_pda, _, merchant_ata) =
        setup_plan_and_subscription(&mut world, amount_per_period, period_hours, end_ts, vec![], vec![]);

    // Pull full period.
    world.md().step("The merchant pulls the full 50-token period");
    let ix = TransferSubscription::new(world.svm_mut(), &merchant, alice.pubkey(), mint, subscription_pda, plan_pda)
        .amount(50_000_000)
        .instruction();
    world.send_ok(&[ix], &[&merchant], "TransferSubscription (period 1)");

    let bal = get_ata_balance(world.svm(), &merchant_ata);
    world.md().check("the merchant has the full period", 50_000_000, bal);

    // Move to next period.
    world.md().step("The clock advances one period");
    world.warp(hours(1));

    // Pull again in new period.
    world.md().step("The merchant pulls 30 tokens in the new period");
    let ix = TransferSubscription::new(world.svm_mut(), &merchant, alice.pubkey(), mint, subscription_pda, plan_pda)
        .amount(30_000_000)
        .instruction();
    world.send_ok(&[ix], &[&merchant], "TransferSubscription (period 2)");

    let bal = get_ata_balance(world.svm(), &merchant_ata);
    world.md().check("the merchant total is 80 tokens", 80_000_000, bal);

    // Verify pulled reset.
    let sub_account = world.svm().get_account(&subscription_pda).unwrap();
    let sub = SubscriptionDelegation::load(&sub_account.data).unwrap();
    world.md().check("the pulled-in-period reset to 30 tokens", 30_000_000, sub.amount_pulled_in_period);
}

#[test]
fn test_transfer_subscription_plan_expired() {
    let mut world = World::new(
        "Transfer subscription: an expired plan is refused",
        "a pull past the plan's end_ts is rejected",
    );

    let amount_per_period = 50_000_000u64;
    let period_hours = 1u64;
    let end_ts = world.now() + days(2) as i64;

    let (alice, merchant, mint, plan_pda, _, subscription_pda, _, _) =
        setup_plan_and_subscription(&mut world, amount_per_period, period_hours, end_ts, vec![], vec![]);

    // Move past plan expiry.
    world.md().step("The clock advances past the plan's end_ts");
    world.warp(days(3));

    let ix = TransferSubscription::new(world.svm_mut(), &merchant, alice.pubkey(), mint, subscription_pda, plan_pda)
        .amount(10_000_000)
        .instruction();
    world.send_err(&[ix], &[&merchant], "TransferSubscription (plan expired)", SubscriptionsError::PlanExpired);
}

#[test]
fn test_transfer_subscription_subscription_cancelled() {
    let mut world = World::new(
        "Transfer subscription: a cancelled subscription past its period is refused",
        "a pull after the subscription's expires_at_ts (a past period end) is rejected",
    );

    let amount_per_period = 50_000_000u64;
    let period_hours = 1u64;
    let end_ts = world.now() + days(30) as i64;

    let (alice, merchant, mint, plan_pda, _, _, _, _) =
        setup_plan_and_subscription(&mut world, amount_per_period, period_hours, end_ts, vec![], vec![]);

    // Create subscription with expires_at_ts set to end of a past period.
    let period_start = world.now() - hours(2) as i64;
    let expires_at = period_start + hours(1) as i64; // end of that period, which is in the past
    let plan_account = world.svm().get_account(&plan_pda).unwrap();
    let plan = Plan::load(&plan_account.data).unwrap();
    let plan_terms = plan.data.terms;
    let subscription_pda = CreateSubscription::new(world.svm_mut(), plan_pda, alice.pubkey(), mint, period_start)
        .terms(plan_terms)
        .expires_at_ts(expires_at)
        .execute();

    // Current time is past expires_at_ts.
    world.md().step("The merchant pulls against a subscription past its cancellation period");
    let ix = TransferSubscription::new(world.svm_mut(), &merchant, alice.pubkey(), mint, subscription_pda, plan_pda)
        .amount(10_000_000)
        .instruction();
    world.send_err(
        &[ix],
        &[&merchant],
        "TransferSubscription (subscription cancelled)",
        SubscriptionsError::SubscriptionCancelled,
    );
}

#[test]
fn test_transfer_subscription_cancelled_allows_current_period() {
    let mut world = World::new(
        "Transfer subscription: cancellation still allows the current period",
        "a pull within the same period as a cancellation still succeeds",
    );

    let amount_per_period = 50_000_000u64;
    let period_hours = 1u64;
    let end_ts = world.now() + days(30) as i64;

    let (alice, merchant, mint, plan_pda, _plan_bump, subscription_pda, _, merchant_ata) =
        setup_plan_and_subscription(&mut world, amount_per_period, period_hours, end_ts, vec![], vec![]);

    // Cancel the subscription (sets expires_at_ts = end of current period).
    world.md().step("Alice cancels her subscription");
    let ix = CancelSubscription::new(world.svm_mut(), &alice, plan_pda, subscription_pda).instruction();
    world.send_ok(&[ix], &[&alice], "CancelSubscription");

    // Pull within the same period should still succeed.
    world.md().step("The merchant pulls within the same period as the cancellation");
    let ix = TransferSubscription::new(world.svm_mut(), &merchant, alice.pubkey(), mint, subscription_pda, plan_pda)
        .amount(10_000_000)
        .instruction();
    world.send_ok(&[ix], &[&merchant], "TransferSubscription (current period)");

    let bal = get_ata_balance(world.svm(), &merchant_ata);
    world.md().check("the merchant received 10 tokens", 10_000_000, bal);
}

#[test]
fn test_transfer_subscription_cancelled_blocks_next_period() {
    let mut world = World::new(
        "Transfer subscription: cancellation blocks the next period",
        "a pull after the period boundary following a cancellation is rejected",
    );

    let amount_per_period = 50_000_000u64;
    let period_hours = 1u64;
    let end_ts = world.now() + days(30) as i64;

    let (alice, merchant, mint, plan_pda, _plan_bump, subscription_pda, _, _) =
        setup_plan_and_subscription(&mut world, amount_per_period, period_hours, end_ts, vec![], vec![]);

    // Cancel the subscription.
    world.md().step("Alice cancels her subscription");
    let ix = CancelSubscription::new(world.svm_mut(), &alice, plan_pda, subscription_pda).instruction();
    world.send_ok(&[ix], &[&alice], "CancelSubscription");

    // Move clock past the period boundary.
    world.md().step("The clock advances past the period boundary");
    world.warp(hours(1));

    // Pull should now fail.
    let ix = TransferSubscription::new(world.svm_mut(), &merchant, alice.pubkey(), mint, subscription_pda, plan_pda)
        .amount(10_000_000)
        .instruction();
    world.send_err(
        &[ix],
        &[&merchant],
        "TransferSubscription (next period blocked)",
        SubscriptionsError::SubscriptionCancelled,
    );
}

#[test]
fn test_transfer_subscription_destination_valid() {
    let mut world = World::new(
        "Transfer subscription: a whitelisted destination is allowed",
        "a pull to a whitelisted destination ATA succeeds",
    );

    let amount_per_period = 50_000_000u64;
    let period_hours = 1u64;
    let end_ts = world.now() + days(30) as i64;

    let dest_wallet = Pubkey::new_unique();
    world.prop(dest_wallet, "dest wallet");

    let (alice, merchant, mint, plan_pda, _, subscription_pda, _, _) =
        setup_plan_and_subscription(&mut world, amount_per_period, period_hours, end_ts, vec![dest_wallet], vec![]);

    let dest_ata = init_ata(world.svm_mut(), mint, dest_wallet, 0);

    world.md().step("The merchant pulls to the whitelisted destination ATA");
    let ix = TransferSubscription::new(world.svm_mut(), &merchant, alice.pubkey(), mint, subscription_pda, plan_pda)
        .amount(10_000_000)
        .to(dest_ata)
        .instruction();
    world.send_ok(&[ix], &[&merchant], "TransferSubscription (valid destination)");

    let bal = get_ata_balance(world.svm(), &dest_ata);
    world.md().check("the destination received 10 tokens", 10_000_000, bal);
}

#[test]
fn test_transfer_subscription_destination_invalid() {
    let mut world = World::new(
        "Transfer subscription: a non-whitelisted destination is refused",
        "a pull to an ATA outside the plan's destination whitelist is rejected",
    );

    let amount_per_period = 50_000_000u64;
    let period_hours = 1u64;
    let end_ts = world.now() + days(30) as i64;

    let dest_wallet = Pubkey::new_unique();
    world.prop(dest_wallet, "dest wallet");

    let (alice, merchant, mint, plan_pda, _, subscription_pda, _, _) =
        setup_plan_and_subscription(&mut world, amount_per_period, period_hours, end_ts, vec![dest_wallet], vec![]);

    // Send to merchant instead of whitelisted dest.
    let merchant_ata = world.fund_ata(mint, &merchant, 0);

    world.md().step("The merchant pulls to its own ATA, outside the whitelist");
    let ix = TransferSubscription::new(world.svm_mut(), &merchant, alice.pubkey(), mint, subscription_pda, plan_pda)
        .amount(10_000_000)
        .to(merchant_ata)
        .instruction();
    world.send_err(
        &[ix],
        &[&merchant],
        "TransferSubscription (invalid destination)",
        SubscriptionsError::UnauthorizedDestination,
    );
}

#[test]
fn test_transfer_subscription_no_destinations_any_receiver() {
    let mut world = World::new(
        "Transfer subscription: no whitelist allows any receiver",
        "with no destination whitelist, a pull to any receiver succeeds",
    );

    let amount_per_period = 50_000_000u64;
    let period_hours = 1u64;
    let end_ts = world.now() + days(30) as i64;

    let (alice, merchant, mint, plan_pda, _, subscription_pda, _, _) =
        setup_plan_and_subscription(&mut world, amount_per_period, period_hours, end_ts, vec![], vec![]);

    // Random third party receives.
    let charlie = world.actor("charlie");
    let charlie_ata = world.fund_ata(mint, &charlie, 0);

    world.md().step("The merchant pulls to Charlie, an arbitrary receiver");
    let ix = TransferSubscription::new(world.svm_mut(), &merchant, alice.pubkey(), mint, subscription_pda, plan_pda)
        .amount(10_000_000)
        .to(charlie_ata)
        .instruction();
    world.send_ok(&[ix], &[&merchant], "TransferSubscription (any receiver)");

    let bal = get_ata_balance(world.svm(), &charlie_ata);
    world.md().check("Charlie received 10 tokens", 10_000_000, bal);
}

#[test]
fn test_transfer_subscription_wrong_subscription_for_plan() {
    let mut world = World::new(
        "Transfer subscription: a mismatched subscription is refused",
        "pulling plan 1 with a subscription minted against plan 2 is rejected",
    );

    let amount_per_period = 50_000_000u64;
    let period_hours = 1u64;
    let end_ts = world.now() + days(30) as i64;

    let (alice, merchant, mint, plan_pda, _, _, _, _) =
        setup_plan_and_subscription(&mut world, amount_per_period, period_hours, end_ts, vec![], vec![]);

    // Create a second plan.
    world.md().step("The merchant creates a second plan");
    let plan2_ix = CreatePlan::new(world.svm_mut(), &merchant, mint)
        .plan_id(2)
        .amount(amount_per_period)
        .period_hours(period_hours)
        .end_ts(end_ts)
        .instruction();
    let plan_pda_2 = get_plan_pda(&merchant.pubkey(), 2).0;
    world.prop(plan_pda_2, "Plan 2");
    world.send_ok(&[plan2_ix], &[&merchant], "CreatePlan (plan 2)");

    // Create subscription for plan 2.
    let plan2_account = world.svm().get_account(&plan_pda_2).unwrap();
    let plan2 = Plan::load(&plan2_account.data).unwrap();
    let plan2_terms = plan2.data.terms;
    let now = world.now();
    let subscription_for_plan2 =
        CreateSubscription::new(world.svm_mut(), plan_pda_2, alice.pubkey(), mint, now)
            .terms(plan2_terms)
            .execute();
    world.prop(subscription_for_plan2, "Subscription (plan 2)");

    // Try to use subscription for plan 2 with plan 1.
    world.md().step("The merchant pulls plan 1 with the plan-2 subscription");
    let ix = TransferSubscription::new(
        world.svm_mut(),
        &merchant,
        alice.pubkey(),
        mint,
        subscription_for_plan2,
        plan_pda, // plan 1
    )
    .amount(10_000_000)
    .instruction();
    world.send_err(
        &[ix],
        &[&merchant],
        "TransferSubscription (plan mismatch)",
        SubscriptionsError::SubscriptionPlanMismatch,
    );
}

#[test]
fn test_transfer_subscription_zero_amount() {
    let mut world = World::new(
        "Transfer subscription: a zero amount is refused",
        "a pull of zero tokens is rejected as an invalid amount",
    );

    let amount_per_period = 50_000_000u64;
    let period_hours = 1u64;
    let end_ts = world.now() + days(30) as i64;

    let (alice, merchant, mint, plan_pda, _, subscription_pda, _, _) =
        setup_plan_and_subscription(&mut world, amount_per_period, period_hours, end_ts, vec![], vec![]);

    world.md().step("The merchant attempts a zero-token pull");
    let ix = TransferSubscription::new(world.svm_mut(), &merchant, alice.pubkey(), mint, subscription_pda, plan_pda)
        .amount(0)
        .instruction();
    world.send_err(&[ix], &[&merchant], "TransferSubscription (zero amount)", SubscriptionsError::InvalidAmount);
}

#[test]
fn test_transfer_subscription_sunset_allows_transfer() {
    // A sunset plan (status=0) should still allow existing subscription pulls.
    // The plan status doesn't block transfers; only end_ts and subscription expires_at_ts do.
    let mut world = World::new(
        "Transfer subscription: a sunset plan still allows pulls",
        "a plan flipped to sunset status still permits an existing subscription's pull",
    );

    let amount_per_period = 50_000_000u64;
    let period_hours = 1u64;
    let end_ts = world.now() + days(30) as i64;

    let (alice, merchant, mint, plan_pda, _, subscription_pda, _, merchant_ata) =
        setup_plan_and_subscription(&mut world, amount_per_period, period_hours, end_ts, vec![], vec![]);

    // Manually set plan status to Sunset (0).
    let mut plan_account = world.svm().get_account(&plan_pda).unwrap();
    // Plan layout: discriminator(1) + owner(32) + bump(1) + status(1) + data(...)
    // status is at offset 34
    plan_account.data[34] = 0; // PlanStatus::Sunset
    world.svm_mut().set_account(plan_pda, plan_account).unwrap();

    world.md().step("Despite the sunset status, the merchant pulls 10 tokens");
    let ix = TransferSubscription::new(world.svm_mut(), &merchant, alice.pubkey(), mint, subscription_pda, plan_pda)
        .amount(10_000_000)
        .instruction();
    world.send_ok(&[ix], &[&merchant], "TransferSubscription (sunset plan)");

    let bal = get_ata_balance(world.svm(), &merchant_ata);
    world.md().check("the merchant received 10 tokens", 10_000_000, bal);
}

#[test]
fn test_transfer_subscription_plan_closed() {
    // When a Plan account is closed (zeroed + ownership transferred to system program),
    // the transfer must fail with PlanClosed rather than a generic error.
    let mut world = World::new(
        "Transfer subscription: a closed plan is refused",
        "pulling against a plan whose account has been closed is rejected with PlanClosed",
    );

    let amount_per_period = 50_000_000u64;
    let period_hours = 1u64;
    let end_ts = world.now() + days(30) as i64;

    let (alice, merchant, mint, plan_pda, _, subscription_pda, _, _) =
        setup_plan_and_subscription(&mut world, amount_per_period, period_hours, end_ts, vec![], vec![]);

    // Simulate plan closure: zero the data and transfer ownership to system program.
    let mut plan_account = world.svm().get_account(&plan_pda).unwrap();
    plan_account.data = vec![];
    plan_account.owner = Pubkey::default(); // system program
    world.svm_mut().set_account(plan_pda, plan_account).unwrap();

    world.md().step("The merchant pulls against the closed plan account");
    let ix = TransferSubscription::new(world.svm_mut(), &merchant, alice.pubkey(), mint, subscription_pda, plan_pda)
        .amount(10_000_000)
        .instruction();
    world.send_err(&[ix], &[&merchant], "TransferSubscription (plan closed)", SubscriptionsError::PlanClosed);
}

#[test]
fn writable_accounts_must_be_writable() {
    use crate::{instructions::transfer_subscription, tests::idl};

    let writable = idl::writable_account_indices("transferSubscription");

    let mut world = World::new(
        "Transfer subscription: writable accounts must be writable",
        "flipping any account the instruction writes to read-only is rejected",
    );

    let amount_per_period = 50_000_000u64;
    let period_hours = 1u64;
    let end_ts = world.now() + days(30) as i64;

    let (alice, merchant, mint, plan_pda, _, subscription_pda, _, _) =
        setup_plan_and_subscription(&mut world, amount_per_period, period_hours, end_ts, vec![], vec![]);
    let fee_payer = world.actor("sponsor");

    let (subscription_authority_pda, _) = get_subscription_authority_pda(&alice.pubkey(), &mint);
    let delegator_ata = get_associated_token_address_with_program_id(&alice.pubkey(), &mint, &TOKEN_PROGRAM_ID);
    let receiver_ata = get_associated_token_address_with_program_id(&merchant.pubkey(), &mint, &TOKEN_PROGRAM_ID);

    let event_authority = Pubkey::new_from_array(event_authority_pda::ID.to_bytes());

    for (idx, name, is_signer) in &writable {
        let mut accounts = vec![
            AccountMeta::new(subscription_pda, false),
            AccountMeta::new_readonly(plan_pda, false),
            AccountMeta::new_readonly(subscription_authority_pda, false),
            AccountMeta::new(delegator_ata, false),
            AccountMeta::new(receiver_ata, false),
            AccountMeta::new_readonly(merchant.pubkey(), true),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            AccountMeta::new_readonly(event_authority, false),
            AccountMeta::new_readonly(PROGRAM_ID, false),
        ];

        let pubkey = accounts[*idx].pubkey;
        accounts[*idx] = AccountMeta::new_readonly(pubkey, *is_signer);

        let transfer_amount: u64 = 10_000_000;
        let data = [
            vec![*transfer_subscription::DISCRIMINATOR],
            transfer_amount.to_le_bytes().to_vec(),
            alice.pubkey().to_bytes().to_vec(),
            mint.to_bytes().to_vec(),
        ]
        .concat();

        let ix = Instruction { program_id: PROGRAM_ID, accounts, data };

        world.send_err(
            &[ix],
            &[&fee_payer, &merchant],
            &format!("TransferSubscription ({name} forced read-only)"),
            SubscriptionsError::AccountNotWritable,
        );
    }
}

#[test]
fn signer_accounts_must_be_signers() {
    use crate::{instructions::transfer_subscription, tests::idl};

    let signers = idl::signer_account_indices("transferSubscription");

    let mut world = World::new(
        "Transfer subscription: signer accounts must sign",
        "flipping any required signer to non-signer is rejected",
    );

    let amount_per_period = 50_000_000u64;
    let period_hours = 1u64;
    let end_ts = world.now() + days(30) as i64;

    let (alice, merchant, mint, plan_pda, _, subscription_pda, _, _) =
        setup_plan_and_subscription(&mut world, amount_per_period, period_hours, end_ts, vec![], vec![]);
    let fee_payer = world.actor("sponsor");

    let (subscription_authority_pda, _) = get_subscription_authority_pda(&alice.pubkey(), &mint);
    let delegator_ata = get_associated_token_address_with_program_id(&alice.pubkey(), &mint, &TOKEN_PROGRAM_ID);
    let receiver_ata = get_associated_token_address_with_program_id(&merchant.pubkey(), &mint, &TOKEN_PROGRAM_ID);

    let event_authority = Pubkey::new_from_array(event_authority_pda::ID.to_bytes());

    for (idx, name, is_writable) in &signers {
        let mut accounts = vec![
            AccountMeta::new(subscription_pda, false),
            AccountMeta::new_readonly(plan_pda, false),
            AccountMeta::new_readonly(subscription_authority_pda, false),
            AccountMeta::new(delegator_ata, false),
            AccountMeta::new(receiver_ata, false),
            AccountMeta::new_readonly(merchant.pubkey(), true),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            AccountMeta::new_readonly(event_authority, false),
            AccountMeta::new_readonly(PROGRAM_ID, false),
        ];

        let pubkey = accounts[*idx].pubkey;
        accounts[*idx] =
            if *is_writable { AccountMeta::new(pubkey, false) } else { AccountMeta::new_readonly(pubkey, false) };

        let transfer_amount: u64 = 10_000_000;
        let data = [
            vec![*transfer_subscription::DISCRIMINATOR],
            transfer_amount.to_le_bytes().to_vec(),
            alice.pubkey().to_bytes().to_vec(),
            mint.to_bytes().to_vec(),
        ]
        .concat();

        let ix = Instruction { program_id: PROGRAM_ID, accounts, data };

        world.send_err(
            &[ix],
            &[&fee_payer],
            &format!("TransferSubscription ({name} forced non-signer)"),
            SubscriptionsError::NotSigner,
        );
    }
}

#[test]
fn test_subscription_transfer_version_mismatch() {
    use crate::state::header::VERSION_OFFSET;

    let mut world = World::new(
        "Transfer subscription: a version-mismatched subscription is refused",
        "a subscription account with a downgraded version requires migration before a pull",
    );

    let amount_per_period = 50_000_000u64;
    let period_hours = 1u64;
    let end_ts = world.now() + days(30) as i64;

    let (alice, merchant, mint, plan_pda, _, subscription_pda, _, merchant_ata) =
        setup_plan_and_subscription(&mut world, amount_per_period, period_hours, end_ts, vec![], vec![]);

    let mut account = world.svm().get_account(&subscription_pda).unwrap();
    account.data[VERSION_OFFSET] = 0;
    world.svm_mut().set_account(subscription_pda, account).unwrap();

    world.md().step("The merchant pulls against a version-mismatched subscription");
    let ix = TransferSubscription::new(world.svm_mut(), &merchant, alice.pubkey(), mint, subscription_pda, plan_pda)
        .amount(10_000_000)
        .instruction();
    world.send_err(
        &[ix],
        &[&merchant],
        "TransferSubscription (version mismatch)",
        SubscriptionsError::MigrationRequired,
    );
    let bal = get_ata_balance(world.svm(), &merchant_ata);
    world.md().check("the merchant ATA is still empty", 0, bal);
}

#[test]
fn test_subscription_transfer_stale_subscription_authority() {
    use crate::tests::utils::CloseSubscriptionAuthority;

    let mut world = World::new(
        "Transfer subscription: a stale subscription authority is refused",
        "re-initializing the authority bumps its init_id, staling the subscription's pull",
    );

    let amount_per_period = 50_000_000;
    let period_hours = 24;
    let end_ts = world.now() + days(30) as i64;

    let (alice, merchant, mint, plan_pda, _, subscription_pda, _, merchant_ata) =
        setup_plan_and_subscription(&mut world, amount_per_period, period_hours, end_ts, vec![], vec![]);

    world.md().step("Alice closes and re-initializes her subscription authority");
    let ix = CloseSubscriptionAuthority::new(world.svm_mut(), &alice, mint).instruction();
    world.send_ok(&[ix], &[&alice], "CloseSubscriptionAuthority");

    world.warp(2);

    world.init_authority(&alice, mint, None).0.assert_ok();

    world.md().step("The merchant pulls against the now-stale subscription");
    let ix = TransferSubscription::new(world.svm_mut(), &merchant, alice.pubkey(), mint, subscription_pda, plan_pda)
        .amount(10_000_000)
        .instruction();
    world.send_err(
        &[ix],
        &[&merchant],
        "TransferSubscription (stale authority)",
        SubscriptionsError::StaleSubscriptionAuthority,
    );
    let bal = get_ata_balance(world.svm(), &merchant_ata);
    world.md().check("the merchant ATA is still empty", 0, bal);
}

#[test]
fn test_transfer_subscription_ghost_plan_rejected() {
    use crate::state::common::PlanStatus;

    let mut world = World::new(
        "Transfer subscription: a ghost (re-created) plan is refused",
        "a plan deleted and re-created at the same PDA with new terms staling the subscription's pull",
    );

    let amount_per_period = 50_000_000u64;
    let period_hours = 1u64;
    let end_ts = world.now() + days(2) as i64;

    let (alice, merchant, mint, plan_pda, _, subscription_pda, _, _) =
        setup_plan_and_subscription(&mut world, amount_per_period, period_hours, end_ts, vec![], vec![]);

    world.md().step("The merchant sunsets, deletes, and re-creates the plan at the same PDA");
    let ix = UpdatePlan::new(world.svm_mut(), &merchant, plan_pda).status(PlanStatus::Sunset).end_ts(end_ts).instruction();
    world.send_ok(&[ix], &[&merchant], "UpdatePlan (sunset)");

    world.warp(days(3));

    let ix = DeletePlan::new(world.svm_mut(), &merchant, plan_pda).instruction();
    world.send_ok(&[ix], &[&merchant], "DeletePlan");

    let new_end_ts = world.now() + days(60) as i64;
    let new_plan_ix = CreatePlan::new(world.svm_mut(), &merchant, mint)
        .plan_id(1)
        .amount(999_000_000)
        .period_hours(720)
        .end_ts(new_end_ts)
        .instruction();
    let new_plan_pda = get_plan_pda(&merchant.pubkey(), 1).0;
    world.send_ok(&[new_plan_ix], &[&merchant], "CreatePlan (re-created)");
    world.md().check("the re-created plan lands at the same PDA", plan_pda, new_plan_pda);

    world.md().step("The merchant pulls against the ghost plan with the original subscription");
    let ix = TransferSubscription::new(world.svm_mut(), &merchant, alice.pubkey(), mint, subscription_pda, plan_pda)
        .amount(10_000_000)
        .instruction();
    world.send_err(
        &[ix],
        &[&merchant],
        "TransferSubscription (ghost plan)",
        SubscriptionsError::PlanTermsMismatch,
    );
}
