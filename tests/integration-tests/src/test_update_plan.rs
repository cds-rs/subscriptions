//! `update_plan`, converted to the World/scenario pattern.
//!
//! The plan owner is the `merchant` from the cast; an unauthorized updater is
//! `mallory`. Each test builds its own `World`, stages a plan through the
//! observed `CreatePlan`, then performs the `UpdatePlan` through the
//! observed `send_*`. Every send renders its surface into the test's report.

use std::vec::Vec;

use solana_pubkey::Pubkey;
use solana_signer::Signer;

use litesvm_utils::TestSVM;

use crate::{
    state::common::PlanStatus,
    state::plan::Plan,
    tests::{
        constants::{MINT_DECIMALS, TOKEN_PROGRAM_ID},
        utils::{as_pubkey, days, init_mint, CreatePlan, UpdatePlan, World},
    },
    SubscriptionsError,
};

#[test]
fn update_plan_happy_path() {
    let mut world = World::new(
        "Update a plan (happy path)",
        "the merchant sets a plan to Sunset with an end timestamp and a fresh metadata URI",
    );
    let owner = world.actor("merchant");
    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    world.prop(mint, "USDC mint");

    let (plan_ix, plan_pda) = {
        let b = CreatePlan::new(world.svm_mut(), &owner, mint).plan_id(1).amount(1_000_000).period_hours(720);
        (b.instruction(), b.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&owner], "CreatePlan");

    world.md().step("The merchant sunsets the plan with a new end timestamp and metadata");
    let end_ts = world.now() + days(60) as i64;
    let update_ix = UpdatePlan::new(world.svm_mut(), &owner, plan_pda)
        .status(PlanStatus::Sunset)
        .end_ts(end_ts)
        .metadata_uri("https://example.com/updated.json")
        .instruction();
    world.send_ok(&[update_ix], &[&owner], "UpdatePlan");

    let account = world.svm().get_account(&plan_pda).unwrap();
    let plan = Plan::load(&account.data).unwrap();
    let status = plan.status;
    let ets = plan.data.end_ts;
    let uri_bytes = plan.data.metadata_uri;
    world.md().check("the plan is Sunset", PlanStatus::Sunset as u8, status);
    assert_ne!(ets, 0);
    let uri = core::str::from_utf8(&uri_bytes).unwrap();
    assert!(uri.starts_with("https://example.com/updated.json"));
}

#[test]
fn update_plan_preserves_immutable_fields() {
    let mut world = World::new(
        "Update a plan preserves immutable fields",
        "an update touches mutable fields only; amount, period, mint, destinations, and id are unchanged",
    );
    let owner = world.actor("merchant");
    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    world.prop(mint, "USDC mint");
    let dest = Pubkey::new_unique();
    let puller = Pubkey::new_unique();

    let (plan_ix, plan_pda) = {
        let b = CreatePlan::new(world.svm_mut(), &owner, mint)
            .plan_id(1)
            .amount(1_000_000)
            .period_hours(720)
            .destinations(vec![dest])
            .pullers(vec![puller])
            .metadata_uri("https://example.com/plan.json");
        (b.instruction(), b.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&owner], "CreatePlan");

    let account_before = world.svm().get_account(&plan_pda).unwrap();
    let plan_before = Plan::load(&account_before.data).unwrap();
    let amount_before = plan_before.data.terms.amount;
    let period_before = plan_before.data.terms.period_hours;
    let mint_before = plan_before.data.mint;
    let dests_before = plan_before.data.destinations;
    let id_before = plan_before.data.plan_id;

    world.md().step("The merchant updates mutable fields (status, end_ts, pullers, metadata)");
    let end_ts = world.now() + days(60) as i64;
    let update_ix = UpdatePlan::new(world.svm_mut(), &owner, plan_pda)
        .status(PlanStatus::Sunset)
        .end_ts(end_ts)
        .pullers(vec![puller])
        .metadata_uri("https://example.com/v2.json")
        .instruction();
    world.send_ok(&[update_ix], &[&owner], "UpdatePlan");

    let account_after = world.svm().get_account(&plan_pda).unwrap();
    let plan_after = Plan::load(&account_after.data).unwrap();
    let amount_after = plan_after.data.terms.amount;
    let period_after = plan_after.data.terms.period_hours;
    let mint_after = plan_after.data.mint;
    let dests_after = plan_after.data.destinations;
    let id_after = plan_after.data.plan_id;
    world.md().check("the amount is unchanged", amount_before, amount_after);
    world.md().check("the period is unchanged", period_before, period_after);
    world.md().check("the mint is unchanged", as_pubkey(mint_before.to_bytes()), as_pubkey(mint_after.to_bytes()));
    for i in 0..4 {
        assert_eq!(dests_after[i].to_bytes(), dests_before[i].to_bytes());
    }
    world.md().check("the plan id is unchanged", id_before, id_after);
}

