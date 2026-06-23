//! `close_subscription_authority`, converted to the World/scenario pattern.
//!
//! Each test builds a `World`, draws its actors from the cast (`alice` the
//! principal closing her authority, `sponsor` the rent payer, `mallory` the
//! adversary), stages the authority through `init_authority`, and performs the
//! close action through the observed `send_*`. Every send renders its surface into
//! the test's report under `target/md-reports/`.

use solana_signer::Signer;

use litesvm_utils::TestSVM;

use crate::{
    tests::utils::{as_pubkey, CloseSubscriptionAuthority, ObservedResultExt, World},
    SubscriptionAuthority, SubscriptionsError,
};

#[test]
fn close_subscription_authority() {
    let mut world = World::new(
        "Close a subscription authority",
        "Alice closes her SubscriptionAuthority and the rent returns to her",
    );
    let alice = world.actor("alice");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    let (res, subscription_authority_pda, _bump) = world.init_authority(&alice, mint, None);
    res.assert_ok();

    let account_before = world.svm().get_account(&subscription_authority_pda);
    assert!(account_before.is_some());
    let rent = account_before.unwrap().lamports;

    let user_balance_before = world.svm().get_account(&alice.pubkey()).unwrap().lamports;

    world.md().step("Alice closes her subscription authority");
    let ix = CloseSubscriptionAuthority::new(world.svm_mut(), &alice, mint).instruction();
    world.send_ok(&[ix], &[&alice], "CloseSubscriptionAuthority");

    let account_after = world.svm().get_account(&subscription_authority_pda);
    world.md().check(
        "the authority account is gone or emptied",
        true,
        account_after.is_none() || account_after.as_ref().map(|a| a.lamports).unwrap_or(0) == 0,
    );

    let user_balance_after = world.svm().get_account(&alice.pubkey()).unwrap().lamports;
    world.md().check("Alice's balance grew", true, user_balance_after > user_balance_before);
    assert!(user_balance_after >= user_balance_before + rent - 10000);
}

#[test]
fn non_owner_cannot_close() {
    let mut world = World::new(
        "A non-owner cannot close the authority",
        "Mallory cannot close Alice's SubscriptionAuthority",
    );
    let alice = world.actor("alice");
    let mallory = world.actor("mallory");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    let (res, subscription_authority_pda, _bump) = world.init_authority(&alice, mint, None);
    res.assert_ok();

    world.md().step("Mallory attempts to close Alice's authority");
    let ix = CloseSubscriptionAuthority::new(world.svm_mut(), &mallory, mint).pda(subscription_authority_pda).instruction();
    world.send_err(&[ix], &[&mallory], "CloseSubscriptionAuthority (non-owner)", SubscriptionsError::Unauthorized);

    // Account should still exist.
    let account_after = world.svm().get_account(&subscription_authority_pda);
    assert!(account_after.is_some());
    world.md().check(
        "the authority is still funded",
        true,
        account_after.as_ref().map(|a| a.lamports).unwrap_or(0) > 0,
    );
}

#[test]
fn writable_accounts_must_be_writable() {
    use solana_instruction::{AccountMeta, Instruction};

    use crate::{instructions::close_subscription_authority, tests::{constants::PROGRAM_ID, idl}};

    let writable = idl::writable_account_indices("closeSubscriptionAuthority");

    let mut world = World::new(
        "Close: writable accounts must be writable",
        "flipping any account the close writes to read-only is rejected",
    );
    let alice = world.actor("alice");
    let sponsor = world.actor("sponsor");

    let mint = world.usdc_mint(&alice);
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    let (res, subscription_authority_pda, _) = world.init_authority(&alice, mint, None);
    res.assert_ok();

    for (idx, name, is_signer) in &writable {
        let mut accounts =
            vec![AccountMeta::new(alice.pubkey(), true), AccountMeta::new(subscription_authority_pda, false)];

        // Flip writable account to readonly, preserving signer flag.
        let pubkey = accounts[*idx].pubkey;
        accounts[*idx] = AccountMeta::new_readonly(pubkey, *is_signer);

        let ix =
            Instruction { program_id: PROGRAM_ID, accounts, data: vec![*close_subscription_authority::DISCRIMINATOR] };

        world.send_err(
            &[ix],
            &[&sponsor, &alice],
            &format!("CloseSubscriptionAuthority ({name} forced read-only)"),
            SubscriptionsError::AccountNotWritable,
        );
    }
}

#[test]
fn signer_accounts_must_be_signers() {
    use solana_instruction::{AccountMeta, Instruction};

    use crate::{instructions::close_subscription_authority, tests::{constants::PROGRAM_ID, idl}};

    let signers = idl::signer_account_indices("closeSubscriptionAuthority");

    let mut world = World::new(
        "Close: signer accounts must sign",
        "flipping any required signer to non-signer is rejected",
    );
    let alice = world.actor("alice");
    let sponsor = world.actor("sponsor");

    let mint = world.usdc_mint(&alice);
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    let (res, subscription_authority_pda, _) = world.init_authority(&alice, mint, None);
    res.assert_ok();

    for (idx, name, is_writable) in &signers {
        let mut accounts =
            vec![AccountMeta::new(alice.pubkey(), true), AccountMeta::new(subscription_authority_pda, false)];

        // Flip signer to non-signer, preserving writable flag.
        let pubkey = accounts[*idx].pubkey;
        accounts[*idx] =
            if *is_writable { AccountMeta::new(pubkey, false) } else { AccountMeta::new_readonly(pubkey, false) };

        let ix =
            Instruction { program_id: PROGRAM_ID, accounts, data: vec![*close_subscription_authority::DISCRIMINATOR] };

        world.send_err(
            &[ix],
            &[&sponsor],
            &format!("CloseSubscriptionAuthority ({name} forced non-signer)"),
            SubscriptionsError::NotSigner,
        );
    }
}

