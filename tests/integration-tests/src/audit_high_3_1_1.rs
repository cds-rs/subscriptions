//! Cantina HIGH finding 3.1.1, reproduced in the DSL.
//!
//! "Missing start time validation allows recurring transfers before the
//! delegation period begins." On this `dsl-audit` branch the PR6 guard is
//! reverted, so a delegatee can pull the full per-period budget BEFORE
//! `start_ts`. The report this test writes shows the transfer reaching the
//! delegatee a full day early. On the fixed code the same pull is refused with
//! `DelegationNotStarted`, so this test (which asserts the exploit succeeds)
//! fails there: that failure is the regression proof the fix holds.

use solana_signer::Signer;

use crate::tests::utils::{
    days, get_ata_balance, hours, CreateDelegation, ObservedResultExt, TransferDelegation, World,
};

#[test]
fn finding_3_1_1_recurring_pull_before_start_ts() {
    let mut world = World::new(
        "AUDIT 3.1.1: a recurring pull lands before the delegation starts",
        "a delegatee pulls the per-period budget a day before start_ts (Cantina HIGH 3.1.1)",
    );
    let alice = world.actor("alice"); // delegator
    let bob = world.actor("bob"); // delegatee (the attacker pulling early)

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    world.fund_ata(mint, &alice, 100_000_000);
    let bob_ata = world.fund_ata(mint, &bob, 0);
    world.init_authority(&alice, mint, None).0.assert_ok();

    // 50 tokens/hour, but the delegation does not start until TOMORROW.
    let amount_per_period: u64 = 50_000_000;
    let start_ts = world.now() + days(1) as i64;
    let expiry_ts = world.now() + days(7) as i64;
    world.md().step("Alice grants Bob a recurring allowance that only starts tomorrow");
    let (create_ix, delegation_pda) = CreateDelegation::new(world.svm_mut(), &alice, mint, bob.pubkey())
        .nonce(0)
        .recurring_ix(amount_per_period, hours(1), start_ts, expiry_ts);
    world.prop(delegation_pda, "Delegation (starts tomorrow)");
    world.send_ok(&[create_ix], &[&alice], "CreateRecurringDelegation");

    let bob_before = get_ata_balance(world.svm(), &bob_ata);

    // The exploit: pull NOW, a full day before start_ts.
    world.md().step("Bob pulls 10 tokens NOW — a day before the delegation begins");
    let pull_ix = TransferDelegation::new(world.svm_mut(), &bob, alice.pubkey(), mint, delegation_pda)
        .amount(10_000_000)
        .recurring_ix();
    let res = world.send(&[pull_ix], &[&bob], "TransferRecurring (before start_ts)");
    let exploit_succeeded = res.is_success();
    let bob_after = get_ata_balance(world.svm(), &bob_ata);

    world.md().note(
        "Finding 3.1.1: the delegation's start_ts is a day in the future, yet the pull above lands now. \
         With the PR6 guard reverted on this branch, validate_recurring_transfer floors the pre-start \
         timestamp via saturating_sub to 0, leaving the full per-period budget available immediately. On \
         the fixed code this pull is refused with DelegationNotStarted, and the assertions below fail — \
         that failure is the regression proof that the fix holds.",
    );
    world.md().check("the pre-start pull SUCCEEDS (the vulnerability)", true, exploit_succeeded);
    world.md().check("Bob received tokens before the delegation started", bob_before + 10_000_000, bob_after);
}

