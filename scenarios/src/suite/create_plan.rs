//! `create_plan`, converted to the World/scenario pattern.
//!
//! Each test builds a `World` over the handed backend, draws the `merchant`
//! (plan owner / payee) from the cast, fabricates a mint, and runs the
//! `CreatePlan` builder's instruction through the observed backend so every send
//! renders its surface into the test's report under `target/md-reports/`.
//! Destinations and pullers are plain props (non-signing pubkeys), not actors.

use solana_account::Account;
use solana_pubkey::Pubkey;
use solana_signer::Signer;

use testsvm::TestSVM;

use crate::{
    state::common::PlanStatus,
    state::plan::Plan,
    tests::{
        constants::{MINT_DECIMALS, TOKEN_PROGRAM_ID},
        pda::get_plan_pda,
        utils::{as_pubkey, days, init_mint, CreatePlan, World},
    },
};

pub fn create_plan_happy_path<B: TestSVM>(backend: B) {
    let mut world = World::new(backend, "Create a plan (happy path)", "the merchant creates a 30-day plan and every field is recorded");
    let merchant = world.actor("merchant");

    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    world.prop(mint, "USDC mint");
    let dest = Pubkey::new_unique();
    let puller = Pubkey::new_unique();
    world.prop(dest, "destination");
    world.prop(puller, "puller");
    let end_ts = world.now() + days(30) as i64;

    let ix = CreatePlan::new(world.svm_mut(), &merchant, mint)
        .plan_id(1)
        .amount(1_000_000)
        .period_hours(720)
        .end_ts(end_ts)
        .destinations(vec![dest])
        .pullers(vec![puller])
        .metadata_uri("https://example.com/plan.json")
        .instruction();
    let (plan_pda, _) = get_plan_pda(&merchant.pubkey(), 1);
    world.prop(plan_pda, "Plan");

    world.md().step("The merchant creates plan 1");
    world.send_ok(&[ix], &[&merchant], "CreatePlan");

    let account = world.svm().get_account(&plan_pda).unwrap();
    assert_eq!(account.data.len(), Plan::LEN);
    let plan = Plan::load(&account.data).unwrap();

    let owner = plan.owner;
    let status = plan.status;
    let id = plan.data.plan_id;
    let plan_mint = plan.data.mint;
    let amt = plan.data.terms.amount;
    let ph = plan.data.terms.period_hours;
    let ets = plan.data.end_ts;
    let dests = plan.data.destinations;
    let pulls = plan.data.pullers;
    let bump = plan.bump;

    world.md().check("the plan owner is the merchant", merchant.pubkey(), as_pubkey(owner.to_bytes()));
    world.md().check("the plan is Active", PlanStatus::Active as u8, status);
    world.md().check("the plan id is 1", 1, id);
    world.md().check("the plan mint is the USDC mint", mint, as_pubkey(plan_mint.to_bytes()));
    world.md().check("the amount is 1_000_000", 1_000_000, amt);
    world.md().check("the period is 720 hours", 720, ph);
    world.md().check("the end timestamp matches", end_ts, ets);
    world.md().check("the first destination is recorded", dest, as_pubkey(dests[0].to_bytes()));
    world.md().check("the first puller is recorded", puller, as_pubkey(pulls[0].to_bytes()));
    assert_ne!(bump, 0);
}

pub fn create_plan_no_expiry<B: TestSVM>(backend: B) {
    let mut world = World::new(backend, "Create a plan with no expiry", "a plan with end_ts 0 never expires");
    let merchant = world.actor("merchant");

    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    let dest = Pubkey::new_unique();

    let ix = CreatePlan::new(world.svm_mut(), &merchant, mint)
        .plan_id(1)
        .amount(500_000)
        .period_hours(24)
        .end_ts(0)
        .destinations(vec![dest])
        .instruction();
    let (plan_pda, _) = get_plan_pda(&merchant.pubkey(), 1);
    world.prop(plan_pda, "Plan");

    world.md().step("The merchant creates a plan with end_ts 0");
    world.send_ok(&[ix], &[&merchant], "CreatePlan (no expiry)");

    let account = world.svm().get_account(&plan_pda).unwrap();
    let plan = Plan::load(&account.data).unwrap();
    let ets = plan.data.end_ts;
    world.md().check("the end timestamp is 0 (no expiry)", 0, ets);
}

pub fn create_plan_period_hours_zero<B: TestSVM>(backend: B) {
    let mut world = World::new(backend, "Reject a zero-hour period", "a plan with period_hours 0 is rejected");
    let merchant = world.actor("merchant");

    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    let dest = Pubkey::new_unique();

    let ix = CreatePlan::new(world.svm_mut(), &merchant, mint)
        .plan_id(1)
        .amount(1_000)
        .period_hours(0)
        .destinations(vec![dest])
        .instruction();

    world.md().step("The merchant tries to create a plan with a zero-hour period");
    world.send_err(&[ix], &[&merchant], "CreatePlan (period_hours 0)", crate::SubscriptionsError::InvalidPeriodLength);
}

