//! `revoke_abandoned_delegation`, converted to the World/scenario pattern.
//!
//! Each test builds a `World`, draws `alice` (the delegator and mint owner) and
//! `sponsor` (the rent/fee payer) from the cast, stages a delegation through the
//! observed sends, then sweeps it back with the `RevokeAbandonedDelegation`
//! verb. The `delegatee` is just a destination pubkey, so it stays a prop.

use solana_pubkey::Pubkey;
use solana_signer::Signer;

use litesvm_utils::TestSVM;

use crate::{
    tests::utils::{
            hours, CloseSubscriptionAuthority, CreateDelegation, ObservedResultExt,
            RevokeAbandonedDelegation, make_backend, World,
        },
    SubscriptionsError,
};

const NO_EXPIRY: i64 = 0;

#[test]
fn sponsor_recovers_no_expiry_fixed_delegation_after_authority_closed() {
    let mut world = World::new(make_backend(), 
        "Sponsor recovers a no-expiry fixed delegation after the authority is closed",
        "Alice closes her authority; the sponsor sweeps the abandoned fixed delegation and recovers its rent",
    );
    let alice = world.actor("alice");
    let sponsor = world.actor("sponsor");
    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);
    world.init_authority(&alice, mint, None).0.assert_ok();

    world.md().step("Alice (with the sponsor paying rent) creates a no-expiry fixed delegation");
    let (delegation_ix, delegation_pda) = {
        CreateDelegation::new(world.svm_mut(), &alice, mint, delegatee).payer(&sponsor).fixed_ix(100, NO_EXPIRY)
    };
    world.prop(delegation_pda, "Delegation");
    world.send_ok(&[delegation_ix], &[&sponsor, &alice], "CreateFixedDelegation");

    let delegation_rent = world.svm().get_account(&delegation_pda).unwrap().lamports;

    world.md().step("Alice closes her subscription authority, abandoning the delegation");
    let close_ix = CloseSubscriptionAuthority::new(world.svm_mut(), &alice, mint).instruction();
    world.send_ok(&[close_ix], &[&alice], "CloseSubscriptionAuthority");

    let sponsor_before = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;

    world.md().step("The sponsor sweeps the abandoned delegation and recovers its rent");
    let revoke_ix = RevokeAbandonedDelegation::new(world.svm_mut(), &sponsor, alice.pubkey(), mint, delegatee).instruction();
    world.send_ok(&[revoke_ix], &[&sponsor], "RevokeAbandonedDelegation");

    let delegation_after = world.svm().get_account(&delegation_pda);
    let delegation_gone =
        delegation_after.is_none() || delegation_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0;
    world.md().check("the delegation account is gone", true, delegation_gone);

    let sponsor_after = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;
    world.md().check(
        "the sponsor recovered the delegation rent (net of fees)",
        true,
        sponsor_after >= sponsor_before + delegation_rent - 10_000,
    );
}

#[test]
fn payer_recovers_no_expiry_recurring_delegation_after_authority_closed() {
    let mut world = World::new(make_backend(), 
        "Payer recovers a no-expiry recurring delegation after the authority is closed",
        "Alice closes her authority; the sponsor sweeps the abandoned recurring delegation and recovers its rent",
    );
    let alice = world.actor("alice");
    let sponsor = world.actor("sponsor");
    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);
    world.init_authority(&alice, mint, None).0.assert_ok();

    world.md().step("Alice (with the sponsor paying rent) creates a no-expiry recurring delegation");
    let now = world.now();
    let (delegation_ix, delegation_pda) = {
        CreateDelegation::new(world.svm_mut(), &alice, mint, delegatee).payer(&sponsor).recurring_ix(
            100,
            hours(1),
            now,
            NO_EXPIRY,
        )
    };
    world.prop(delegation_pda, "Delegation");
    world.send_ok(&[delegation_ix], &[&sponsor, &alice], "CreateRecurringDelegation");

    let delegation_rent = world.svm().get_account(&delegation_pda).unwrap().lamports;

    world.md().step("Alice closes her subscription authority, abandoning the delegation");
    let close_ix = CloseSubscriptionAuthority::new(world.svm_mut(), &alice, mint).instruction();
    world.send_ok(&[close_ix], &[&alice], "CloseSubscriptionAuthority");

    let sponsor_before = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;

    world.md().step("The sponsor sweeps the abandoned delegation and recovers its rent");
    let revoke_ix = RevokeAbandonedDelegation::new(world.svm_mut(), &sponsor, alice.pubkey(), mint, delegatee).instruction();
    world.send_ok(&[revoke_ix], &[&sponsor], "RevokeAbandonedDelegation");

    let delegation_after = world.svm().get_account(&delegation_pda);
    let delegation_gone =
        delegation_after.is_none() || delegation_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0;
    world.md().check("the delegation account is gone", true, delegation_gone);

    let sponsor_after = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;
    world.md().check(
        "the sponsor recovered the delegation rent (net of fees)",
        true,
        sponsor_after >= sponsor_before + delegation_rent - 10_000,
    );
}

