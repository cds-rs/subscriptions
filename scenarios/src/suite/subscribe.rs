//! `subscribe`, converted to the World/scenario pattern.
//!
//! Each test builds its own `World`, draws Alice (subscriber), the merchant
//! (plan owner), and the occasional counterparty (`bob`, `charlie`) or `sponsor`
//! from the cast, stages a plan through `setup_plan`, then performs the
//! subscribe action through the observed `send_*`. Every send renders its surface
//! into the test's report under `target/md-reports/`.

use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use solana_signer::Signer;

use testsvm::TestSVM;

use crate::{
    event_engine::event_authority_pda,
    instructions::subscribe,
    state::{Plan, PlanStatus, SubscriptionAuthority, SubscriptionDelegation},
    tests::{
        constants::{PROGRAM_ID, SYSTEM_PROGRAM_ID},
        pda::{get_plan_pda, get_subscription_authority_pda, get_subscription_pda},
        utils::{as_pubkey,
            days, CloseSubscriptionAuthority, CreatePlan, ObservedResultExt, Subscribe,
            UpdatePlan, World,
        },
    },
    AccountDiscriminator, SubscriptionsError,
};

/// Stage a merchant's plan and Alice's authority on `world`: Alice (subscriber)
/// with her ATA and SubscriptionAuthority, plus the merchant's plan (id 1, 50
/// tokens, the given period and end). Mirrors the file's old `setup_plan`, but
/// every send is observed. Returns the cast and the derived plan accounts.
fn setup_plan<B: TestSVM>(
    world: &mut World<B>,
    period_hours: u64,
    end_ts: i64,
) -> (
    solana_keypair::Keypair, // alice (subscriber)
    solana_keypair::Keypair, // merchant
    Pubkey,                  // mint
    Pubkey,                  // plan_pda
    u8,                      // plan_bump
) {
    let alice = world.actor("alice");
    let merchant = world.actor("merchant");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _alice_ata = world.fund_ata(mint, &alice, 100_000_000);

    // Initialize subscription_authority for alice
    world.md().step("Stage: Alice's authority and the merchant's plan");
    world.init_authority(&alice, mint, None).0.assert_ok();

    // Create plan
    let plan_ix = {
        CreatePlan::new(world.svm_mut(), &merchant, mint)
            .plan_id(1)
            .amount(50_000_000)
            .period_hours(period_hours)
            .end_ts(end_ts)
            .instruction()
    };
    let (plan_pda, plan_bump) = get_plan_pda(&merchant.pubkey(), 1);
    world.prop(plan_pda, "Plan");
    world.send_ok(&[plan_ix], &[&merchant], "CreatePlan");

    (alice, merchant, mint, plan_pda, plan_bump)
}

pub fn subscribe_happy_path<B: TestSVM>(backend: B) {
    let mut world = World::new(backend, "Subscribe (happy path)", "Alice subscribes to the merchant's active plan");
    let end_ts = world.now() + days(30) as i64;
    let (alice, merchant, mint, plan_pda, plan_bump) = setup_plan(&mut world, 1, end_ts);

    let sub_ix = { Subscribe::new(world.svm_mut(), &alice, merchant.pubkey(), plan_pda, 1, plan_bump, mint).instruction() };
    let (subscription_pda, _) = get_subscription_pda(&plan_pda, &alice.pubkey());
    world.prop(subscription_pda, "Subscription");
    world.md().step("Alice subscribes to the plan");
    world.send_ok(&[sub_ix], &[&alice], "Subscribe");

    // Verify subscription state
    let sub_account = world.svm().get_account(&subscription_pda).unwrap();
    world.md().check("the subscription account is sized correctly", SubscriptionDelegation::LEN, sub_account.data.len());

    let sub = SubscriptionDelegation::load(&sub_account.data).unwrap();
    world.md().check(
        "the account is tagged SubscriptionDelegation",
        AccountDiscriminator::SubscriptionDelegation as u8,
        sub.header.discriminator,
    );
    world.md().check("the delegator is Alice", alice.pubkey(), as_pubkey(sub.header.delegator.to_bytes()));
    world.md().check("the delegatee is the plan", plan_pda, as_pubkey(sub.header.delegatee.to_bytes()));
    world.md().check("the payer defaults to Alice", alice.pubkey(), as_pubkey(sub.header.payer.to_bytes()));
    let amount_pulled = sub.amount_pulled_in_period;
    let expires_at = sub.expires_at_ts;
    world.md().check("nothing has been pulled yet", 0u64, amount_pulled);
    world.md().check("the subscription has no expiry", 0i64, expires_at);
}

