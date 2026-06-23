//! `revoke_subscription_authority`, converted to the World/scenario pattern.
//!
//! Each test builds a `World`, draws `alice` (the principal: mint owner,
//! delegator) from the cast, fabricates her mint and ATA, initializes her
//! authority through the `init_authority` verb, then performs the revoke (and,
//! where relevant, the close) through the observed `send_*`. Every send renders
//! its surface into the test's report under `target/md-reports/`.

use solana_signer::Signer;
use spl_token_2022_interface::state::Account as TokenAccount;

use crate::{
    tests::{
        constants::{MINT_DECIMALS, TOKEN_2022_PROGRAM_ID},
        utils::{
            fetch_account, init_mint, CloseSubscriptionAuthority, ObservedResultExt,
            RevokeSubscriptionAuthority, make_backend, World,
        },
    },
    SubscriptionsError,
};

#[test]
fn revoke_subscription_authority_clears_delegate() {
    let mut world = World::new(make_backend(), 
        "Revoke clears the delegate",
        "Alice revokes her subscription authority; her ATA's delegate is cleared",
    );
    let user = world.actor("alice");

    let mint = world.usdc_mint(&user);
    world.prop(mint, "USDC mint");
    let user_ata = world.fund_ata(mint, &user, 1_000_000);

    world.md().step("Alice initializes her subscription authority");
    world.init_authority(&user, mint, None).0.assert_ok();

    let before = fetch_account::<TokenAccount, _>(world.svm(), &user_ata);
    assert!(before.delegate.is_some());
    world.md().check("the ATA is delegated for the full amount before revoke", u64::MAX, before.delegated_amount);

    world.md().step("Alice revokes her subscription authority");
    let ix = RevokeSubscriptionAuthority::new(world.svm_mut(), &user, mint).instruction();
    world.send_ok(&[ix], &[&user], "RevokeSubscriptionAuthority");

    let after = fetch_account::<TokenAccount, _>(world.svm(), &user_ata);
    world.md().check("the delegate is cleared after revoke", true, after.delegate.is_none());
    world.md().check("the delegated amount is zeroed after revoke", 0, after.delegated_amount);
}

#[test]
fn revoke_subscription_authority_clears_delegate_token_2022() {
    let mut world = World::new(make_backend(), 
        "Revoke clears the delegate (Token-2022)",
        "Alice revokes her authority over a Token-2022 mint; her ATA's delegate is cleared",
    );
    let user = world.actor("alice");

    let mint = init_mint(world.svm_mut(), TOKEN_2022_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, Some(user.pubkey()), &[]);
    world.prop(mint, "USDC mint (T22)");
    let user_ata = world.fund_ata(mint, &user, 1_000_000);

    world.md().step("Alice initializes her subscription authority");
    world.init_authority(&user, mint, None).0.assert_ok();

    let before = fetch_account::<TokenAccount, _>(world.svm(), &user_ata);
    assert!(before.delegate.is_some());
    world.md().check("the ATA is delegated for the full amount before revoke", u64::MAX, before.delegated_amount);

    world.md().step("Alice revokes her subscription authority");
    let ix = RevokeSubscriptionAuthority::new(world.svm_mut(), &user, mint).instruction();
    world.send_ok(&[ix], &[&user], "RevokeSubscriptionAuthority");

    let after = fetch_account::<TokenAccount, _>(world.svm(), &user_ata);
    world.md().check("the delegate is cleared after revoke", true, after.delegate.is_none());
    world.md().check("the delegated amount is zeroed after revoke", 0, after.delegated_amount);
}

#[test]
fn revoke_subscription_authority_works_after_close() {
    let mut world = World::new(make_backend(), 
        "Revoke works after close",
        "Alice closes her authority leaving a dangling delegate; revoke still clears it",
    );
    let user = world.actor("alice");

    let mint = world.usdc_mint(&user);
    world.prop(mint, "USDC mint");
    let user_ata = world.fund_ata(mint, &user, 1_000_000);

    world.md().step("Alice initializes her authority, then closes it");
    world.init_authority(&user, mint, None).0.assert_ok();

    {
        let ix = CloseSubscriptionAuthority::new(world.svm_mut(), &user, mint).instruction();
        world.send_ok(&[ix], &[&user], "CloseSubscriptionAuthority");
    }

    // Closing the authority leaves the ATA delegation dangling.
    let dangling = fetch_account::<TokenAccount, _>(world.svm(), &user_ata);
    assert!(dangling.delegate.is_some());
    world.md().check("the delegate dangles after close", u64::MAX, dangling.delegated_amount);

    world.md().step("Alice revokes the dangling delegate after the authority is closed");
    let ix = RevokeSubscriptionAuthority::new(world.svm_mut(), &user, mint).instruction();
    world.send_ok(&[ix], &[&user], "RevokeSubscriptionAuthority");

    let after = fetch_account::<TokenAccount, _>(world.svm(), &user_ata);
    world.md().check(
        "revoke clears the dangling delegate even after the authority is closed",
        true,
        after.delegate.is_none(),
    );
    world.md().check("the delegated amount is zeroed", 0, after.delegated_amount);
}

#[test]
fn revoke_subscription_authority_rejects_ata_mint_mismatch() {
    let mut world = World::new(make_backend(), 
        "Reject an ATA / mint mismatch",
        "revoke is refused when the passed ATA belongs to a different mint than the instruction's mint",
    );
    let user = world.actor("alice");

    let mint_a = world.usdc_mint(&user);
    world.prop(mint_a, "USDC mint A");
    let ata_a = world.fund_ata(mint_a, &user, 1_000_000);
    world.prop(ata_a, "Alice's ATA (mint A)");
    let mint_b = world.usdc_mint(&user);
    world.prop(mint_b, "USDC mint B");

    world.md().step("Alice initializes her authority over mint A");
    world.init_authority(&user, mint_a, None).0.assert_ok();

    world.md().step("Alice revokes mint B but passes mint A's ATA");
    let ix = RevokeSubscriptionAuthority::new(world.svm_mut(), &user, mint_b).ata(ata_a).instruction();
    world.send_err(&[ix], &[&user], "RevokeSubscriptionAuthority (ATA/mint mismatch)", SubscriptionsError::MintMismatch);
}