#[test]
fn payer_recovers_delegation_after_authority_reinit_bumps_init_id() {
    let mut world = World::new(make_backend(), 
        "Payer recovers a delegation after the authority is re-initialized",
        "re-initializing the authority bumps its init_id, so the old delegation is stale and the sponsor can sweep it",
    );
    let alice = world.actor("alice");
    let sponsor = world.actor("sponsor");
    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);
    world.init_authority(&alice, mint, None).0.assert_ok();

    world.md().step("Alice (with the sponsor paying rent) creates a fixed delegation");
    let (delegation_ix, delegation_pda) = {
        CreateDelegation::new(world.svm_mut(), &alice, mint, delegatee).payer(&sponsor).fixed_ix(100, NO_EXPIRY)
    };
    world.prop(delegation_pda, "Delegation");
    world.send_ok(&[delegation_ix], &[&sponsor, &alice], "CreateFixedDelegation");

    world.md().step("Alice closes her authority, then re-initializes it (bumping init_id)");
    let close_ix = CloseSubscriptionAuthority::new(world.svm_mut(), &alice, mint).instruction();
    world.send_ok(&[close_ix], &[&alice], "CloseSubscriptionAuthority");

    world.warp(10);
    world.init_authority(&alice, mint, None).0.assert_ok();

    world.md().step("The sponsor sweeps the now-stale delegation");
    let revoke_ix = RevokeAbandonedDelegation::new(world.svm_mut(), &sponsor, alice.pubkey(), mint, delegatee).instruction();
    world.send_ok(&[revoke_ix], &[&sponsor], "RevokeAbandonedDelegation");

    let delegation_after = world.svm().get_account(&delegation_pda);
    let delegation_gone =
        delegation_after.is_none() || delegation_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0;
    world.md().check("the delegation account is gone", true, delegation_gone);
}

#[test]
fn revoke_abandoned_rejects_live_delegation() {
    let mut world = World::new(make_backend(), 
        "Revoke-abandoned rejects a live delegation",
        "while the authority is open the delegation is live, so the sweep is unauthorized",
    );
    let alice = world.actor("alice");
    let sponsor = world.actor("sponsor");
    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);
    world.init_authority(&alice, mint, None).0.assert_ok();

    world.md().step("Alice (with the sponsor paying rent) creates a fixed delegation");
    let (delegation_ix, delegation_pda) = {
        CreateDelegation::new(world.svm_mut(), &alice, mint, delegatee).payer(&sponsor).fixed_ix(100, NO_EXPIRY)
    };
    world.prop(delegation_pda, "Delegation");
    world.send_ok(&[delegation_ix], &[&sponsor, &alice], "CreateFixedDelegation");

    world.md().step("The sponsor tries to sweep the still-live delegation");
    let revoke_ix = RevokeAbandonedDelegation::new(world.svm_mut(), &sponsor, alice.pubkey(), mint, delegatee).instruction();
    world.send_err(&[revoke_ix], &[&sponsor], "RevokeAbandonedDelegation (live)", SubscriptionsError::Unauthorized);
}

#[test]
fn revoke_abandoned_rejects_non_sponsor_caller() {
    let mut world = World::new(make_backend(), 
        "Revoke-abandoned rejects a non-sponsor caller",
        "only the recorded payer can sweep an abandoned delegation; a stranger is rejected",
    );
    let alice = world.actor("alice");
    let sponsor = world.actor("sponsor");
    let stranger = world.actor("mallory");
    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);
    world.init_authority(&alice, mint, None).0.assert_ok();

    world.md().step("Alice (with the sponsor paying rent) creates a fixed delegation");
    let (delegation_ix, delegation_pda) = {
        CreateDelegation::new(world.svm_mut(), &alice, mint, delegatee).payer(&sponsor).fixed_ix(100, NO_EXPIRY)
    };
    world.prop(delegation_pda, "Delegation");
    world.send_ok(&[delegation_ix], &[&sponsor, &alice], "CreateFixedDelegation");

    world.md().step("Alice closes her authority, abandoning the delegation");
    let close_ix = CloseSubscriptionAuthority::new(world.svm_mut(), &alice, mint).instruction();
    world.send_ok(&[close_ix], &[&alice], "CloseSubscriptionAuthority");

    world.md().step("Mallory (not the recorded payer) tries to sweep the abandoned delegation");
    let revoke_ix = RevokeAbandonedDelegation::new(world.svm_mut(), &stranger, alice.pubkey(), mint, delegatee).instruction();
    world.send_err(
        &[revoke_ix],
        &[&stranger],
        "RevokeAbandonedDelegation (non-sponsor caller)",
        SubscriptionsError::Unauthorized,
    );
}

#[test]
fn revoke_abandoned_rejects_unbound_authority_account() {
    let mut world = World::new(make_backend(), 
        "Revoke-abandoned rejects an unbound authority account",
        "passing an authority account the delegation does not derive from is rejected",
    );
    let alice = world.actor("alice");
    let sponsor = world.actor("sponsor");
    let delegatee = Pubkey::new_unique();
    world.prop(delegatee, "delegatee");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);
    world.init_authority(&alice, mint, None).0.assert_ok();

    world.md().step("Alice (with the sponsor paying rent) creates a fixed delegation");
    let (delegation_ix, delegation_pda) = {
        CreateDelegation::new(world.svm_mut(), &alice, mint, delegatee).payer(&sponsor).fixed_ix(100, NO_EXPIRY)
    };
    world.prop(delegation_pda, "Delegation");
    world.send_ok(&[delegation_ix], &[&sponsor, &alice], "CreateFixedDelegation");

    world.md().step("Alice closes her authority, abandoning the delegation");
    let close_ix = CloseSubscriptionAuthority::new(world.svm_mut(), &alice, mint).instruction();
    world.send_ok(&[close_ix], &[&alice], "CloseSubscriptionAuthority");

    world.md().step("The sponsor sweeps but points at an unrelated authority account");
    let revoke_ix = RevokeAbandonedDelegation::new(world.svm_mut(), &sponsor, alice.pubkey(), mint, delegatee)
        .authority(Pubkey::new_unique())
        .instruction();
    world.send_err(
        &[revoke_ix],
        &[&sponsor],
        "RevokeAbandonedDelegation (unbound authority)",
        SubscriptionsError::Unauthorized,
    );
}