pub fn subscribe_plan_sunset_rejected<B: TestSVM>(backend: B) {
    let mut world = World::new(backend, "Subscribe rejects a sunset plan", "subscribing to a plan in Sunset status is refused");
    let end_ts = world.now() + days(30) as i64;
    let (alice, merchant, mint, plan_pda, plan_bump) = setup_plan(&mut world, 1, end_ts);

    let update_ix =
        { UpdatePlan::new(world.svm_mut(), &merchant, plan_pda).status(PlanStatus::Sunset).end_ts(end_ts).instruction() };
    world.md().step("The merchant sunsets the plan");
    world.send_ok(&[update_ix], &[&merchant], "UpdatePlan (Sunset)");

    let sub_ix = { Subscribe::new(world.svm_mut(), &alice, merchant.pubkey(), plan_pda, 1, plan_bump, mint).instruction() };
    world.md().step("Alice tries to subscribe to the sunset plan");
    world.send_err(&[sub_ix], &[&alice], "Subscribe (sunset)", SubscriptionsError::PlanSunset);
}

pub fn subscribe_plan_expired_rejected<B: TestSVM>(backend: B) {
    let mut world = World::new(backend, "Subscribe rejects an expired plan", "subscribing after the plan's end_ts is refused");
    let end_ts = world.now() + days(2) as i64;
    let (alice, merchant, mint, plan_pda, plan_bump) = setup_plan(&mut world, 1, end_ts);

    world.md().step("Time advances past the plan's end");
    world.warp(days(3));

    let sub_ix = { Subscribe::new(world.svm_mut(), &alice, merchant.pubkey(), plan_pda, 1, plan_bump, mint).instruction() };
    world.md().step("Alice tries to subscribe to the expired plan");
    world.send_err(&[sub_ix], &[&alice], "Subscribe (expired)", SubscriptionsError::PlanExpired);
}

pub fn subscribe_mint_mismatch_rejected<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Subscribe rejects a mint mismatch",
        "subscribing with an authority over a different mint is refused",
    );
    let end_ts = world.now() + days(30) as i64;
    let (alice, merchant, _mint, plan_pda, plan_bump) = setup_plan(&mut world, 1, end_ts);

    // Create a different mint and subscription_authority for it
    let other_mint =
        world.usdc_mint(&alice);
    world.prop(other_mint, "other mint");
    let _other_ata = world.fund_ata(other_mint, &alice, 100_000_000);
    world.md().step("Alice initializes an authority over a different mint");
    world.init_authority(&alice, other_mint, None).0.assert_ok();

    let sub_ix =
        { Subscribe::new(world.svm_mut(), &alice, merchant.pubkey(), plan_pda, 1, plan_bump, other_mint).instruction() };
    world.md().step("Alice subscribes pointing at the wrong mint's authority");
    world.send_err(&[sub_ix], &[&alice], "Subscribe (mint mismatch)", SubscriptionsError::MintMismatch);
}

pub fn subscribe_non_subscriber_subscription_authority_rejected<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Subscribe by a second user",
        "a different user with their own authority subscribes to the same plan",
    );
    let end_ts = world.now() + days(30) as i64;
    let (_alice, merchant, mint, plan_pda, plan_bump) = setup_plan(&mut world, 1, end_ts);

    // Create another user (bob) with their own subscription_authority
    let bob = world.actor("bob");
    let _bob_ata = world.fund_ata(mint, &bob, 100_000_000);
    world.md().step("Bob initializes his own authority");
    world.init_authority(&bob, mint, None).0.assert_ok();

    // Bob subscribes normally with his own authority; this should succeed.
    let sub_ix = { Subscribe::new(world.svm_mut(), &bob, merchant.pubkey(), plan_pda, 1, plan_bump, mint).instruction() };
    world.md().step("Bob subscribes with his own authority");
    world.send_ok(&[sub_ix], &[&bob], "Subscribe (bob)");
}

pub fn subscribe_no_subscription_authority_rejected<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Subscribe without an authority",
        "subscribing with no SubscriptionAuthority PDA in place is refused",
    );
    let end_ts = world.now() + days(30) as i64;
    let (_alice, merchant, mint, plan_pda, plan_bump) = setup_plan(&mut world, 1, end_ts);

    // Charlie has no subscription_authority.
    let charlie = world.actor("charlie");
    let _charlie_ata = world.fund_ata(mint, &charlie, 100_000_000);

    let sub_ix =
        { Subscribe::new(world.svm_mut(), &charlie, merchant.pubkey(), plan_pda, 1, plan_bump, mint).instruction() };
    world.md().step("Charlie subscribes with no authority PDA in place");
    // Should fail because subscription_authority PDA doesn't exist (not owned by program).
    world.send_err(&[sub_ix], &[&charlie], "Subscribe (no authority)", SubscriptionsError::InvalidSubscriptionAuthorityPda);
}