pub fn create_plan_period_hours_exceeds_max<B: TestSVM>(backend: B) {
    let mut world = World::new(backend, "Reject an over-long period", "a plan whose period exceeds the max is rejected");
    let merchant = world.actor("merchant");

    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    let dest = Pubkey::new_unique();

    let ix = CreatePlan::new(world.svm_mut(), &merchant, mint)
        .plan_id(1)
        .amount(1_000)
        .period_hours(8761)
        .destinations(vec![dest])
        .instruction();

    world.md().step("The merchant tries a period of 8761 hours (over the max)");
    world.send_err(&[ix], &[&merchant], "CreatePlan (period_hours over max)", crate::SubscriptionsError::InvalidPeriodLength);
}

pub fn create_plan_amount_zero<B: TestSVM>(backend: B) {
    let mut world = World::new(backend, "Reject a zero amount", "a plan with a zero amount is rejected");
    let merchant = world.actor("merchant");

    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    let dest = Pubkey::new_unique();

    let ix = CreatePlan::new(world.svm_mut(), &merchant, mint)
        .plan_id(1)
        .amount(0)
        .period_hours(24)
        .destinations(vec![dest])
        .instruction();

    world.md().step("The merchant tries to create a plan with a zero amount");
    world.send_err(&[ix], &[&merchant], "CreatePlan (amount 0)", crate::SubscriptionsError::InvalidAmount);
}

pub fn create_plan_no_destinations<B: TestSVM>(backend: B) {
    let mut world = World::new(backend, "Create a plan with no destinations", "a plan without destinations leaves the slots zeroed");
    let merchant = world.actor("merchant");

    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);

    let ix = CreatePlan::new(world.svm_mut(), &merchant, mint).plan_id(1).amount(1_000).period_hours(24).instruction();
    let (plan_pda, _) = get_plan_pda(&merchant.pubkey(), 1);
    world.prop(plan_pda, "Plan");

    world.md().step("The merchant creates a plan with no destinations");
    world.send_ok(&[ix], &[&merchant], "CreatePlan (no destinations)");

    let account = world.svm().get_account(&plan_pda).unwrap();
    let plan = Plan::load(&account.data).unwrap();
    let zero = [0u8; 32];
    for dest in &plan.data.destinations {
        assert_eq!(dest.to_bytes(), zero);
    }
}

pub fn create_plan_expired_end_ts<B: TestSVM>(backend: B) {
    let mut world = World::new(backend, "Reject an already-expired end_ts", "a plan whose end_ts is in the past is rejected");
    let merchant = world.actor("merchant");

    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    let dest = Pubkey::new_unique();

    let ix = CreatePlan::new(world.svm_mut(), &merchant, mint)
        .plan_id(1)
        .amount(1_000)
        .period_hours(24)
        .end_ts(1_000)
        .destinations(vec![dest])
        .instruction();

    world.md().step("The merchant tries an end_ts of 1_000 (in the past)");
    world.send_err(&[ix], &[&merchant], "CreatePlan (expired end_ts)", crate::SubscriptionsError::InvalidEndTs);
}

pub fn create_plan_end_ts_before_first_period<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Reject an end_ts before the first period",
        "a plan whose end_ts lands before its first period closes is rejected",
    );
    let merchant = world.actor("merchant");

    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    let dest = Pubkey::new_unique();
    let end_ts = world.now() + days(1) as i64;

    let ix = CreatePlan::new(world.svm_mut(), &merchant, mint)
        .plan_id(1)
        .amount(1_000)
        .period_hours(720)
        .end_ts(end_ts)
        .destinations(vec![dest])
        .instruction();

    world.md().step("The merchant sets an end_ts before the first 720-hour period closes");
    world.send_err(&[ix], &[&merchant], "CreatePlan (end_ts before first period)", crate::SubscriptionsError::InvalidEndTs);
}

pub fn create_plan_wrong_pda<B: TestSVM>(backend: B) {
    let mut world = World::new(backend, "Reject a forged plan PDA", "passing a non-canonical plan PDA is rejected");
    let merchant = world.actor("merchant");

    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    let dest = Pubkey::new_unique();
    let wrong_pda = Pubkey::new_unique();

    let ix = CreatePlan::new(world.svm_mut(), &merchant, mint)
        .plan_id(1)
        .amount(1_000)
        .period_hours(24)
        .destinations(vec![dest])
        .pda(wrong_pda)
        .instruction();

    world.md().step("The merchant points the instruction at a forged plan PDA");
    world.send_err(&[ix], &[&merchant], "CreatePlan (wrong PDA)", crate::SubscriptionsError::InvalidPlanPda);
}