#[test]
fn close_returns_rent_to_sponsor() {
    let mut world = World::new(
        "Close returns rent to the sponsor",
        "when a sponsor funded the authority, the close returns rent to them",
    );
    let alice = world.actor("alice");
    let sponsor = world.actor("sponsor");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    let (res, subscription_authority_pda, _bump) = world.init_authority(&alice, mint, Some(&sponsor));
    res.assert_ok();

    // Stored payer should be the sponsor.
    let account = world.svm().get_account(&subscription_authority_pda).unwrap();
    let md = SubscriptionAuthority::load(&account.data).unwrap();
    world.md().check("the stored payer is the sponsor", sponsor.pubkey(), as_pubkey(md.payer.to_bytes()));

    let rent = account.lamports;
    let sponsor_balance_before = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;

    world.md().step("Alice closes, directing rent back to the sponsor");
    let ix = CloseSubscriptionAuthority::new(world.svm_mut(), &alice, mint).receiver(sponsor.pubkey()).instruction();
    world.send_ok(&[ix], &[&alice], "CloseSubscriptionAuthority (rent to sponsor)");

    let sponsor_balance_after = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;
    assert!(sponsor_balance_after >= sponsor_balance_before + rent - 10_000);
}

#[test]
fn close_without_receiver_when_sponsor_funded_fails() {
    let mut world = World::new(
        "Close without a receiver fails when a sponsor funded",
        "a sponsor-funded authority cannot be closed without naming the rent receiver",
    );
    let alice = world.actor("alice");
    let sponsor = world.actor("sponsor");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    world.init_authority(&alice, mint, Some(&sponsor)).0.assert_ok();

    world.md().step("Alice closes without naming the sponsor receiver");
    // No receiver passed -> must fail because stored payer differs from user.
    let ix = CloseSubscriptionAuthority::new(world.svm_mut(), &alice, mint).instruction();
    world.send_err(
        &[ix],
        &[&alice],
        "CloseSubscriptionAuthority (no receiver)",
        SubscriptionsError::NotEnoughAccountKeys,
    );
}

#[test]
fn close_with_wrong_receiver_unauthorized() {
    let mut world = World::new(
        "Close with the wrong receiver is unauthorized",
        "the rent receiver must be the stored payer, not an arbitrary account",
    );
    let alice = world.actor("alice");
    let sponsor = world.actor("sponsor");
    let mallory = world.actor("mallory");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    world.init_authority(&alice, mint, Some(&sponsor)).0.assert_ok();

    world.md().step("Alice closes, but points the rent at Mallory");
    let ix = CloseSubscriptionAuthority::new(world.svm_mut(), &alice, mint).receiver(mallory.pubkey()).instruction();
    world.send_err(
        &[ix],
        &[&alice],
        "CloseSubscriptionAuthority (wrong receiver)",
        SubscriptionsError::Unauthorized,
    );
}

#[test]
fn idempotent_init_preserves_original_payer() {
    let mut world = World::new(
        "Idempotent init preserves the original payer",
        "a second init by a different sponsor leaves the stored payer untouched",
    );
    let alice = world.actor("alice");
    let sponsor_a = world.actor("sponsor");
    let sponsor_b = world.actor("sponsor2");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    // Sponsor A inits.
    world.md().step("Sponsor A initializes the authority");
    let (res, subscription_authority_pda, _) = world.init_authority(&alice, mint, Some(&sponsor_a));
    res.assert_ok();

    // Sponsor B re-runs init.
    world.md().step("Sponsor B re-runs init");
    world.init_authority(&alice, mint, Some(&sponsor_b)).0.assert_ok();

    // Stored payer must remain sponsor A.
    let account = world.svm().get_account(&subscription_authority_pda).unwrap();
    let md = SubscriptionAuthority::load(&account.data).unwrap();
    world.md().check("the stored payer is still sponsor A", sponsor_a.pubkey(), as_pubkey(md.payer.to_bytes()));
}

#[test]
fn closed_account_is_zeroed() {
    let mut world = World::new(
        "A closed authority account is zeroed",
        "after closing, any residual account data is all zeros",
    );
    let alice = world.actor("alice");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let _user_ata = world.fund_ata(mint, &alice, 1_000_000);

    let (res, subscription_authority_pda, _bump) = world.init_authority(&alice, mint, None);
    res.assert_ok();

    world.md().step("Alice closes her authority");
    let ix = CloseSubscriptionAuthority::new(world.svm_mut(), &alice, mint).instruction();
    world.send_ok(&[ix], &[&alice], "CloseSubscriptionAuthority");

    let account_after = world.svm().get_account(&subscription_authority_pda);
    if let Some(account) = account_after {
        assert!(account.data.iter().all(|&byte| byte == 0), "All data should be zeroed after close");
    }
}
