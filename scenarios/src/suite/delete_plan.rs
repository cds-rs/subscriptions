//! `delete_plan`, converted to the World/scenario pattern.
//!
//! The plan owner is the `merchant`; the unauthorized caller is `mallory`.
//! Each test builds its own `World` over the handed backend, stages a plan
//! through the observed `CreatePlan`/`UpdatePlan` actions, then deletes it (or
//! fails to). Every send renders its surface into the test's report under
//! `target/md-reports/`.
//!
//! The one balance-shaped assertion (`delete_plan_happy_path`'s "the merchant's
//! balance grew") holds via the rent the merchant reclaims, independent of the
//! transaction fee; the `rent - 10000` tolerance on the follow-up `assert!`
//! already absorbs the fee, so it needs no capability gate.

use solana_signer::Signer;

use testsvm::TestSVM;

use crate::{
    state::common::PlanStatus,
    tests::{
        constants::{MINT_DECIMALS, TOKEN_PROGRAM_ID},
        utils::{days, init_mint, CreatePlan, DeletePlan, UpdatePlan, World},
    },
    SubscriptionsError,
};

pub fn delete_plan_happy_path<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Delete a sunset, expired plan",
        "the merchant sunsets a plan, lets it expire, then reclaims its rent by deleting it",
    );
    let merchant = world.actor("merchant");
    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    world.prop(mint, "USDC mint");

    let end_ts = world.now() + days(2) as i64;
    let (plan_ix, plan_pda) = {
        let b = CreatePlan::new(world.svm_mut(), &merchant, mint).plan_id(1).amount(1_000).period_hours(24).end_ts(end_ts);
        (b.instruction(), b.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&merchant], "CreatePlan");

    world.md().step("The merchant sunsets the plan");
    let update_ix = UpdatePlan::new(world.svm_mut(), &merchant, plan_pda).status(PlanStatus::Sunset).end_ts(end_ts).instruction();
    world.send_ok(&[update_ix], &[&merchant], "UpdatePlan (sunset)");

    world.warp(days(3));

    let account_before = world.svm().get_account(&plan_pda);
    assert!(account_before.is_some());
    let rent = account_before.unwrap().lamports;
    let owner_balance_before = world.svm().get_account(&merchant.pubkey()).unwrap().lamports;

    world.md().step("The merchant deletes the expired plan and reclaims its rent");
    let delete_ix = DeletePlan::new(world.svm_mut(), &merchant, plan_pda).instruction();
    world.send_ok(&[delete_ix], &[&merchant], "DeletePlan");

    let account_after = world.svm().get_account(&plan_pda);
    let plan_drained = account_after.is_none() || account_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0;
    world.md().check("the plan account is drained", true, plan_drained);

    let owner_balance_after = world.svm().get_account(&merchant.pubkey()).unwrap().lamports;
    world.md().check("the merchant's balance grew", true, owner_balance_after > owner_balance_before);
    assert!(owner_balance_after >= owner_balance_before + rent - 10000);
}

pub fn delete_plan_not_owner<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Reject delete by a non-owner",
        "an unauthorized caller (Mallory) cannot delete the merchant's plan",
    );
    let merchant = world.actor("merchant");
    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    world.prop(mint, "USDC mint");

    let end_ts = world.now() + days(2) as i64;
    let (plan_ix, plan_pda) = {
        let b = CreatePlan::new(world.svm_mut(), &merchant, mint).plan_id(1).amount(1_000).period_hours(24).end_ts(end_ts);
        (b.instruction(), b.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&merchant], "CreatePlan");

    let update_ix = UpdatePlan::new(world.svm_mut(), &merchant, plan_pda).status(PlanStatus::Sunset).end_ts(end_ts).instruction();
    world.send_ok(&[update_ix], &[&merchant], "UpdatePlan (sunset)");

    world.warp(days(3));

    let mallory = world.actor("mallory");
    world.md().step("Mallory tries to delete a plan she does not own");
    let delete_ix = DeletePlan::new(world.svm_mut(), &mallory, plan_pda).instruction();
    world.send_err(&[delete_ix], &[&mallory], "DeletePlan (not owner)", SubscriptionsError::NotPlanOwner);

    let account_after = world.svm().get_account(&plan_pda);
    assert!(account_after.is_some());
    let still_funded = account_after.as_ref().map(|a| a.lamports).unwrap_or(0) > 0;
    world.md().check("the plan survives the unauthorized delete", true, still_funded);
}

pub fn delete_active_expired_plan<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Delete an active but expired plan",
        "an Active plan past its end_ts can be deleted (sunset is not required)",
    );
    let merchant = world.actor("merchant");
    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    world.prop(mint, "USDC mint");

    let end_ts = world.now() + days(2) as i64;
    let (plan_ix, plan_pda) = {
        let b = CreatePlan::new(world.svm_mut(), &merchant, mint).plan_id(1).amount(1_000).period_hours(24).end_ts(end_ts);
        (b.instruction(), b.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&merchant], "CreatePlan");

    world.warp(days(3));

    world.md().step("The merchant deletes the expired (still Active) plan");
    let delete_ix = DeletePlan::new(world.svm_mut(), &merchant, plan_pda).instruction();
    world.send_ok(&[delete_ix], &[&merchant], "DeletePlan");

    let account_after = world.svm().get_account(&plan_pda);
    let plan_drained = account_after.is_none() || account_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0;
    world.md().check("the plan account is drained", true, plan_drained);
}