pub fn subscribe_with_sponsor<B: TestSVM>(backend: B) {
    let mut world = World::new(backend, "Subscribe with a sponsor", "a sponsor pays rent and fee; Alice's lamports stay untouched");
    let end_ts = world.now() + days(30) as i64;
    let (alice, merchant, mint, plan_pda, plan_bump) = setup_plan(&mut world, 1, end_ts);
    let sponsor = world.actor("sponsor");

    let alice_balance_before = world.svm().get_account(&alice.pubkey()).unwrap().lamports;
    let sponsor_balance_before = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;

    let sub_ix = {
        Subscribe::new(world.svm_mut(), &alice, merchant.pubkey(), plan_pda, 1, plan_bump, mint).payer(&sponsor).instruction()
    };
    world.md().step("Alice subscribes; the sponsor pays");
    world.send_ok(&[sub_ix], &[&sponsor, &alice], "Subscribe (sponsored)");

    // Subscriber must not be charged.
    let alice_balance_after = world.svm().get_account(&alice.pubkey()).unwrap().lamports;
    world.md().check("Alice's lamports are untouched", alice_balance_before, alice_balance_after);
    let sponsor_balance_after = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;
    world.md().check("the sponsor was charged", true, sponsor_balance_after < sponsor_balance_before);

    // header.payer should be sponsor.
    let (subscription_pda, _) = get_subscription_pda(&plan_pda, &alice.pubkey());
    let sub_account = world.svm().get_account(&subscription_pda).unwrap();
    let sub = SubscriptionDelegation::load(&sub_account.data).unwrap();
    world.md().check("the payer is the sponsor", sponsor.pubkey(), as_pubkey(sub.header.payer.to_bytes()));
    world.md().check("the delegator is still Alice", alice.pubkey(), as_pubkey(sub.header.delegator.to_bytes()));
}

pub fn subscribe_duplicate_rejected<B: TestSVM>(backend: B) {
    let mut world = World::new(backend, "Subscribe rejects a duplicate", "subscribing twice to the same plan is refused");
    let end_ts = world.now() + days(30) as i64;
    let (alice, merchant, mint, plan_pda, plan_bump) = setup_plan(&mut world, 1, end_ts);

    // First subscription should succeed
    let first_ix =
        { Subscribe::new(world.svm_mut(), &alice, merchant.pubkey(), plan_pda, 1, plan_bump, mint).instruction() };
    world.md().step("Alice subscribes once");
    world.send_ok(&[first_ix], &[&alice], "Subscribe (first)");

    // Second subscription to same plan should fail (PDA already exists)
    let second_ix =
        { Subscribe::new(world.svm_mut(), &alice, merchant.pubkey(), plan_pda, 1, plan_bump, mint).instruction() };
    world.md().step("Alice subscribes a second time to the same plan");
    world.send_err(&[second_ix], &[&alice], "Subscribe (duplicate)", SubscriptionsError::AlreadySubscribed);
}

