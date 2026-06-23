//! `subscribe`, observed: a sponsor funds the subscription and pays the fee, and
//! the subscriber's lamports stay untouched. The point of this twin is the tail:
//! after the assertions, `render_all` lays the one transaction out five ways.

use std::sync::Arc;

use litesvm_utils::{
    deterministic_keypair, Keypair, Pubkey, Report, Signer, TestSVM, TransactionResult,
};
use solana_instruction::{AccountMeta, Instruction};

use subscriptions::{
    event_engine::{event_authority_pda, EMIT_EVENT_IX_DISC, EVENT_IX_TAG_LE},
    instructions::subscribe,
    state::{Plan, SubscriptionAuthority, SubscriptionDelegation},
};
use tests_subscriptions::tests::{
    constants::{MINT_DECIMALS, PROGRAM_ID, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID},
    pda::{get_plan_pda, get_subscription_authority_pda, get_subscription_pda},
    utils::{days, init_ata, init_mint, initialize_subscription_authority_action, CreatePlan},
};

use crate::harness::{render_all, world, NOW};

/// `SubscriptionCreated`'s 1-byte event discriminator (the 9th wire byte, after
/// the 8-byte tag); see `event_engine::EventDiscriminators::SubscriptionCreated`.
const SUBSCRIPTION_CREATED_DISC: u8 = 0;

#[test]
fn subscribe_with_sponsor_tells_who_pays() {
    let mut md = Report::new(
        "Subscribe with a sponsor",
        "a sponsor funds the subscription account and pays the fee; the subscriber's lamports stay untouched",
    );

    let mut backend = world();

    // Deterministic actors so every PDA derived from them (and thus the whole
    // report) is byte-reproducible across runs.
    let alice = deterministic_keypair("subscriptions", "alice");
    let merchant = deterministic_keypair("subscriptions", "merchant");
    let sponsor = deterministic_keypair("subscriptions", "sponsor");
    backend.fund_sol(&alice.pubkey(), 10_000_000_000);
    backend.fund_sol(&merchant.pubkey(), 10_000_000_000);
    backend.fund_sol(&sponsor.pubkey(), 10_000_000_000);

    // --- fabricate the mint + Alice's funded ATA (not observed) -------------
    let mint = init_mint(&mut backend, TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, Some(alice.pubkey()), &[]);
    init_ata(&mut backend, mint, alice.pubkey(), 100_000_000);

    // --- setup sends through their builders (must succeed; not the focus) ---
    md.step("Set the stage: Alice's authority, the merchant's plan");
    // The setup sends route through the engine-neutral `TestSVM::send` and assert
    // success internally (silent setup, not the observed action under test).
    initialize_subscription_authority_action(&mut backend, &alice, mint);
    let end_ts = NOW + days(30) as i64;
    let (_, plan_pda) = CreatePlan::new(&mut backend, &merchant, mint)
        .plan_id(1)
        .amount(50_000_000)
        .period_hours(1)
        .end_ts(end_ts)
        .execute();
    let (_, plan_bump) = get_plan_pda(&merchant.pubkey(), 1);

    let (subscription_authority_pda, _) = get_subscription_authority_pda(&alice.pubkey(), &mint);
    let (subscription_pda, _) = get_subscription_pda(&plan_pda, &alice.pubkey());
    let event_authority = Pubkey::new_from_array(event_authority_pda::ID.to_bytes());

    // The cast list and decode vocabulary, registered ON THE BACKEND through the
    // `TestSVM` sockets. `send` stamps the backend's registries onto the record,
    // so `record.into()` carries them onto the rich result with no per-result
    // re-attachment: registering on the backend is the single source.
    backend.register_alias(&alice.pubkey(), "alice");
    backend.register_alias(&merchant.pubkey(), "merchant");
    backend.register_alias(&sponsor.pubkey(), "sponsor");
    backend.register_alias(&mint, "USDC mint");
    backend.register_alias(&plan_pda, "Plan");
    backend.register_alias(&subscription_authority_pda, "Alice's SubAuthority");
    backend.register_alias(&subscription_pda, "Subscription");
    backend.register_alias(&event_authority, "EventAuthority");
    backend.register_alias(&PROGRAM_ID, "subscriptions");

    backend.register_program_instructions(
        &PROGRAM_ID,
        &[(*subscribe::DISCRIMINATOR, "Subscribe"), (EMIT_EVENT_IX_DISC, "EmitEvent")],
    );

    // The self-CPI event decoder. The program emits events as a CPI to itself
    // with data = EVENT_IX_TAG_LE ++ <1-byte disc> ++ borsh fields, so there is
    // no `Program data:` log; the payload is the inner instruction's data, which
    // the trace carries onto the frame. We key on the tag+disc prefix and decode
    // the fields by their fixed offsets (the program ships no IDL for events).
    let mut created_prefix = EVENT_IX_TAG_LE.to_vec();
    created_prefix.push(SUBSCRIPTION_CREATED_DISC);
    backend.register_cpi_event(
        &PROGRAM_ID,
        &created_prefix,
        "SubscriptionCreated",
        Arc::new(|body: &[u8]| {
            // plan(32) ++ subscriber(32) ++ mint(32) ++ created_ts(i64 LE)
            if body.len() != 32 * 3 + 8 {
                return None;
            }
            let pk = |r: &[u8]| Pubkey::new_from_array(<[u8; 32]>::try_from(r).unwrap()).to_string();
            let created_ts = i64::from_le_bytes(<[u8; 8]>::try_from(&body[96..104]).unwrap());
            Some(vec![
                ("plan".to_string(), pk(&body[0..32])),
                ("subscriber".to_string(), pk(&body[32..64])),
                ("mint".to_string(), pk(&body[64..96])),
                ("created_ts".to_string(), created_ts.to_string()),
            ])
        }),
    );

    // --- the observed send: Alice subscribes, the sponsor pays --------------
    md.step("Alice subscribes; the sponsor pays");
    let alice_before = backend.get_account(&alice.pubkey()).unwrap().lamports;
    let sponsor_before = backend.get_account(&sponsor.pubkey()).unwrap().lamports;

    let ix = build_subscribe_ix(
        &backend, &alice, &merchant, plan_pda, plan_bump, mint, &sponsor, subscription_pda,
        subscription_authority_pda, event_authority,
    );
    let record = backend.send(&[ix], &[&sponsor, &alice]);
    assert!(record.error.is_none(), "subscribe should succeed: {:?}", record.error);
    let result: TransactionResult = record.into();

    let alice_after = backend.get_account(&alice.pubkey()).unwrap().lamports;
    let sponsor_after = backend.get_account(&sponsor.pubkey()).unwrap().lamports;

    // --- the story, with teeth ---------------------------------------------
    md.transition(
        "Alice's lamports, across the subscribe",
        alice_before,
        alice_before,
        alice_after,
        "the subscriber is not charged: not the fee, not the rent",
    );
    md.check("the sponsor was charged", true, sponsor_after < sponsor_before);

    let sub_acc = backend.get_account(&subscription_pda).unwrap();
    let sub = SubscriptionDelegation::load(&sub_acc.data).unwrap();
    let payer = Pubkey::new_from_array(sub.header.payer.to_bytes());
    let delegator = Pubkey::new_from_array(sub.header.delegator.to_bytes());
    md.check("the Subscription's payer is the sponsor", sponsor.pubkey(), payer);
    md.check("the Subscription's delegator is Alice", alice.pubkey(), delegator);

    // --- the artifacts: one transaction, read five ways ---------------------
    md.note(
        "Below, the same `Subscribe` transaction is rendered five ways. The structured tree shows \
         the one top-level call and its two inner CPIs at depth 2: one to `System` (creating the \
         Subscription account) and one self-CPI back into the program. That self-CPI is the event \
         emission: an Anchor-compatible engine emits by invoking itself with \
         `EVENT_IX_TAG ++ disc ++ fields` as the instruction data, so there is no `Program data:` \
         log to read; the decoder reads that payload off the frame and renders it as \
         `SubscriptionCreated { .. }`. The two sequence diagrams say the same thing as messages \
         between participants (the lifelines variant adds activation bars showing how long each \
         frame is on the stack). The authority graph reads the transaction by privilege (who \
         signed, who merely funded, which PDA the program signed for); the ownership graph reads it \
         by account owner.",
    );
    md.note(
        "Two signer lists appear below, in two different orders, and both are correct. The tree \
         header reads `signers=[sponsor, alice]`: the transaction's required signers in message \
         order, fee payer first (the sponsor is `signers[0]`, so it pays the fee). The `Subscribe` \
         frame reads `signer=[alice, sponsor]`: the same two signers, but in the order their \
         accounts appear in *this instruction's* account list, where the subscriber sits at index 0 \
         and the sponsor is the optional trailing payer the program resolves last. The header \
         projects message order; the frame projects account-slot order. Neither list is sorted, so \
         with this layout they happen to read as exact reverses; reordering the accounts to make \
         them match would hand the program the keys in the wrong slots (it reads roles by position, \
         not by name).",
    );
    render_all(&mut md, &result, &backend, "Subscribe");

    // Echo to stdout for the spike (run with --nocapture).
    result.print_logs_structured();
}