#[test]
fn update_plan_not_owner() {
    let mut world = World::new(
        "Update a plan rejects a non-owner",
        "an unauthorized signer cannot update someone else's plan",
    );
    let owner = world.actor("merchant");
    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    world.prop(mint, "USDC mint");

    let (plan_ix, plan_pda) = {
        let b = CreatePlan::new(world.svm_mut(), &owner, mint).plan_id(1).amount(1_000).period_hours(24);
        (b.instruction(), b.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&owner], "CreatePlan");

    let non_owner = world.actor("mallory");
    world.md().step("Mallory tries to update the merchant's plan");
    let end_ts = world.now() + days(60) as i64;
    let update_ix =
        UpdatePlan::new(world.svm_mut(), &non_owner, plan_pda).status(PlanStatus::Sunset).end_ts(end_ts).instruction();
    world.send_err(&[update_ix], &[&non_owner], "UpdatePlan (not owner)", SubscriptionsError::NotPlanOwner);
}

#[test]
fn update_plan_invalid_status() {
    let mut world = World::new(
        "Update a plan rejects an invalid status",
        "a raw status value outside the valid set is rejected",
    );
    let owner = world.actor("merchant");
    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    world.prop(mint, "USDC mint");

    let (plan_ix, plan_pda) = {
        let b = CreatePlan::new(world.svm_mut(), &owner, mint).plan_id(1).amount(1_000).period_hours(24);
        (b.instruction(), b.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&owner], "CreatePlan");

    world.md().step("The merchant submits a raw status value of 99");
    let update_ix = UpdatePlan::new(world.svm_mut(), &owner, plan_pda).status_raw(99).instruction();
    world.send_err(&[update_ix], &[&owner], "UpdatePlan (invalid status)", SubscriptionsError::InvalidPlanStatus);
}

#[test]
fn update_plan_end_ts_in_past() {
    let mut world = World::new(
        "Update a plan rejects an end_ts in the past",
        "an end timestamp before the current clock is rejected",
    );
    let owner = world.actor("merchant");
    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    world.prop(mint, "USDC mint");

    let (plan_ix, plan_pda) = {
        let b = CreatePlan::new(world.svm_mut(), &owner, mint).plan_id(1).amount(1_000).period_hours(24);
        (b.instruction(), b.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&owner], "CreatePlan");

    world.md().step("The merchant submits an end timestamp in the past");
    let update_ix = UpdatePlan::new(world.svm_mut(), &owner, plan_pda).end_ts(1000).instruction();
    world.send_err(&[update_ix], &[&owner], "UpdatePlan (end_ts in past)", SubscriptionsError::InvalidEndTs);
}

#[test]
fn update_plan_clear_end_ts() {
    let mut world = World::new(
        "Update a plan can clear its end_ts",
        "setting end_ts to zero clears the previously set expiry",
    );
    let owner = world.actor("merchant");
    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    world.prop(mint, "USDC mint");

    let end_ts = world.now() + days(30) as i64;
    let (plan_ix, plan_pda) = {
        let b = CreatePlan::new(world.svm_mut(), &owner, mint).plan_id(1).amount(1_000).period_hours(24).end_ts(end_ts);
        (b.instruction(), b.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&owner], "CreatePlan");

    world.md().step("The merchant clears the end timestamp");
    let update_ix = UpdatePlan::new(world.svm_mut(), &owner, plan_pda).end_ts(0).instruction();
    world.send_ok(&[update_ix], &[&owner], "UpdatePlan (clear end_ts)");

    let account = world.svm().get_account(&plan_pda).unwrap();
    let plan = Plan::load(&account.data).unwrap();
    let ets = plan.data.end_ts;
    world.md().check("the end timestamp is cleared", 0_i64, ets);
}

