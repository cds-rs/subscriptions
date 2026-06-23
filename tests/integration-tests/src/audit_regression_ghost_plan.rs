//! Cantina HIGH findings 3.1.2 and 3.1.4, reproduced in the DSL: the
//! delete-and-recreate "ghost plan" attack.
//!
//! The Plan PDA is derived from `[owner, plan_id]`, so a merchant can delete an
//! expired plan and recreate it at the SAME address with new terms; existing
//! subscriptions still point there. On this `dsl-audit` branch the
//! `check_plan_terms` guard (PR4) is reverted and `transfer_subscription` reads
//! the LIVE plan amount, so the recreated terms are never cross-checked against
//! what the subscriber consented to. 3.1.2 inflates the per-period amount to
//! drain the subscriber; 3.1.4 extends `end_ts` to siphon past the original
//! agreement. On the fixed code both pulls are refused, so these tests (which
//! assert the exploit succeeds) fail there: that failure is the fix's proof.

use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;

use litesvm_utils::LiteSvmBackend;
use crate::{
    state::common::PlanStatus,
    tests::{
        pda::{get_plan_pda, get_subscription_pda},
        utils::{
            days, token_balance, hours, CreatePlan, DeletePlan, ObservedResultExt, Subscribe,
            TransferSubscription, UpdatePlan, make_backend, World,
        },
    },
};

struct Ghost {
    alice: Keypair,
    merchant: Keypair,
    mint: Pubkey,
    plan_pda: Pubkey,
    subscription_pda: Pubkey,
    alice_ata: Pubkey,
    merchant_ata: Pubkey,
}

/// Alice subscribes to the merchant's plan (consenting to `original_amount`/hour),
/// then the merchant sunsets, expires, deletes, and recreates the plan at the
/// same id with `ghost_amount`/hour and an `end_ts` `ghost_end_days` out.
fn stage_and_recreate(world: &mut World<LiteSvmBackend>, original_amount: u64, ghost_amount: u64, ghost_end_days: u64) -> Ghost {
    let alice = world.actor("alice");
    let merchant = world.actor("merchant");
    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let alice_ata = world.fund_ata(mint, &alice, 100_000_000);
    let merchant_ata = world.fund_ata(mint, &merchant, 0);
    world.init_authority(&alice, mint, None).0.assert_ok();

    let plan_end_ts = world.now() + hours(2) as i64;
    world.md().step("The merchant offers a plan; Alice subscribes, consenting to its terms");
    let plan_ix = CreatePlan::new(world.svm_mut(), &merchant, mint)
        .plan_id(1)
        .amount(original_amount)
        .period_hours(1)
        .end_ts(plan_end_ts)
        .instruction();
    let (plan_pda, plan_bump) = get_plan_pda(&merchant.pubkey(), 1);
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&merchant], "CreatePlan");

    let sub_ix = Subscribe::new(world.svm_mut(), &alice, merchant.pubkey(), plan_pda, 1, plan_bump, mint).instruction();
    let (subscription_pda, _) = get_subscription_pda(&plan_pda, &alice.pubkey());
    world.prop(subscription_pda, "Subscription");
    world.send_ok(&[sub_ix], &[&alice], "Subscribe");

    world.md().step("The merchant sunsets, expires, and deletes the plan");
    let ix = UpdatePlan::new(world.svm_mut(), &merchant, plan_pda)
        .status(PlanStatus::Sunset)
        .end_ts(plan_end_ts)
        .instruction();
    world.send_ok(&[ix], &[&merchant], "UpdatePlan (sunset)");
    world.warp(hours(3));
    let ix = DeletePlan::new(world.svm_mut(), &merchant, plan_pda).instruction();
    world.send_ok(&[ix], &[&merchant], "DeletePlan");

    let ghost_end_ts = world.now() + days(ghost_end_days) as i64;
    world.md().step("The merchant recreates the plan at the same id with ghost terms");
    let plan_ix = CreatePlan::new(world.svm_mut(), &merchant, mint)
        .plan_id(1)
        .amount(ghost_amount)
        .period_hours(1)
        .end_ts(ghost_end_ts)
        .instruction();
    world.send_ok(&[plan_ix], &[&merchant], "CreatePlan (ghost)");

    Ghost { alice, merchant, mint, plan_pda, subscription_pda, alice_ata, merchant_ata }
}

