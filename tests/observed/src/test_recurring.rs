//! A recurring pull payment, observed, and the audit question it surfaces.
//!
//! The program lets a delegatee (a merchant) pull up to `amount_per_period`
//! tokens per period from a subscriber's delegation. Act 1 renders one such pull
//! the full five ways (`render_all`): the CPI tree, both sequence diagrams, and
//! the authority and ownership graphs. Acts 2 and 3 then put a non-obvious
//! behavior under the report vocabulary so it can be questioned: the
//! delegation's `expiry_ts` is *soft*. `is_effectively_expired` rejects only
//! when `current_ts > expiry_ts + TIME_DRIFT_ALLOWED_SECS` (120s), so a pull can
//! land up to two minutes *after* the stated expiry. This test passes: the
//! behavior is the design (a clock-drift allowance). The question it hands the
//! maintainer is whether a subscriber expecting "no charges after expiry" should
//! be charged in that window.

use std::sync::Arc;

use litesvm_utils::{
    deterministic_keypair, Keypair, MarkdownBlock, Pubkey, Report, Signer, TestSVM, TransactionResult,
};
use solana_instruction::{AccountMeta, Instruction};

use subscriptions::{
    event_engine::{event_authority_pda, EMIT_EVENT_IX_DISC, EVENT_IX_TAG_LE},
    instructions::transfer_recurring_delegation,
};
use tests_subscriptions::tests::{
    asserts::TransactionResultExt,
    constants::{MINT_DECIMALS, PROGRAM_ID, TOKEN_PROGRAM_ID},
    pda::get_subscription_authority_pda,
    utils::{
        get_ata_balance, init_ata, init_mint, initialize_subscription_authority_action, move_clock_forward,
        CreateDelegation,
    },
};

use spl_associated_token_account_interface::address::get_associated_token_address_with_program_id;

use crate::harness::{render_all, world, NOW};

/// `RecurringTransfer`'s 1-byte event discriminator; see
/// `event_engine::EventDiscriminators::RecurringTransfer`.
const RECURRING_TRANSFER_DISC: u8 = 4;
const TIME_DRIFT_ALLOWED_SECS: i64 = 120;

#[test]
fn recurring_pull_and_the_soft_expiry_it_survives() {
    let mut md = Report::new(
        "A recurring pull, and the soft expiry it survives",
        "a merchant pulls a recurring payment; the same pull still lands within a drift window past expiry",
    );

    let mut backend = world();

    let alice = deterministic_keypair("subscriptions", "alice"); // subscriber (delegator)
    let bob = deterministic_keypair("subscriptions", "bob"); // merchant (delegatee)
    backend.fund_sol(&alice.pubkey(), 10_000_000_000);
    backend.fund_sol(&bob.pubkey(), 10_000_000_000);

    let mint = init_mint(backend.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, Some(alice.pubkey()), &[]);
    init_ata(backend.svm_mut(), mint, alice.pubkey(), 100_000_000);
    init_ata(backend.svm_mut(), mint, bob.pubkey(), 0);

    // The merchant's recurring allowance: 50 tokens/hour, expiring in 10 minutes.
    md.step("Set the stage: Alice grants Bob a recurring allowance, expiring in 10 minutes");
    initialize_subscription_authority_action(backend.svm_mut(), &alice, mint).0.assert_ok();
    let amount_per_period: u64 = 50_000_000;
    let expiry_ts = NOW + 600; // 10 minutes
    let (create_res, delegation_pda) = CreateDelegation::new(backend.svm_mut(), &alice, mint, bob.pubkey())
        .nonce(0)
        .recurring(amount_per_period, 3600, NOW, expiry_ts);
    create_res.assert_ok();

    let (subscription_authority_pda, _) = get_subscription_authority_pda(&alice.pubkey(), &mint);
    let event_authority = Pubkey::new_from_array(event_authority_pda::ID.to_bytes());
    let bob_ata = get_associated_token_address_with_program_id(&bob.pubkey(), &mint, &TOKEN_PROGRAM_ID);

    // Vocabulary on the backend (the single source; `record.into()` carries it).
    backend.register_alias(&alice.pubkey(), "alice");
    backend.register_alias(&bob.pubkey(), "bob (merchant)");
    backend.register_alias(&mint, "USDC mint");
    backend.register_alias(&delegation_pda, "Bob's delegation");
    backend.register_alias(&subscription_authority_pda, "Alice's SubAuthority");
    backend.register_alias(&bob_ata, "bob's ATA");
    backend.register_alias(&event_authority, "EventAuthority");
    backend.register_alias(&PROGRAM_ID, "subscriptions");
    backend.register_program_instructions(&PROGRAM_ID, &[
        (*transfer_recurring_delegation::DISCRIMINATOR, "TransferRecurring"),
        (EMIT_EVENT_IX_DISC, "EmitEvent"),
    ]);
    let mut prefix = EVENT_IX_TAG_LE.to_vec();
    prefix.push(RECURRING_TRANSFER_DISC);
    backend.register_cpi_event(&PROGRAM_ID, &prefix, "RecurringTransfer", recurring_transfer_decoder());
    backend.register_program_errors(&PROGRAM_ID, &[(128, "DelegationExpired"), (400, "AmountExceedsPeriodLimit")]);

    // --- Act 1: a normal pull, within the allowance, before expiry ----------
    md.step("Act 1: Bob pulls 10 tokens, within his allowance");
    let bob_before = get_ata_balance(backend.svm(), &bob_ata);
    let pull = pull_ix(&alice, &bob, mint, delegation_pda, subscription_authority_pda, event_authority);
    let r1 = backend.send(&[pull], &[&bob]);
    assert!(r1.error.is_none(), "the in-window pull should succeed: {:?}", r1.error);
    let r1: TransactionResult = r1.into();
    let bob_after = get_ata_balance(backend.svm(), &bob_ata);
    md.transition("bob's token balance", bob_before, bob_before + 10_000_000, bob_after, "the recurring pull reached the merchant");
    md.note(
        "The in-window pull, rendered five ways. Read the same transaction as a CPI tree, as a \
         sequence of messages (with and without lifelines), as an authority graph, and as an \
         ownership graph.",
    );
    render_all(&mut md, &r1, backend.svm(), "Recurring pull");

    // --- Act 2: the same pull, 60s AFTER the stated expiry ------------------
    md.step("Act 2: the clock passes the stated expiry; Bob pulls again 60 seconds late");
    // Move to expiry + 60s (still inside the 120s drift window).
    move_clock_forward(backend.svm_mut(), (expiry_ts - NOW + 60) as u64);
    let late = pull_ix(&alice, &bob, mint, delegation_pda, subscription_authority_pda, event_authority);
    let r2 = backend.send(&[late], &[&bob]);
    let late_succeeded = r2.error.is_none();
    md.check("a pull 60s past `expiry_ts` is still accepted (soft expiry)", true, late_succeeded);
    md.note(
        "This is the question for review. Bob's allowance carries an explicit `expiry_ts`, yet the pull above \
         landed 60 seconds after it. The program treats expiry as soft: `is_effectively_expired` rejects only \
         when `current_ts > expiry_ts + TIME_DRIFT_ALLOWED_SECS`, a 120-second clock-drift allowance. It is a \
         deliberate tolerance for validator clock skew, and this test passes because the behavior is the design. \
         The question is whether a subscriber who reads `expiry_ts` as 'no charges after this instant' should be \
         charged in the two minutes that follow, and whether 120 seconds is the right window.",
    );

    // --- Act 3: past the drift window, the pull is finally refused ----------
    md.step("Act 3: past the 120s drift window, the same pull is refused");
    move_clock_forward(backend.svm_mut(), (TIME_DRIFT_ALLOWED_SECS + 5) as u64);
    let too_late = pull_ix(&alice, &bob, mint, delegation_pda, subscription_authority_pda, event_authority);
    let r3 = backend.send(&[too_late], &[&bob]);
    md.check("past the drift window the pull is refused", true, r3.error.is_some());
    let r3: TransactionResult = r3.into();
    md.block("Refused pull (the hard cutoff)", MarkdownBlock::Fenced { lang: "text".into(), body: r3.logs_structured_string() });

    r1.print_logs_structured();
}

