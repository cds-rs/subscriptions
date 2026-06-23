//! The World: one object that holds everything a converted test needs, plus the
//! scenario verbs that act on it. This is the anchor-litesvm world/scenario
//! pattern applied to the subscriptions suite.
//!
//! A test builds a `World`, draws its actors from the canonical cast (`actor`),
//! runs setup through the existing fabrication helpers (reached via `svm_mut`),
//! then performs each on-chain action through a scenario verb or the observed
//! `send_ok` / `send_err`. Every observed send routes through `LiteSvmBackend`
//! (so the per-frame trace is captured) and appends the rendered surface to the
//! `Report` the World owns; the report flushes to `target/md-reports/` when the
//! World drops at end of test.
//!
//! ## The cast
//!
//! Catalogued from across the suite, the ad-hoc signers collapse onto a small
//! set of roles. `actor(name)` derives a deterministic keypair for a role,
//! funds it once, and registers its alias, so every rendered artifact names the
//! same key the same way:
//!
//! | cast member         | role                                             |
//! |---------------------|--------------------------------------------------|
//! | `alice`             | the principal: subscriber, delegator, mint owner |
//! | `merchant`          | plan owner / payee                               |
//! | `bob`               | delegatee / payee (delegation family)            |
//! | `charlie`           | second counterparty                              |
//! | `sponsor`/`sponsor2`| rent and fee payer, not the beneficiary          |
//! | `mallory`           | the adversary (unifies attacker/eve/random)      |
//!
//! `bob` and `merchant` are kept distinct on purpose: the domain separates "a
//! merchant offering a plan" from "a delegatee granted a pull", and that
//! distinction may carry meaning for the maintainers.

use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use litesvm_utils::{
    deterministic_keypair, Keypair, LiteSVM, LiteSvmBackend, MarkdownBlock, Pubkey, Report, Signer, TestSVM,
    TransactionResult,
};
use solana_instruction::{AccountMeta, Instruction};
use spl_associated_token_account_interface::address::get_associated_token_address_with_program_id;

use crate::{
    event_engine::{event_authority_pda, EMIT_EVENT_IX_DISC, EVENT_IX_TAG_LE},
    instructions::{
        cancel_subscription, close_subscription_authority, create_fixed_delegation, create_plan,
        create_recurring_delegation, delete_plan, initialize_subscription_authority, resume_subscription,
        revoke_abandoned_delegation, revoke_delegation, revoke_subscription_authority, subscribe,
        transfer_fixed_delegation, transfer_recurring_delegation, transfer_subscription, update_plan,
    },
    tests::{
        constants::{MINT_DECIMALS, PROGRAM_ID, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID},
        pda::{get_plan_pda, get_subscription_authority_pda, get_subscription_pda},
        utils::{days, init_ata, init_mint, CreatePlan, Subscribe},
    },
    SubscriptionsError,
};

/// A fixed wall-clock for the world. Pinned so PDAs, timestamps, and thus the
/// rendered reports are byte-reproducible across runs. Converted tests read it
/// through [`World::now`] rather than the machine clock.
pub const NOW: i64 = 1_700_000_000;

/// The default funding for a freshly cast actor (100 SOL).
const ACTOR_FUNDING: u64 = 100 * 1_000_000_000;

/// Expand `(SubscriptionsError::V as u32, "V")` pairs from a variant list, so the
/// codes are taken from the enum (correct across its gaps) and never transcribed.
macro_rules! errors {
    ($($v:ident),* $(,)?) => {
        &[ $((SubscriptionsError::$v as u32, stringify!($v))),* ]
    };
}

/// One object holding the backend, the narrative report, and the set of
/// already-funded actors.
pub struct World {
    backend: LiteSvmBackend,
    report: Report,
    funded: HashSet<String>,
}