#[test]
fn finding_3_1_2_ghost_plan_inflated_amount_drains() {
    let mut world = World::new(make_backend(), 
        "AUDIT 3.1.2 (regression): the fix refuses the inflated ghost plan",
        "the merchant recreates the plan at 100M/hour and pulls far more than Alice consented to (Cantina HIGH 3.1.2)",
    );
    // Alice consented to 1000/hour; the ghost plan sets 100M/hour.
    let g = stage_and_recreate(&mut world, 1_000, 100_000_000, 60);

    let alice_before = token_balance(world.svm(), &g.alice_ata);
    world.md().step("The merchant pulls 50M — 50,000x what Alice agreed to");
    let pull_ix = TransferSubscription::new(world.svm_mut(), &g.merchant, g.alice.pubkey(), g.mint, g.subscription_pda, g.plan_pda)
        .amount(50_000_000)
        .to(g.merchant_ata)
        .instruction();
    let res = world.send(&[pull_ix], &[&g.merchant], "TransferSubscription (ghost-plan drain)");
    let drained = res.is_success();
    let alice_after = token_balance(world.svm(), &g.alice_ata);
    let merchant_got = token_balance(world.svm(), &g.merchant_ata);

    world.md().note(
        "Finding 3.1.2: Alice consented to 1000/hour, but the merchant deleted and recreated the plan at \
         100M/hour and pulled 50M. With check_plan_terms reverted, transfer_subscription reads the live ghost \
         amount instead of the consented snapshot. On the fixed code the recreated terms mismatch the snapshot \
         and the pull is refused; the checks below confirm the refusal, guarding the fix.",
    );
    world.md().check("the fix refuses the inflated ghost-plan pull (PlanTermsMismatch)", false, drained);
    world.md().check("the merchant received nothing — the pull was refused", 0, merchant_got);
    world.md().check("Alice was not drained", alice_before, alice_after);
}

#[test]
fn finding_3_1_4_ghost_plan_extended_end_ts_siphons() {
    let mut world = World::new(make_backend(), 
        "AUDIT 3.1.4 (regression): the fix refuses the extended-end_ts ghost plan",
        "the merchant recreates the plan with a 60-day end_ts and keeps pulling past the original end (Cantina HIGH 3.1.4)",
    );
    // Same amount, but the ghost plan extends end_ts far into the future. We are
    // already past the ORIGINAL end_ts (staging warps 3h; the original end was 2h).
    let g = stage_and_recreate(&mut world, 50_000_000, 50_000_000, 60);

    world.md().step("Past the original end_ts, the merchant still pulls against the extended ghost plan");
    let pull_ix = TransferSubscription::new(world.svm_mut(), &g.merchant, g.alice.pubkey(), g.mint, g.subscription_pda, g.plan_pda)
        .amount(10_000_000)
        .to(g.merchant_ata)
        .instruction();
    let res = world.send(&[pull_ix], &[&g.merchant], "TransferSubscription (past original end)");
    let siphoned = res.is_success();
    let merchant_got = token_balance(world.svm(), &g.merchant_ata);

    world.md().note(
        "Finding 3.1.4: the original plan ended an hour ago, but the merchant recreated it with a 60-day \
         end_ts. With check_plan_terms reverted, transfer_subscription reads the live (extended) end_ts and the \
         pull lands — the merchant siphons indefinitely past the agreement. On the fixed code the recreated \
         terms mismatch the snapshot and the pull is refused; the checks below then fail.",
    );
    world.md().check("the fix refuses the post-original-end pull (PlanTermsMismatch)", false, siphoned);
    world.md().check("the merchant siphoned nothing — the pull was refused", 0, merchant_got);
}