/// Hand-build the `TransferRecurring` instruction (mirrors the suite's
/// `TransferDelegation` builder, routed through the observed backend).
fn pull_ix(
    alice: &Keypair,
    bob: &Keypair,
    mint: Pubkey,
    delegation_pda: Pubkey,
    subscription_authority_pda: Pubkey,
    event_authority: Pubkey,
) -> Instruction {
    let delegator_ata = get_associated_token_address_with_program_id(&alice.pubkey(), &mint, &TOKEN_PROGRAM_ID);
    let receiver_ata = get_associated_token_address_with_program_id(&bob.pubkey(), &mint, &TOKEN_PROGRAM_ID);
    let accounts = vec![
        AccountMeta::new(delegation_pda, false),
        AccountMeta::new(subscription_authority_pda, false),
        AccountMeta::new(delegator_ata, false),
        AccountMeta::new(receiver_ata, false),
        AccountMeta::new_readonly(mint, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(bob.pubkey(), true),
        AccountMeta::new_readonly(event_authority, false),
        AccountMeta::new_readonly(PROGRAM_ID, false),
    ];
    let data = [
        vec![*transfer_recurring_delegation::DISCRIMINATOR],
        10_000_000u64.to_le_bytes().to_vec(),
        alice.pubkey().to_bytes().to_vec(),
        mint.to_bytes().to_vec(),
    ]
    .concat();
    Instruction { program_id: PROGRAM_ID, accounts, data }
}

/// Decoder for the `RecurringTransfer` event body (after the 8-byte tag + 1-byte
/// disc): delegation(32) ++ delegator(32) ++ delegatee(32) ++ mint(32) ++
/// amount(u64) ++ period_start(i64) ++ period_end(i64) ++ pulled(u64) ++ receiver(32).
fn recurring_transfer_decoder() -> Arc<dyn Fn(&[u8]) -> Option<Vec<(String, String)>> + Send + Sync> {
    Arc::new(|body: &[u8]| {
        if body.len() != 32 * 5 + 8 * 4 {
            return None;
        }
        let pk = |r: &[u8]| Pubkey::new_from_array(<[u8; 32]>::try_from(r).unwrap()).to_string();
        let u64le = |r: &[u8]| u64::from_le_bytes(<[u8; 8]>::try_from(r).unwrap());
        Some(vec![
            ("delegatee".to_string(), pk(&body[64..96])),
            ("amount".to_string(), u64le(&body[128..136]).to_string()),
            ("pulled_in_period".to_string(), u64le(&body[152..160]).to_string()),
            ("receiver".to_string(), pk(&body[160..192])),
        ])
    })
}