#[test]
fn update_plan_sunset_is_terminal() {
    let mut world = World::new(
        "Sunset is terminal",
        "once a plan is sunset it cannot be reverted to Active",
    );
    let owner = world.actor("merchant");
    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    world.prop(mint, "USDC mint");

    let (plan_ix, plan_pda) = {
        let b = CreatePlan::new(world.svm_mut(), &owner, mint).plan_id(1).amount(1_000).period_hours(24);
        (b.instruction(), b.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&owner], "CreatePlan");

    world.md().step("The merchant sunsets the plan");
    let end_ts = world.now() + days(60) as i64;
    let sunset_ix =
        UpdatePlan::new(world.svm_mut(), &owner, plan_pda).status(PlanStatus::Sunset).end_ts(end_ts).instruction();
    world.send_ok(&[sunset_ix], &[&owner], "UpdatePlan (sunset)");
    let account = world.svm().get_account(&plan_pda).unwrap();
    let plan = Plan::load(&account.data).unwrap();
    world.md().check("the plan is Sunset", PlanStatus::Sunset as u8, plan.status);

    world.md().step("The merchant tries to revive the plan back to Active");
    let revive_ix = UpdatePlan::new(world.svm_mut(), &owner, plan_pda).status(PlanStatus::Active).instruction();
    world.send_err(
        &[revive_ix],
        &[&owner],
        "UpdatePlan (revive after sunset)",
        SubscriptionsError::PlanImmutableAfterSunset,
    );
}

#[test]
fn update_plan_no_op() {
    let mut world = World::new(
        "Update a plan no-op leaves it unchanged",
        "an update with no changed fields leaves the plan account bytes untouched",
    );
    let owner = world.actor("merchant");
    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    world.prop(mint, "USDC mint");

    let (plan_ix, plan_pda) = {
        let b = CreatePlan::new(world.svm_mut(), &owner, mint).plan_id(1).amount(1_000).period_hours(24);
        (b.instruction(), b.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&owner], "CreatePlan");

    let account_before = world.svm().get_account(&plan_pda).unwrap();

    world.md().step("The merchant submits an empty update");
    let update_ix = UpdatePlan::new(world.svm_mut(), &owner, plan_pda).instruction();
    world.send_ok(&[update_ix], &[&owner], "UpdatePlan (no-op)");

    let account_after = world.svm().get_account(&plan_pda).unwrap();
    world.md().check("the plan bytes are unchanged", true, account_before.data == account_after.data);
}

#[test]
fn update_plan_sunset_requires_end_ts() {
    let mut world = World::new(
        "Sunset requires an end_ts",
        "sunsetting a plan without supplying a non-zero end timestamp is rejected",
    );
    let owner = world.actor("merchant");
    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    world.prop(mint, "USDC mint");

    let (plan_ix, plan_pda) = {
        let b = CreatePlan::new(world.svm_mut(), &owner, mint).plan_id(1).amount(1_000).period_hours(24);
        (b.instruction(), b.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&owner], "CreatePlan");

    world.md().step("The merchant tries to sunset without an end timestamp");
    let update_ix = UpdatePlan::new(world.svm_mut(), &owner, plan_pda).status(PlanStatus::Sunset).instruction();
    world.send_err(&[update_ix], &[&owner], "UpdatePlan (sunset, no end_ts)", SubscriptionsError::SunsetRequiresEndTs);
}