pub fn delete_active_not_expired_fails<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Reject delete of an unexpired Active plan",
        "an Active plan whose end_ts is still in the future cannot be deleted",
    );
    let merchant = world.actor("merchant");
    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    world.prop(mint, "USDC mint");

    let end_ts = world.now() + days(30) as i64;
    let (plan_ix, plan_pda) = {
        let b = CreatePlan::new(world.svm_mut(), &merchant, mint).plan_id(1).amount(1_000).period_hours(24).end_ts(end_ts);
        (b.instruction(), b.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&merchant], "CreatePlan");

    world.md().step("The merchant tries to delete a plan that has not expired");
    let delete_ix = DeletePlan::new(world.svm_mut(), &merchant, plan_pda).instruction();
    world.send_err(&[delete_ix], &[&merchant], "DeletePlan (not expired)", SubscriptionsError::PlanNotExpired);
}

pub fn delete_sunset_not_expired_fails<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Reject delete of a sunset but unexpired plan",
        "sunsetting does not waive the expiry check; an unexpired sunset plan cannot be deleted",
    );
    let merchant = world.actor("merchant");
    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    world.prop(mint, "USDC mint");

    let end_ts = world.now() + days(30) as i64;
    let (plan_ix, plan_pda) = {
        let b = CreatePlan::new(world.svm_mut(), &merchant, mint).plan_id(1).amount(1_000).period_hours(24).end_ts(end_ts);
        (b.instruction(), b.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&merchant], "CreatePlan");

    let update_ix = UpdatePlan::new(world.svm_mut(), &merchant, plan_pda).status(PlanStatus::Sunset).end_ts(end_ts).instruction();
    world.send_ok(&[update_ix], &[&merchant], "UpdatePlan (sunset)");

    world.md().step("The merchant tries to delete a sunset plan that has not yet expired");
    let delete_ix = DeletePlan::new(world.svm_mut(), &merchant, plan_pda).instruction();
    world.send_err(&[delete_ix], &[&merchant], "DeletePlan (sunset, not expired)", SubscriptionsError::PlanNotExpired);
}

pub fn delete_sunset_exactly_at_end_ts_fails<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Reject delete exactly at end_ts",
        "the expiry boundary is exclusive: a plan exactly at its end_ts is not yet deletable",
    );
    let merchant = world.actor("merchant");
    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    world.prop(mint, "USDC mint");

    let end_ts = world.now() + days(2) as i64;
    let (plan_ix, plan_pda) = {
        let b = CreatePlan::new(world.svm_mut(), &merchant, mint).plan_id(1).amount(1_000).period_hours(24).end_ts(end_ts);
        (b.instruction(), b.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&merchant], "CreatePlan");

    let update_ix = UpdatePlan::new(world.svm_mut(), &merchant, plan_pda).status(PlanStatus::Sunset).end_ts(end_ts).instruction();
    world.send_ok(&[update_ix], &[&merchant], "UpdatePlan (sunset)");

    world.warp(days(2));

    world.md().step("The clock sits exactly on end_ts; the merchant attempts to delete");
    let delete_ix = DeletePlan::new(world.svm_mut(), &merchant, plan_pda).instruction();
    world.send_err(&[delete_ix], &[&merchant], "DeletePlan (exactly at end_ts)", SubscriptionsError::PlanNotExpired);
}

pub fn delete_plan_double_delete_fails<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Reject a double delete",
        "deleting an already-deleted plan fails (the account is gone)",
    );
    let merchant = world.actor("merchant");
    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    world.prop(mint, "USDC mint");

    let end_ts = world.now() + days(2) as i64;
    let (plan_ix, plan_pda) = {
        let b = CreatePlan::new(world.svm_mut(), &merchant, mint).plan_id(1).amount(1_000).period_hours(24).end_ts(end_ts);
        (b.instruction(), b.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&merchant], "CreatePlan");

    let update_ix = UpdatePlan::new(world.svm_mut(), &merchant, plan_pda).status(PlanStatus::Sunset).end_ts(end_ts).instruction();
    world.send_ok(&[update_ix], &[&merchant], "UpdatePlan (sunset)");

    world.warp(days(3));

    world.md().step("First delete succeeds");
    let delete_ix = DeletePlan::new(world.svm_mut(), &merchant, plan_pda).instruction();
    world.send_ok(&[delete_ix], &[&merchant], "DeletePlan (first)");

    world.md().step("Second delete of the same plan must fail");
    let delete_ix = DeletePlan::new(world.svm_mut(), &merchant, plan_pda).instruction();
    let res = world.send(&[delete_ix], &[&merchant], "DeletePlan (double delete)");
    world.md().check("the second delete is refused", false, res.is_success());
}

pub fn delete_plan_data_zeroed<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Delete zeroes the plan data",
        "after a delete, any lingering account bytes are zeroed",
    );
    let merchant = world.actor("merchant");
    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    world.prop(mint, "USDC mint");

    let end_ts = world.now() + days(2) as i64;
    let (plan_ix, plan_pda) = {
        let b = CreatePlan::new(world.svm_mut(), &merchant, mint).plan_id(1).amount(1_000).period_hours(24).end_ts(end_ts);
        (b.instruction(), b.plan_pda())
    };
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&merchant], "CreatePlan");

    let update_ix = UpdatePlan::new(world.svm_mut(), &merchant, plan_pda).status(PlanStatus::Sunset).end_ts(end_ts).instruction();
    world.send_ok(&[update_ix], &[&merchant], "UpdatePlan (sunset)");

    world.warp(days(3));

    world.md().step("The merchant deletes the plan");
    let delete_ix = DeletePlan::new(world.svm_mut(), &merchant, plan_pda).instruction();
    world.send_ok(&[delete_ix], &[&merchant], "DeletePlan");

    let account_after = world.svm().get_account(&plan_pda);
    if let Some(account) = account_after {
        assert!(account.data.iter().all(|&byte| byte == 0), "All data should be zeroed after delete");
    }
}