impl World {
    /// Build the world: the program loaded, the clock pinned to [`NOW`], and the
    /// whole instruction / error / event vocabulary registered on the backend so
    /// every rendered artifact reads in names rather than bytes.
    pub fn new(title: &str, intent: &str) -> Self {
        let mut svm = LiteSVM::new();
        let so = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy/subscriptions_program.so");
        svm.add_program_from_file(PROGRAM_ID.to_bytes(), so).unwrap();

        let mut backend = LiteSvmBackend::new(svm);
        backend.warp_to_timestamp(NOW);
        backend.register_alias(&PROGRAM_ID, "subscriptions");
        backend.register_alias(&Pubkey::new_from_array(event_authority_pda::ID.to_bytes()), "EventAuthority");

        backend.register_program_instructions(
            &PROGRAM_ID,
            &[
                (*initialize_subscription_authority::DISCRIMINATOR, "InitSubscriptionAuthority"),
                (*subscribe::DISCRIMINATOR, "Subscribe"),
                (*cancel_subscription::DISCRIMINATOR, "CancelSubscription"),
                (*resume_subscription::DISCRIMINATOR, "ResumeSubscription"),
                (*transfer_subscription::DISCRIMINATOR, "TransferSubscription"),
                (*create_plan::DISCRIMINATOR, "CreatePlan"),
                (*update_plan::DISCRIMINATOR, "UpdatePlan"),
                (*delete_plan::DISCRIMINATOR, "DeletePlan"),
                (*create_fixed_delegation::DISCRIMINATOR, "CreateFixedDelegation"),
                (*create_recurring_delegation::DISCRIMINATOR, "CreateRecurringDelegation"),
                (*transfer_fixed_delegation::DISCRIMINATOR, "TransferFixed"),
                (*transfer_recurring_delegation::DISCRIMINATOR, "TransferRecurring"),
                (*revoke_delegation::DISCRIMINATOR, "RevokeDelegation"),
                (*revoke_abandoned_delegation::DISCRIMINATOR, "RevokeAbandonedDelegation"),
                (*revoke_subscription_authority::DISCRIMINATOR, "RevokeSubscriptionAuthority"),
                (*close_subscription_authority::DISCRIMINATOR, "CloseSubscriptionAuthority"),
                (EMIT_EVENT_IX_DISC, "EmitEvent"),
            ],
        );

        backend.register_program_errors(
            &PROGRAM_ID,
            errors![
                NotSigner, InvalidAddress, InvalidEscrowPda, InvalidSubscriptionAuthorityPda, NotSystemProgram,
                InvalidTokenProgram, InvalidToken2022MintAccountData, InvalidToken2022TokenAccountData,
                InvalidAssociatedTokenAccountDerivedAddress, InvalidTokenSplMintAccountData,
                InvalidTokenSplTokenAccountData, InvalidAccountData, InvalidInstructionData, NotEnoughAccountKeys,
                InvalidInstruction, ArithmeticOverflow, ArithmeticUnderflow, InvalidAccountDiscriminator,
                MintHasConfidentialTransfer, MintHasNonTransferable, MintHasPermanentDelegate, MintHasTransferHook,
                MintHasTransferFee, MintHasMintCloseAuthority, MintHasPausable, MintMismatch, InvalidDelegatePda,
                InvalidHeaderData, DelegationExpired, InvalidAmount, Unauthorized, AccountNotWritable,
                AtaOwnerMismatch, DelegationVersionMismatch, MigrationRequired, DelegationAlreadyExists,
                StaleSubscriptionAuthority, TransferHookTooManyAccounts, TransferHookValidationAccountMissing,
                AmountExceedsLimit, FixedDelegationExpiryInPast, FixedDelegationAmountZero, AmountExceedsPeriodLimit,
                PeriodNotElapsed, InvalidPeriodLength, InvalidPayerData, RecurringDelegationStartTimeInPast,
                RecurringDelegationStartTimeGreaterThanExpiry, RecurringDelegationAmountZero, DelegationNotStarted,
                RecurringDelegationStartOnLandingRequiresExpiry, PlanSunset, PlanExpired, InvalidPlanPda,
                InvalidSubscriptionPda, NotPlanOwner, SubscriptionPlanMismatch, UnauthorizedDestination,
                InvalidNumDestinations, SubscriptionCancelled, SubscriptionAlreadyCancelled, SubscriptionNotCancelled,
                InvalidEndTs, InvalidPlanStatus, PlanImmutableAfterSunset, SunsetRequiresEndTs, PlanNotExpired,
                PlanClosed, AlreadySubscribed, PlanAlreadyExists, PlanTermsMismatch, InvalidEventAuthority,
                InvalidEventData, InvalidEventTag, InvalidEventDiscriminator
            ],
        );

        register_events(&mut backend);

        Self { backend, report: Report::new(title, intent), funded: HashSet::new() }
    }