#[test]
fn update_plan_at_exact_expiry_boundary() {
    let mut world = World::new(
        "Update a plan at the exact expiry boundary",
        "a plan can still be updated at the instant its end timestamp is reached",
    );
    let owner = world.actor("merchant");
    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    world.prop(mint, "USDC mint");

    let end_ts = world.now() + days(2) as i64;
    let (plan_ix, plan_pda) = {
        let b = CreatePlan::new(world.svm_mut(), &owner, mint).plan_id(1).amount(1_000).period_hours(24).end_ts(end_ts);
        (b.instruction(), b.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&owner], "CreatePlan");

    world.md().step("Time advances to the exact expiry boundary");
    world.warp(days(2));

    let update_ix =
        UpdatePlan::new(world.svm_mut(), &owner, plan_pda).metadata_uri("https://example.com/at-boundary.json").instruction();
    world.send_ok(&[update_ix], &[&owner], "UpdatePlan (at boundary)");

    let account = world.svm().get_account(&plan_pda).unwrap();
    let plan = Plan::load(&account.data).unwrap();
    let uri = core::str::from_utf8(&plan.data.metadata_uri).unwrap();
    assert!(uri.starts_with("https://example.com/at-boundary.json"));
}

#[test]
fn update_plan_expired() {
    let mut world = World::new(
        "Update an expired plan is rejected",
        "once past its end timestamp, a plan can no longer be updated",
    );
    let owner = world.actor("merchant");
    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    world.prop(mint, "USDC mint");

    let end_ts = world.now() + days(2) as i64;
    let (plan_ix, plan_pda) = {
        let b = CreatePlan::new(world.svm_mut(), &owner, mint).plan_id(1).amount(1_000).period_hours(24).end_ts(end_ts);
        (b.instruction(), b.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&owner], "CreatePlan");

    world.md().step("Time advances past the plan's expiry");
    world.warp(days(3));

    let new_end = world.now() + days(30) as i64;
    let update_ix = UpdatePlan::new(world.svm_mut(), &owner, plan_pda).end_ts(new_end).instruction();
    world.send_err(&[update_ix], &[&owner], "UpdatePlan (expired)", SubscriptionsError::PlanExpired);
}

#[test]
fn update_plan_add_pullers() {
    let mut world = World::new(
        "Update a plan can add pullers",
        "an update populates the previously empty puller whitelist",
    );
    let owner = world.actor("merchant");
    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    world.prop(mint, "USDC mint");

    let (plan_ix, plan_pda) = {
        let b = CreatePlan::new(world.svm_mut(), &owner, mint).plan_id(1).amount(1_000).period_hours(24);
        (b.instruction(), b.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&owner], "CreatePlan");

    let account = world.svm().get_account(&plan_pda).unwrap();
    let plan = Plan::load(&account.data).unwrap();
    let zero = [0u8; 32];
    for p in &plan.data.pullers {
        assert_eq!(p.to_bytes(), zero);
    }

    let puller_a = Pubkey::new_unique();
    let puller_b = Pubkey::new_unique();
    world.md().step("The merchant adds two pullers to the whitelist");
    let update_ix =
        UpdatePlan::new(world.svm_mut(), &owner, plan_pda).pullers(vec![puller_a, puller_b]).instruction();
    world.send_ok(&[update_ix], &[&owner], "UpdatePlan (add pullers)");

    let account = world.svm().get_account(&plan_pda).unwrap();
    let plan = Plan::load(&account.data).unwrap();
    world.md().check("puller 0 is puller_a", puller_a, as_pubkey(plan.data.pullers[0].to_bytes()));
    world.md().check("puller 1 is puller_b", puller_b, as_pubkey(plan.data.pullers[1].to_bytes()));
    assert_eq!(plan.data.pullers[2].to_bytes(), zero);
    assert_eq!(plan.data.pullers[3].to_bytes(), zero);
}

#[test]
fn update_plan_remove_pullers_owner_still_authorized() {
    let mut world = World::new(
        "Removing pullers keeps the owner authorized",
        "clearing the puller whitelist leaves the plan owner able to pull; a random key cannot",
    );
    let owner = world.actor("merchant");
    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    world.prop(mint, "USDC mint");

    let puller_a = Pubkey::new_unique();
    let puller_b = Pubkey::new_unique();
    let (plan_ix, plan_pda) = {
        let b = CreatePlan::new(world.svm_mut(), &owner, mint)
            .plan_id(1)
            .amount(1_000)
            .period_hours(24)
            .pullers(vec![puller_a, puller_b]);
        (b.instruction(), b.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&owner], "CreatePlan");

    world.md().step("The merchant clears the puller whitelist");
    let update_ix = UpdatePlan::new(world.svm_mut(), &owner, plan_pda).instruction();
    world.send_ok(&[update_ix], &[&owner], "UpdatePlan (clear pullers)");

    let account = world.svm().get_account(&plan_pda).unwrap();
    let plan = Plan::load(&account.data).unwrap();
    let zero = [0u8; 32];
    for p in &plan.data.pullers {
        assert_eq!(p.to_bytes(), zero);
    }

    let owner_addr: pinocchio::Address = owner.pubkey().to_bytes().into();
    world.md().check("the owner can still pull", true, plan.can_pull(&owner_addr).is_ok());

    let random_addr: pinocchio::Address = Pubkey::new_unique().to_bytes().into();
    world.md().check("a random key cannot pull", true, plan.can_pull(&random_addr).is_err());
}