/// Hand-build the sponsored `Subscribe` instruction, mirroring the suite's
/// `Subscribe` builder but routing through the observed backend. The data binds
/// the subscriber to the plan's live terms (read back from the accounts).
#[allow(clippy::too_many_arguments)]
fn build_subscribe_ix(
    backend: &litesvm_utils::LiteSvmBackend,
    alice: &Keypair,
    merchant: &Keypair,
    plan_pda: Pubkey,
    plan_bump: u8,
    mint: Pubkey,
    sponsor: &Keypair,
    subscription_pda: Pubkey,
    subscription_authority_pda: Pubkey,
    event_authority: Pubkey,
) -> Instruction {
    let plan_acc = backend.get_account(&plan_pda).unwrap();
    let plan = Plan::load(&plan_acc.data).unwrap();
    let auth_acc = backend.get_account(&subscription_authority_pda).unwrap();
    let init_id = SubscriptionAuthority::load(&auth_acc.data).unwrap().init_id;

    let accounts = vec![
        AccountMeta::new(alice.pubkey(), true),
        AccountMeta::new_readonly(merchant.pubkey(), false),
        AccountMeta::new_readonly(plan_pda, false),
        AccountMeta::new(subscription_pda, false),
        AccountMeta::new_readonly(subscription_authority_pda, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        AccountMeta::new_readonly(event_authority, false),
        AccountMeta::new_readonly(PROGRAM_ID, false),
        // The sponsor: an extra signing, writable account the program debits.
        AccountMeta::new(sponsor.pubkey(), true),
    ];

    let data = [
        vec![*subscribe::DISCRIMINATOR],
        1u64.to_le_bytes().to_vec(),
        vec![plan_bump],
        plan.data.mint.as_ref().to_vec(),
        plan.data.terms.amount.to_le_bytes().to_vec(),
        plan.data.terms.period_hours.to_le_bytes().to_vec(),
        plan.data.terms.created_at.to_le_bytes().to_vec(),
        init_id.to_le_bytes().to_vec(),
    ]
    .concat();

    let _ = mint;
    Instruction { program_id: PROGRAM_ID, accounts, data }
}