    // --- cast and props ------------------------------------------------------

    /// Draw an actor from the cast by role name: a deterministic keypair, funded
    /// once, aliased to its name. Returns an owned clone so the caller can hold
    /// it while still borrowing the world mutably for sends.
    pub fn actor(&mut self, name: &str) -> Keypair {
        let kp = deterministic_keypair("subscriptions", name);
        if self.funded.insert(name.to_string()) {
            self.backend.fund_sol(&kp.pubkey(), ACTOR_FUNDING);
        }
        self.backend.register_alias(&kp.pubkey(), name);
        kp
    }

    /// Register a non-signing account's alias (a prop): a PDA, an ATA, a mint.
    pub fn prop(&mut self, pubkey: Pubkey, name: &str) {
        self.backend.register_alias(&pubkey, name);
    }

    // --- escape hatches to the inner svm and report --------------------------

    /// The backend, for the suite's fabrication helpers (`init_mint`,
    /// `init_ata`, ...). Sends made directly through this are setup, not the
    /// observed action under test.
    pub fn svm_mut(&mut self) -> &mut LiteSvmBackend {
        &mut self.backend
    }

    /// Read-only view of the backend.
    pub fn svm(&self) -> &LiteSvmBackend {
        &self.backend
    }

    /// The narrative report, for `md.step` / `md.note` / `md.check` steps.
    pub fn md(&mut self) -> &mut Report {
        &mut self.report
    }

    /// The world's current unix timestamp (the pinned clock, advanced by
    /// [`warp`](Self::warp)).
    pub fn now(&self) -> i64 {
        self.backend.clock().unix_timestamp
    }

    /// Advance the clock by `seconds` (and the slot, mirroring the suite's
    /// `move_clock_forward`).
    pub fn warp(&mut self, seconds: u64) {
        // Each setter read-modify-writes the same Clock, touching only its own
        // field, so two sequential calls reproduce the old single combined write.
        let clock = self.backend.clock();
        self.backend.warp_to_timestamp(clock.unix_timestamp + seconds as i64);
        self.backend.warp_to_slot(clock.slot + seconds * 2);
    }

    // --- the observed send ---------------------------------------------------

    /// Send `ixs` through the observing backend, append the rendered surface to
    /// the report under `label`, and return the rich result. The full surface
    /// (CPI tree, both sequence diagrams, authority and ownership graphs) is
    /// rendered on success; a refused transaction renders only its tree, which
    /// already carries the named error.
    ///
    /// `signers[0]` is the fee payer; order a sponsor first when it should pay.
    pub fn send(&mut self, ixs: &[Instruction], signers: &[&Keypair], label: &str) -> TransactionResult {
        let result: TransactionResult = self.backend.send(ixs, signers).into();

        self.report.block(
            format!("{label}: structured CPI tree"),
            MarkdownBlock::Fenced { lang: "text".into(), body: result.logs_structured_string() },
        );
        if result.is_success() {
            self.report.block(format!("{label}: sequence diagram"), MarkdownBlock::Raw(result.mermaid_string()));
            self.report.block(
                format!("{label}: sequence diagram, with lifelines"),
                MarkdownBlock::Raw(result.mermaid_string_with_lifelines()),
            );
            self.report
                .block(format!("{label}: authority graph"), MarkdownBlock::Raw(result.authority_graph_string()));
            self.report.block(
                format!("{label}: ownership graph"),
                MarkdownBlock::Raw(result.ownership_graph_string()),
            );
        }
        result
    }

    /// Observed send that must succeed; asserts and returns the rich result.
    pub fn send_ok(&mut self, ixs: &[Instruction], signers: &[&Keypair], label: &str) -> TransactionResult {
        self.send(ixs, signers, label).assert_ok()
    }

