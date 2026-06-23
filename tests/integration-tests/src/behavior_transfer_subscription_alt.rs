//! A doc-mirroring behavior report for the `transfer_subscription` pull.
//!
//! ADR-002 (`docs/002-subscriptions-architecture.md`) draws this instruction as
//! a hand-written `sequenceDiagram` with an `alt` block: one branch where the
//! caller is the plan owner or a whitelisted puller (the pull lands), and an
//! `else` branch where the caller is neither (the program returns
//! `Unauthorized`). This test reproduces *both* branches against the real
//! program, in one `World`, so the single report it writes carries the generated
//! counterpart to that hand-drawn `alt`: the authorized branch renders its full
//! surface (CPI tree, sequence diagram with and without lifelines, authority and
//! ownership graphs), and the refused branch renders the tree that already names
//! the `Unauthorized` error.
//!
//! The point is not another assertion on `transfer_subscription` (the
//! `test_transfer_subscription` suite covers the behavior exhaustively); it is to
//! show that the diagram a human drew to *explain* the instruction falls out of
//! the executor for free, frame by frame, when we run the instruction both ways.

use solana_signer::Signer;
use spl_associated_token_account_interface::address::get_associated_token_address_with_program_id;

use crate::{
    tests::{constants::TOKEN_PROGRAM_ID, utils::{token_balance, TransferSubscription, World}},
    SubscriptionsError,
};

#[test]
fn transfer_subscription_the_authorization_alt() {
    let mut world = World::new(
        "Transfer subscription (pull): the authorization alt",
        "the same pull, run by an authorized caller (it lands) and an unauthorized one (Unauthorized), \
         mirroring the alt block in the ADR-002 sequence diagram",
    );

    // One staged precondition for both branches: Alice subscribed to the
    // merchant's plan (50 tokens/hour, ending in 30 days), her authority live.
    let staged = world.stage_subscription();
    let alice = staged.alice;
    let merchant = staged.merchant;
    let mint = staged.mint;
    let plan_pda = staged.plan_pda;
    let subscription_pda = staged.subscription_pda;

    // The merchant's receiving ATA: the default destination for the merchant's
    // own pull, and the concrete receiver Mallory will aim at in the else branch.
    let merchant_ata = world.fund_ata(mint, &merchant, 0);

    // Alias the two token accounts the pull touches, so the authority and
    // ownership graphs read in roles rather than raw ATA addresses.
    let alice_ata = get_associated_token_address_with_program_id(&alice.pubkey(), &mint, &TOKEN_PROGRAM_ID);
    world.prop(alice_ata, "Alice ATA");
    world.prop(merchant_ata, "merchant ATA");

    world.md().note(
        "ADR-002 draws `transfer_subscription` as a sequence diagram whose `alt` splits on the \
         authorization check: **`alt` Caller is owner or in pullers** versus **`else` Caller not \
         authorized**. The two sections below run exactly those two branches against the program; the \
         rendered sequence diagrams and authority graphs are the generated mirror of that hand-drawn alt.",
    );

    // --- alt: Caller is owner or in pullers -------------------------------
    world.md().step("alt — the caller is the plan owner (authorized): the pull lands");
    world.md().note(
        "The merchant owns the plan, so authorization passes. The pull moves 10 tokens through the \
         subscription authority's signed token transfer and emits the recurring-transfer event. The \
         authority graph shows the merchant signing and the program writing the subscription and the \
         token accounts.",
    );
    let ix = TransferSubscription::new(world.svm_mut(), &merchant, alice.pubkey(), mint, subscription_pda, plan_pda)
        .amount(10_000_000)
        .instruction();
    world.send_ok(&[ix], &[&merchant], "alt: authorized pull (merchant)");

    let bal = token_balance(world.svm(), &merchant_ata);
    world.md().check("the authorized pull credited the merchant 10 tokens", 10_000_000, bal);

    // --- else: Caller not authorized --------------------------------------
    world.md().step("else — the caller is neither owner nor puller (unauthorized): the pull is refused");
    world.md().note(
        "Mallory is neither the plan owner nor a whitelisted puller, so the authorization check at step 4 \
         fails before any token moves. The refused tree names the guard: `Unauthorized`. The \
         transaction-level surface stops at the program frame; nothing is signed into the token program.",
    );
    let mallory = world.actor("mallory");
    let ix = TransferSubscription::new(world.svm_mut(), &mallory, alice.pubkey(), mint, subscription_pda, plan_pda)
        .amount(10_000_000)
        .to(merchant_ata)
        .instruction();
    world.send_err(&[ix], &[&mallory], "else: unauthorized pull (mallory)", SubscriptionsError::Unauthorized);

    let bal = token_balance(world.svm(), &merchant_ata);
    world.md().check("the refused pull moved nothing more — the merchant still holds only the authorized 10", 10_000_000, bal);
}