pub fn subscribe_rejects_stale_subscription_authority_generation<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Subscribe rejects a stale authority generation",
        "a subscribe carrying a closed authority's init_id is refused",
    );
    let end_ts = world.now() + days(30) as i64;
    let (alice, merchant, mint, plan_pda, plan_bump) = setup_plan(&mut world, 1, end_ts);

    let plan_account = world.svm().get_account(&plan_pda).unwrap();
    let plan = Plan::load(&plan_account.data).unwrap();
    let live_amount = plan.data.terms.amount;
    let live_period_hours = plan.data.terms.period_hours;
    let live_created_at = plan.data.terms.created_at;
    let live_mint = plan.data.mint;

    let (subscription_authority_pda, _) = get_subscription_authority_pda(&alice.pubkey(), &mint);
    let authority_before_account = world.svm().get_account(&subscription_authority_pda).unwrap();
    let authority_before = SubscriptionAuthority::load(&authority_before_account.data).unwrap();
    let stale_init_id = authority_before.init_id;

    let close_ix = { CloseSubscriptionAuthority::new(world.svm_mut(), &alice, mint).instruction() };
    world.md().step("Alice closes her authority, then re-initializes it (new generation)");
    world.send_ok(&[close_ix], &[&alice], "CloseSubscriptionAuthority");
    world.warp(1);
    world.init_authority(&alice, mint, None).0.assert_ok();

    let authority_after_account = world.svm().get_account(&subscription_authority_pda).unwrap();
    let authority_after = SubscriptionAuthority::load(&authority_after_account.data).unwrap();
    let new_init_id = authority_after.init_id;
    assert_ne!(new_init_id, stale_init_id);

    let (subscription_pda, _) = get_subscription_pda(&plan_pda, &alice.pubkey());
    let event_authority = Pubkey::new_from_array(event_authority_pda::ID.to_bytes());

    let accounts = vec![
        AccountMeta::new(alice.pubkey(), true),
        AccountMeta::new_readonly(merchant.pubkey(), false),
        AccountMeta::new_readonly(plan_pda, false),
        AccountMeta::new(subscription_pda, false),
        AccountMeta::new_readonly(subscription_authority_pda, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        AccountMeta::new_readonly(event_authority, false),
        AccountMeta::new_readonly(PROGRAM_ID, false),
    ];

    let data = [
        vec![*subscribe::DISCRIMINATOR],
        1u64.to_le_bytes().to_vec(),
        vec![plan_bump],
        live_mint.as_ref().to_vec(),
        live_amount.to_le_bytes().to_vec(),
        live_period_hours.to_le_bytes().to_vec(),
        live_created_at.to_le_bytes().to_vec(),
        stale_init_id.to_le_bytes().to_vec(),
    ]
    .concat();

    let ix = Instruction { program_id: PROGRAM_ID, accounts, data };

    world.md().step("Alice subscribes carrying the stale (closed) authority init_id");
    world.send_err(&[ix], &[&alice], "Subscribe (stale authority)", SubscriptionsError::StaleSubscriptionAuthority);
}

pub fn subscribe_rejects_stale_expected_terms<B: TestSVM>(backend: B) {
    let mut world = World::new(backend,
        "Subscribe rejects stale expected terms",
        "a subscribe carrying a stale expected_amount is refused",
    );
    let end_ts = world.now() + days(30) as i64;
    let (alice, merchant, mint, plan_pda, plan_bump) = setup_plan(&mut world, 1, end_ts);

    // Snapshot live terms, then submit subscribe with a stale `expected_amount`.
    let plan_account = world.svm().get_account(&plan_pda).unwrap();
    let plan = Plan::load(&plan_account.data).unwrap();
    let live_amount = plan.data.terms.amount;
    let stale_amount = live_amount.wrapping_add(1);
    let live_period_hours = plan.data.terms.period_hours;
    let live_created_at = plan.data.terms.created_at;
    let live_mint = plan.data.mint;

    let (subscription_authority_pda, _) = get_subscription_authority_pda(&alice.pubkey(), &mint);
    let subscription_authority_account = world.svm().get_account(&subscription_authority_pda).unwrap();
    let subscription_authority = SubscriptionAuthority::load(&subscription_authority_account.data).unwrap();
    let live_subscription_authority_init_id = subscription_authority.init_id;
    let (subscription_pda, _) = get_subscription_pda(&plan_pda, &alice.pubkey());
    let event_authority = Pubkey::new_from_array(event_authority_pda::ID.to_bytes());

    let accounts = vec![
        AccountMeta::new(alice.pubkey(), true),
        AccountMeta::new_readonly(merchant.pubkey(), false),
        AccountMeta::new_readonly(plan_pda, false),
        AccountMeta::new(subscription_pda, false),
        AccountMeta::new_readonly(subscription_authority_pda, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        AccountMeta::new_readonly(event_authority, false),
        AccountMeta::new_readonly(PROGRAM_ID, false),
    ];

    let data = [
        vec![*subscribe::DISCRIMINATOR],
        1u64.to_le_bytes().to_vec(),
        vec![plan_bump],
        live_mint.as_ref().to_vec(),
        stale_amount.to_le_bytes().to_vec(),
        live_period_hours.to_le_bytes().to_vec(),
        live_created_at.to_le_bytes().to_vec(),
        live_subscription_authority_init_id.to_le_bytes().to_vec(),
    ]
    .concat();

    let ix = Instruction { program_id: PROGRAM_ID, accounts, data };

    world.md().step("Alice subscribes carrying a stale expected_amount");
    world.send_err(&[ix], &[&alice], "Subscribe (stale terms)", SubscriptionsError::PlanTermsMismatch);
}