    /// Observed send that must fail with `expected`; asserts the named error.
    pub fn send_err(&mut self, ixs: &[Instruction], signers: &[&Keypair], label: &str, expected: SubscriptionsError) {
        self.send(ixs, signers, label).assert_err(expected);
    }

    // --- scenario verbs ------------------------------------------------------

    /// Initialize `user`'s SubscriptionAuthority for `mint`, optionally with a
    /// sponsor paying rent and fee. Mirrors the suite's
    /// `initialize_subscription_authority_action`, routed through the observed
    /// backend. Returns the rich result, the authority PDA, and its bump.
    pub fn init_authority(
        &mut self,
        user: &Keypair,
        mint: Pubkey,
        sponsor: Option<&Keypair>,
    ) -> (TransactionResult, Pubkey, u8) {
        let token_program = self.backend.get_account(&mint).unwrap().owner;
        let user_ata = get_associated_token_address_with_program_id(&user.pubkey(), &mint, &token_program);
        let (pda, bump) = get_subscription_authority_pda(&user.pubkey(), &mint);
        self.prop(pda, "SubAuthority");

        let mut accounts = vec![
            AccountMeta::new(user.pubkey(), true),
            AccountMeta::new(pda, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(user_ata, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(token_program, false),
        ];
        // `signers[0]` is the fee payer: sponsor first when one pays.
        let mut signers: Vec<&Keypair> = vec![user];
        if let Some(s) = sponsor {
            accounts.push(AccountMeta::new(s.pubkey(), true));
            signers.insert(0, s);
        }

        let ix = Instruction {
            program_id: PROGRAM_ID,
            accounts,
            data: vec![*initialize_subscription_authority::DISCRIMINATOR],
        };
        let res = self.send(&[ix], &signers, "InitSubscriptionAuthority");
        (res, pda, bump)
    }

    /// Stage a live subscription: Alice (subscriber) with her authority, a
    /// merchant's plan (id 1, 50 tokens/hour, ending in 30 days), and Alice
    /// subscribed to it. Mirrors the suite's `setup_with_subscription`, but every
    /// send is observed. Returns the cast and the derived accounts.
    ///
    /// Each setup send renders into the report too, so a converted test's report
    /// opens with the staging sends before its own action.
    pub fn stage_subscription(&mut self) -> StagedSubscription {
        let alice = self.actor("alice");
        let merchant = self.actor("merchant");

        let mint = init_mint(self.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, Some(alice.pubkey()), &[]);
        self.prop(mint, "USDC mint");
        init_ata(self.svm_mut(), mint, alice.pubkey(), 100_000_000);

        self.md().step("Stage: Alice's authority, the merchant's plan, Alice subscribed");
        self.init_authority(&alice, mint, None).0.assert_ok();

        let end_ts = self.now() + days(30) as i64;
        let plan_ix = CreatePlan::new(self.svm_mut(), &merchant, mint)
            .plan_id(1)
            .amount(50_000_000)
            .period_hours(1)
            .end_ts(end_ts)
            .instruction();
        let (plan_pda, plan_bump) = get_plan_pda(&merchant.pubkey(), 1);
        self.prop(plan_pda, "Plan");
        self.send_ok(&[plan_ix], &[&merchant], "CreatePlan");

        let sub_ix = Subscribe::new(self.svm_mut(), &alice, merchant.pubkey(), plan_pda, 1, plan_bump, mint).instruction();
        let (subscription_pda, _) = get_subscription_pda(&plan_pda, &alice.pubkey());
        self.prop(subscription_pda, "Subscription");
        self.send_ok(&[sub_ix], &[&alice], "Subscribe");

        StagedSubscription { alice, merchant, mint, plan_pda, plan_bump, subscription_pda }
    }

    /// Fabricate the suite's standard mint: a 6-decimal classic-SPL mint with a
    /// billion-unit supply and `owner` as mint authority. The common case behind
    /// most setups; returns the mint (alias it yourself if the report wants one).
    pub fn usdc_mint(&mut self, owner: &Keypair) -> Pubkey {
        init_mint(self.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, Some(owner.pubkey()), &[])
    }

    /// Fabricate `owner`'s associated token account for `mint`, funded with
    /// `amount`. Returns the ATA address.
    pub fn fund_ata(&mut self, mint: Pubkey, owner: &Keypair, amount: u64) -> Pubkey {
        init_ata(self.svm_mut(), mint, owner.pubkey(), amount)
    }
}

/// The accounts and actors a staged subscription hands back (see
/// [`World::stage_subscription`]).
pub struct StagedSubscription {
    pub alice: Keypair,
    pub merchant: Keypair,
    pub mint: Pubkey,
    pub plan_pda: Pubkey,
    pub plan_bump: u8,
    pub subscription_pda: Pubkey,
}

/// The event decoders the suite renders. Each is keyed on the 8-byte event tag
/// plus the 1-byte event discriminator (the program emits events as a self-CPI
/// whose data is `EVENT_IX_TAG_LE ++ disc ++ borsh fields`, so there is no
/// `Program data:` log; the decoder reads the payload off the traced frame).
fn register_events(backend: &mut LiteSvmBackend) {
    // SubscriptionCreated (disc 0): plan(32) ++ subscriber(32) ++ mint(32) ++ created_ts(i64).
    backend.register_cpi_event(
        &PROGRAM_ID,
        &cpi_prefix(0),
        "SubscriptionCreated",
        Arc::new(|body: &[u8]| {
            if body.len() != 32 * 3 + 8 {
                return None;
            }
            Some(vec![
                ("plan".into(), pk(&body[0..32])),
                ("subscriber".into(), pk(&body[32..64])),
                ("mint".into(), pk(&body[64..96])),
                ("created_ts".into(), i64::from_le_bytes(body[96..104].try_into().unwrap()).to_string()),
            ])
        }),
    );

    // RecurringTransfer (disc 4): delegation(32) ++ delegator(32) ++ delegatee(32)
    // ++ mint(32) ++ amount(u64) ++ period_start(i64) ++ period_end(i64) ++ pulled(u64) ++ receiver(32).
    backend.register_cpi_event(
        &PROGRAM_ID,
        &cpi_prefix(4),
        "RecurringTransfer",
        Arc::new(|body: &[u8]| {
            if body.len() != 32 * 5 + 8 * 4 {
                return None;
            }
            Some(vec![
                ("delegatee".into(), pk(&body[64..96])),
                ("amount".into(), u64::from_le_bytes(body[128..136].try_into().unwrap()).to_string()),
                ("pulled_in_period".into(), u64::from_le_bytes(body[152..160].try_into().unwrap()).to_string()),
                ("receiver".into(), pk(&body[160..192])),
            ])
        }),
    );
}

/// The CPI-event key for a 1-byte event discriminator: the tag followed by the disc.
fn cpi_prefix(disc: u8) -> Vec<u8> {
    let mut p = EVENT_IX_TAG_LE.to_vec();
    p.push(disc);
    p
}

/// Render 32 bytes as a base58 pubkey string (decoders substitute aliases later).
fn pk(raw: &[u8]) -> String {
    Pubkey::new_from_array(<[u8; 32]>::try_from(raw).unwrap()).to_string()
}

/// Ergonomic assertions on the observed (rich) result, mirroring the suite's
/// `TransactionResultExt` so converted tests read the same: `.assert_ok()` /
/// `.assert_err(SubscriptionsError::X)`.
pub trait ObservedResultExt {
    /// Assert the transaction succeeded; returns the result for chaining.
    fn assert_ok(self) -> Self;
    /// Assert the transaction failed with `expected` (matched by error code).
    fn assert_err(self, expected: SubscriptionsError);
}

impl ObservedResultExt for TransactionResult {
    fn assert_ok(self) -> Self {
        self.assert_success()
    }

    fn assert_err(self, expected: SubscriptionsError) {
        self.assert_error_code(expected as u32);
    }
}

/// Render a program-side address's 32 bytes as a base58 `Pubkey`, so report
/// `check` rows read in keys rather than raw byte arrays.
pub fn as_pubkey(bytes: [u8; 32]) -> Pubkey {
    Pubkey::new_from_array(bytes)
}