pub fn create_plan_mint_mismatch_attack<B: TestSVM>(backend: B) {
    use solana_instruction::{AccountMeta, Instruction};

    use crate::{
        instructions::create_plan::{PlanData, PlanTerms, MAX_DESTINATIONS, MAX_PULLERS},
        tests::{
            constants::{PROGRAM_ID, SYSTEM_PROGRAM_ID},
            pda::get_plan_pda,
        },
    };

    let mut world = World::new(backend,
        "Reject a mint-mismatch attack",
        "a forged instruction whose embedded mint differs from the passed mint account is rejected",
    );
    let merchant = world.actor("merchant");

    let clean_mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    let malicious_mint = Pubkey::new_unique();
    let dest = Pubkey::new_unique();

    let plan_id: u64 = 1;
    let (plan_pda, _) = get_plan_pda(&merchant.pubkey(), plan_id);

    let zero_addr: pinocchio::Address = [0u8; 32].into();
    let mut destinations = [zero_addr; MAX_DESTINATIONS];
    destinations[0] = dest.to_bytes().into();

    let plan_data = PlanData {
        plan_id,
        mint: malicious_mint.to_bytes().into(),
        terms: PlanTerms { amount: 1_000, period_hours: 24, created_at: 0 },
        end_ts: 0,
        destinations,
        pullers: [zero_addr; MAX_PULLERS],
        metadata_uri: [0u8; 128],
    };

    let plan_data_bytes =
        unsafe { std::slice::from_raw_parts(&plan_data as *const PlanData as *const u8, PlanData::LEN) };
    let mut data = vec![7u8];
    data.extend_from_slice(plan_data_bytes);

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(merchant.pubkey(), true),
            AccountMeta::new(plan_pda, false),
            AccountMeta::new_readonly(clean_mint, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        ],
        data,
    };

    world.md().step("The merchant embeds a mint in the instruction data that differs from the passed mint account");
    world.send_err(&[ix], &[&merchant], "CreatePlan (mint mismatch)", crate::SubscriptionsError::MintMismatch);
}

pub fn create_plan_prefunded_pda<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Survive a pre-funded plan PDA",
        "a griefer pre-funds the plan PDA; the merchant can still create the plan",
    );
    let merchant = world.actor("merchant");

    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    let dest = Pubkey::new_unique();
    let plan_id: u64 = 42;

    let (plan_pda_addr, _) = get_plan_pda(&merchant.pubkey(), plan_id);
    world.prop(plan_pda_addr, "Plan");
    world.svm_mut()
        .set_account(
            &plan_pda_addr,
            Account { lamports: 1_000, data: vec![], owner: Pubkey::default(), executable: false, rent_epoch: 0 },
        );

    let ix = CreatePlan::new(world.svm_mut(), &merchant, mint)
        .plan_id(plan_id)
        .amount(1_000_000)
        .period_hours(720)
        .destinations(vec![dest])
        .instruction();

    world.md().step("Despite the pre-funded PDA, the merchant creates the plan");
    world.send_ok(&[ix], &[&merchant], "CreatePlan (pre-funded PDA)");

    let account = world.svm().get_account(&plan_pda_addr).unwrap();
    let plan = Plan::load(&account.data).unwrap();
    let owner = plan.owner;
    let status = plan.status;
    world.md().check("the plan owner is the merchant", merchant.pubkey(), as_pubkey(owner.to_bytes()));
    world.md().check("the plan is Active", PlanStatus::Active as u8, status);
}

pub fn create_plan_duplicate_plan_id<B: TestSVM>(backend: B) {
    let mut world = World::new(backend, "Reject a duplicate plan id", "re-creating a plan with an existing id is rejected");
    let merchant = world.actor("merchant");

    let mint = init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, None, &[]);
    let dest = Pubkey::new_unique();

    let ix = CreatePlan::new(world.svm_mut(), &merchant, mint)
        .plan_id(1)
        .amount(1_000)
        .period_hours(24)
        .destinations(vec![dest])
        .instruction();
    let (plan_pda, _) = get_plan_pda(&merchant.pubkey(), 1);
    world.prop(plan_pda, "Plan");

    world.md().step("The merchant creates plan 1");
    world.send_ok(&[ix], &[&merchant], "CreatePlan (first)");

    let ix2 = CreatePlan::new(world.svm_mut(), &merchant, mint)
        .plan_id(1)
        .amount(2_000)
        .period_hours(48)
        .destinations(vec![dest])
        .instruction();

    world.md().step("The merchant tries to create plan 1 again");
    world.send_err(&[ix2], &[&merchant], "CreatePlan (duplicate id)", crate::SubscriptionsError::PlanAlreadyExists);
}