#[test]
fn update_plan_replace_pullers() {
    let mut world = World::new(
        "Update a plan replaces the puller whitelist",
        "an update overwrites the existing pullers wholesale, not appends",
    );
    let owner = world.actor("merchant");
    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    world.prop(mint, "USDC mint");

    let puller_a = Pubkey::new_unique();
    let (plan_ix, plan_pda) = {
        let b = CreatePlan::new(world.svm_mut(), &owner, mint)
            .plan_id(1)
            .amount(1_000)
            .period_hours(24)
            .pullers(vec![puller_a]);
        (b.instruction(), b.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&owner], "CreatePlan");

    let puller_b = Pubkey::new_unique();
    world.md().step("The merchant replaces the whitelist with a single new puller");
    let update_ix = UpdatePlan::new(world.svm_mut(), &owner, plan_pda).pullers(vec![puller_b]).instruction();
    world.send_ok(&[update_ix], &[&owner], "UpdatePlan (replace pullers)");

    let account = world.svm().get_account(&plan_pda).unwrap();
    let plan = Plan::load(&account.data).unwrap();
    world.md().check("puller 0 is the replacement", puller_b, as_pubkey(plan.data.pullers[0].to_bytes()));
    let zero = [0u8; 32];
    assert_eq!(plan.data.pullers[1].to_bytes(), zero);
}

#[test]
fn update_plan_max_pullers() {
    let mut world = World::new(
        "Update a plan fills the puller whitelist to capacity",
        "an update can set the full set of four pullers",
    );
    let owner = world.actor("merchant");
    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    world.prop(mint, "USDC mint");

    let (plan_ix, plan_pda) = {
        let b = CreatePlan::new(world.svm_mut(), &owner, mint).plan_id(1).amount(1_000).period_hours(24);
        (b.instruction(), b.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&owner], "CreatePlan");

    let pullers: Vec<Pubkey> = (0..4).map(|_| Pubkey::new_unique()).collect();
    world.md().step("The merchant sets the maximum of four pullers");
    let update_ix = UpdatePlan::new(world.svm_mut(), &owner, plan_pda).pullers(pullers.clone()).instruction();
    world.send_ok(&[update_ix], &[&owner], "UpdatePlan (max pullers)");

    let account = world.svm().get_account(&plan_pda).unwrap();
    let plan = Plan::load(&account.data).unwrap();
    for (i, p) in pullers.iter().enumerate() {
        assert_eq!(plan.data.pullers[i].to_bytes(), p.to_bytes());
    }
}

#[test]
fn update_plan_rejects_near_immediate_end_ts() {
    let mut world = World::new(
        "Update a plan rejects a near-immediate end_ts",
        "an end timestamp only seconds away (shorter than one period) is rejected",
    );
    let owner = world.actor("merchant");
    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    world.prop(mint, "USDC mint");

    let (plan_ix, plan_pda) = {
        let b = CreatePlan::new(world.svm_mut(), &owner, mint).plan_id(1).amount(1_000).period_hours(720);
        (b.instruction(), b.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&owner], "CreatePlan");

    world.md().step("The merchant submits an end timestamp only two seconds out");
    let near = world.now() + 2;
    let update_ix = UpdatePlan::new(world.svm_mut(), &owner, plan_pda).end_ts(near).instruction();
    world.send_err(&[update_ix], &[&owner], "UpdatePlan (near-immediate end_ts)", SubscriptionsError::InvalidEndTs);
}
