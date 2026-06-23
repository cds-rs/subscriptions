use std::time::{SystemTime, UNIX_EPOCH};
use std::vec::Vec;

pub use scenarios::helpers::{days, minutes, rent_exempt_lamports};

use litesvm_utils::{model, TestSVM};
use solana_account::Account;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_program_pack::{IsInitialized, Pack};
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use spl_associated_token_account_interface::address::get_associated_token_address_with_program_id;
use spl_token_2022_interface::{
    extension::{
        confidential_transfer::ConfidentialTransferMint,
        immutable_owner::ImmutableOwner,
        mint_close_authority::MintCloseAuthority,
        non_transferable::{NonTransferable, NonTransferableAccount},
        pausable::{PausableAccount, PausableConfig},
        permanent_delegate::PermanentDelegate,
        transfer_fee::{TransferFeeAmount, TransferFeeConfig},
        transfer_hook::{TransferHook, TransferHookAccount},
        BaseStateWithExtensions, BaseStateWithExtensionsMut, ExtensionType, StateWithExtensions,
        StateWithExtensionsMut,
    },
    state::{Account as TokenAccount, AccountState, Mint as Mint2022},
};

use solana_instruction::AccountMeta;
#[cfg(test)]
use spl_tlv_account_resolution::{account::ExtraAccountMeta, state::ExtraAccountMetaList};
#[cfg(test)]
use spl_transfer_hook_interface::instruction::ExecuteInstruction;

use crate::{
    event_engine::event_authority_pda,
    instructions::create_plan::{PlanData, PlanTerms, MAX_DESTINATIONS, MAX_PULLERS},
    instructions::update_plan::UpdatePlanData,
    instructions::{
        cancel_subscription, close_subscription_authority, create_fixed_delegation, create_plan,
        create_recurring_delegation, delete_plan, initialize_subscription_authority, resume_subscription,
        revoke_abandoned_delegation, revoke_delegation, revoke_subscription_authority, subscribe,
        transfer_fixed_delegation, transfer_recurring_delegation, transfer_subscription, update_plan,
    },
    state::common::PlanStatus,
    tests::{
        constants::{PROGRAM_ID, SYSTEM_PROGRAM_ID},
        pda::{get_delegation_pda, get_plan_pda, get_subscription_authority_pda, get_subscription_pda},
        utils::ModelTxExt,
    },
};

/// Converts number of hours into seconds
pub fn hours(hours: u64) -> u64 {
    hours * minutes(60)
}

pub fn current_ts() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

pub fn move_clock_forward<B: TestSVM>(backend: &mut B, seconds: u64) {
    // `warp_to_timestamp` / `warp_to_slot` each read-modify-write the same Clock,
    // touching only their own field, so two sequential calls reproduce the old
    // single combined write (unix_timestamp += seconds, slot += seconds*2).
    let clock = backend.clock();
    backend.warp_to_timestamp(clock.unix_timestamp + seconds as i64);
    backend.warp_to_slot(clock.slot + seconds * 2);
}

pub fn token_balance<B: TestSVM>(backend: &B, ata: &Pubkey) -> u64 {
    let account = fetch_account::<spl_token_2022_interface::state::Account, _>(backend, ata);
    account.amount
}

pub fn fetch_account<T: Pack + IsInitialized, B: TestSVM>(backend: &B, pubkey: &Pubkey) -> T {
    let account = backend.get_account(pubkey).unwrap();
    T::unpack(&account.data[..T::LEN]).unwrap()
}

pub fn init_mint<B: TestSVM>(
    backend: &mut B,
    token_program: Pubkey,
    decimals: u8,
    supply: u64,
    authority: Option<Pubkey>,
    extensions: &[ExtensionType],
) -> Pubkey {
    let mint = Pubkey::new_unique();

    let space = if extensions.is_empty() {
        Mint2022::LEN
    } else {
        ExtensionType::try_calculate_account_len::<Mint2022>(extensions).unwrap()
    };
    let mut mint_data = vec![0u8; space];

    if extensions.is_empty() {
        let mint_state = Mint2022 {
            mint_authority: authority.into(),
            supply,
            decimals,
            is_initialized: true,
            freeze_authority: None.into(),
        };
        Mint2022::pack(mint_state, &mut mint_data).unwrap();
    } else {
        let mut state = StateWithExtensionsMut::<Mint2022>::unpack_uninitialized(&mut mint_data).unwrap();

        state.base.mint_authority = authority.into();
        state.base.supply = supply;
        state.base.decimals = decimals;
        state.base.is_initialized = true;
        state.base.freeze_authority = None.into();

        state.pack_base();
        state.init_account_type().unwrap();

        for ext in extensions {
            match ext {
                ExtensionType::ConfidentialTransferMint => {
                    state.init_extension::<ConfidentialTransferMint>(true).unwrap();
                }
                ExtensionType::NonTransferable => {
                    state.init_extension::<NonTransferable>(true).unwrap();
                }
                ExtensionType::PermanentDelegate => {
                    state.init_extension::<PermanentDelegate>(true).unwrap();
                }
                ExtensionType::TransferFeeConfig => {
                    let extension = state.init_extension::<TransferFeeConfig>(true).unwrap();
                    extension.older_transfer_fee.epoch = 0.into();
                    extension.older_transfer_fee.maximum_fee = 1_000_000.into();
                    extension.older_transfer_fee.transfer_fee_basis_points = 100.into();
                    extension.newer_transfer_fee = extension.older_transfer_fee;
                }
                ExtensionType::TransferHook => {
                    state.init_extension::<TransferHook>(true).unwrap();
                }
                ExtensionType::Pausable => {
                    state.init_extension::<PausableConfig>(true).unwrap();
                }
                ExtensionType::MintCloseAuthority => {
                    state.init_extension::<MintCloseAuthority>(true).unwrap();
                }
                _ => panic!("Unsupported extension type in test helper: {:?}", ext),
            }
        }
    }

    let lamports = rent_exempt_lamports(space);

    backend.set_account(
        &mint,
        Account { lamports, data: mint_data, owner: token_program, executable: false, rent_epoch: 0 },
    );

    mint
}

pub fn set_transfer_hook_config<B: TestSVM>(
    backend: &mut B,
    mint: Pubkey,
    authority: Option<Pubkey>,
    program_id: Option<Pubkey>,
) {
    let mut account = backend.get_account(&mint).unwrap();
    {
        let mut state = StateWithExtensionsMut::<Mint2022>::unpack(&mut account.data).unwrap();
        let extension = state.get_extension_mut::<TransferHook>().unwrap();
        extension.authority = authority.try_into().unwrap();
        extension.program_id = program_id.try_into().unwrap();
    }
    backend.set_account(&mint, account);
}

pub const TRANSFER_HOOK_EXAMPLE_PROGRAM_ID: Pubkey = Pubkey::new_from_array([42u8; 32]);

pub fn load_transfer_hook_example<B: TestSVM>(backend: &mut B) {
    let so_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../transfer-hook-example/target/deploy/transfer_hook_example.so");
    let bytes = std::fs::read(so_path).unwrap();
    backend.deploy_program(TRANSFER_HOOK_EXAMPLE_PROGRAM_ID, &bytes);
}

#[cfg(test)]
pub fn install_transfer_hook_extra_metas<B: TestSVM>(backend: &mut B, mint: Pubkey) -> (Pubkey, Pubkey) {
    let program_id = TRANSFER_HOOK_EXAMPLE_PROGRAM_ID;
    let (validation_pda, _) = Pubkey::find_program_address(&[b"extra-account-metas", mint.as_ref()], &program_id);
    let counter = Pubkey::new_unique();

    let meta = ExtraAccountMeta {
        discriminator: 0,
        address_config: counter.to_bytes(),
        is_signer: false.into(),
        is_writable: true.into(),
    };
    let mut validation_data = vec![0u8; ExtraAccountMetaList::size_of(1).unwrap()];
    ExtraAccountMetaList::init::<ExecuteInstruction>(&mut validation_data, &[meta]).unwrap();

    let validation_lamports = rent_exempt_lamports(validation_data.len());
    backend.set_account(
        &validation_pda,
        Account {
            lamports: validation_lamports,
            data: validation_data,
            owner: program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    let counter_lamports = rent_exempt_lamports(1);
    backend.set_account(
        &counter,
        Account {
            lamports: counter_lamports,
            data: vec![0u8; 1],
            owner: program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    (validation_pda, counter)
}

pub fn init_ata<B: TestSVM>(backend: &mut B, mint: Pubkey, owner: Pubkey, amount: u64) -> Pubkey {
    let token_program = backend.get_account(&mint).unwrap().owner;
    let ata = get_associated_token_address_with_program_id(&owner, &mint, &token_program);
    init_token_account_at(backend, ata, mint, owner, amount)
}

pub fn init_aux_token_account<B: TestSVM>(backend: &mut B, mint: Pubkey, owner: Pubkey, amount: u64) -> Pubkey {
    init_token_account_at(backend, Pubkey::new_unique(), mint, owner, amount)
}

fn init_token_account_at<B: TestSVM>(
    backend: &mut B,
    token_account: Pubkey,
    mint: Pubkey,
    owner: Pubkey,
    amount: u64,
) -> Pubkey {
    let token_program = backend.get_account(&mint).unwrap().owner;
    let account_extensions = if token_program == crate::tests::constants::TOKEN_2022_PROGRAM_ID {
        let mint_account = backend.get_account(&mint).unwrap();
        let mint_state = StateWithExtensions::<Mint2022>::unpack(&mint_account.data).unwrap();
        let mint_extensions = mint_state.get_extension_types().unwrap();
        ExtensionType::get_required_init_account_extensions(&mint_extensions)
    } else {
        vec![]
    };
    let space = if account_extensions.is_empty() {
        TokenAccount::LEN
    } else {
        ExtensionType::try_calculate_account_len::<TokenAccount>(&account_extensions).unwrap()
    };
    let ata_state = TokenAccount {
        mint,
        owner,
        amount,
        delegate: None.into(),
        state: AccountState::Initialized,
        is_native: None.into(),
        delegated_amount: 0,
        close_authority: None.into(),
    };

    let mut ata_data = vec![0u8; space];
    if account_extensions.is_empty() {
        TokenAccount::pack(ata_state, &mut ata_data).unwrap();
    } else {
        let mut state = StateWithExtensionsMut::<TokenAccount>::unpack_uninitialized(&mut ata_data).unwrap();
        state.base = ata_state;
        state.pack_base();
        state.init_account_type().unwrap();

        for ext in account_extensions {
            match ext {
                ExtensionType::TransferFeeAmount => {
                    state.init_extension::<TransferFeeAmount>(true).unwrap();
                }
                ExtensionType::NonTransferableAccount => {
                    state.init_extension::<NonTransferableAccount>(true).unwrap();
                }
                ExtensionType::ImmutableOwner => {
                    state.init_extension::<ImmutableOwner>(true).unwrap();
                }
                ExtensionType::TransferHookAccount => {
                    state.init_extension::<TransferHookAccount>(true).unwrap();
                }
                ExtensionType::PausableAccount => {
                    state.init_extension::<PausableAccount>(true).unwrap();
                }
                _ => panic!("Unsupported account extension type in test helper: {:?}", ext),
            }
        }
    }
    let lamports = rent_exempt_lamports(space);

    backend.set_account(
        &token_account,
        Account { lamports, data: ata_data, owner: token_program, executable: false, rent_epoch: 0 },
    );

    token_account
}

/// Initialize `user`'s SubscriptionAuthority for `mint` as a silent setup send,
/// routed through the engine-neutral [`TestSVM::send`] (asserted ok, not
/// rendered). The optional sponsor is appended as a writable signer and becomes
/// the fee payer (`signers[0]`). Returns the asserted record, the authority PDA,
/// and its bump. The World path renders this instead through `World::init_authority`.
pub fn initialize_subscription_authority_action<B: TestSVM>(
    backend: &mut B,
    user: &Keypair,
    mint: Pubkey,
) -> (model::Transaction, Pubkey, u8) {
    initialize_subscription_authority_action_with_sponsor(backend, user, mint, None)
}

/// See [`initialize_subscription_authority_action`]; this variant takes an
/// optional `sponsor` that pays the rent and fee.
pub fn initialize_subscription_authority_action_with_sponsor<B: TestSVM>(
    backend: &mut B,
    user: &Keypair,
    mint: Pubkey,
    sponsor: Option<&Keypair>,
) -> (model::Transaction, Pubkey, u8) {
    let token_program = backend.get_account(&mint).unwrap().owner;
    let user_ata = get_associated_token_address_with_program_id(&user.pubkey(), &mint, &token_program);
    let (subscription_authority_pda, bump) = get_subscription_authority_pda(&user.pubkey(), &mint);

    let mut accounts = vec![
        AccountMeta::new(user.pubkey(), true),
        AccountMeta::new(subscription_authority_pda, false),
        AccountMeta::new_readonly(mint, false),
        AccountMeta::new(user_ata, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        AccountMeta::new_readonly(token_program, false),
    ];

    // `signers[0]` is the fee payer: the sponsor pays when one is set.
    let mut signers: Vec<&Keypair> = vec![user];
    if let Some(sponsor) = sponsor {
        accounts.push(AccountMeta::new(sponsor.pubkey(), true));
        signers.insert(0, sponsor);
    }

    let ix =
        Instruction { program_id: PROGRAM_ID, accounts, data: vec![*initialize_subscription_authority::DISCRIMINATOR] };

    (backend.send(&[ix], &signers).assert_success(), subscription_authority_pda, bump)
}

pub struct CreateDelegation<'a, B: TestSVM> {
    svm: &'a mut B,
    delegator: &'a Keypair,
    payer: Option<&'a Keypair>,
    mint: Pubkey,
    delegatee: Pubkey,
    nonce: u64,
    custom_pda: Option<Pubkey>,
    expected_subscription_authority_init_id: Option<i64>,
}

impl<'a, B: TestSVM> CreateDelegation<'a, B> {
    pub fn new(svm: &'a mut B, delegator: &'a Keypair, mint: Pubkey, delegatee: Pubkey) -> Self {
        Self {
            svm,
            delegator,
            payer: None,
            mint,
            delegatee,
            nonce: 0,
            custom_pda: None,
            expected_subscription_authority_init_id: None,
        }
    }

    pub fn payer(mut self, payer: &'a Keypair) -> Self {
        self.payer = Some(payer);
        self
    }

    pub fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = nonce;
        self
    }

    pub fn pda(mut self, pda: Pubkey) -> Self {
        self.custom_pda = Some(pda);
        self
    }

    pub fn expected_subscription_authority_init_id(mut self, init_id: i64) -> Self {
        self.expected_subscription_authority_init_id = Some(init_id);
        self
    }

    fn resolved_expected_subscription_authority_init_id(&self) -> i64 {
        self.expected_subscription_authority_init_id.unwrap_or_else(|| {
            let (subscription_authority_pda, _) = get_subscription_authority_pda(&self.delegator.pubkey(), &self.mint);
            self.svm
                .get_account(&subscription_authority_pda)
                .and_then(|account| {
                    crate::state::SubscriptionAuthority::load(&account.data).ok().map(|authority| authority.init_id)
                })
                .unwrap_or_default()
        })
    }

    /// The `CreateFixedDelegation` instruction (and its delegation PDA), built
    /// but not sent, so a caller can route the send through an observing backend.
    pub fn fixed_ix(self, amount: u64, expiry_ts: i64) -> (Instruction, Pubkey) {
        let nonce_bytes = self.nonce.to_le_bytes().to_vec();
        let init_id = self.resolved_expected_subscription_authority_init_id();
        self.instruction(
            *create_fixed_delegation::DISCRIMINATOR,
            [nonce_bytes, amount.to_le_bytes().to_vec(), expiry_ts.to_le_bytes().to_vec(), init_id.to_le_bytes().to_vec()]
                .concat(),
        )
    }

    /// The `CreateRecurringDelegation` instruction (and its delegation PDA),
    /// built but not sent.
    pub fn recurring_ix(
        self,
        amount_per_period: u64,
        period_length_s: u64,
        start_ts: i64,
        expiry_ts: i64,
    ) -> (Instruction, Pubkey) {
        let nonce_bytes = self.nonce.to_le_bytes().to_vec();
        let init_id = self.resolved_expected_subscription_authority_init_id();
        self.instruction(
            *create_recurring_delegation::DISCRIMINATOR,
            [
                nonce_bytes,
                amount_per_period.to_le_bytes().to_vec(),
                period_length_s.to_le_bytes().to_vec(),
                start_ts.to_le_bytes().to_vec(),
                expiry_ts.to_le_bytes().to_vec(),
                init_id.to_le_bytes().to_vec(),
            ]
            .concat(),
        )
    }

    /// Construct the create-delegation instruction (the optional sponsor payer is
    /// appended as a writable signer account; the caller supplies the matching
    /// signer, sponsor first, when one pays).
    fn build_ix(&self, discriminator: u8, data: Vec<u8>) -> (Instruction, Pubkey) {
        let (subscription_authority_pda, _) = get_subscription_authority_pda(&self.delegator.pubkey(), &self.mint);
        let (derived_pda, _) =
            get_delegation_pda(&subscription_authority_pda, &self.delegator.pubkey(), &self.delegatee, self.nonce);
        let delegation_pda = self.custom_pda.unwrap_or(derived_pda);

        let mut accounts = vec![
            AccountMeta::new(self.delegator.pubkey(), true),
            AccountMeta::new(subscription_authority_pda, false),
            AccountMeta::new(delegation_pda, false),
            AccountMeta::new_readonly(self.delegatee, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];
        if let Some(p) = self.payer {
            accounts.push(AccountMeta::new(p.pubkey(), true));
        }

        let ix = Instruction { program_id: PROGRAM_ID, accounts, data: [vec![discriminator], data].concat() };
        (ix, delegation_pda)
    }

    fn instruction(self, discriminator: u8, data: Vec<u8>) -> (Instruction, Pubkey) {
        self.build_ix(discriminator, data)
    }
}

/// The send-driving builders route through the engine-neutral [`TestSVM::send`]
/// for silent setup sends (asserted ok, not rendered into a report). The World
/// path uses the `_ix` builders above instead and routes the send through the
/// observing backend so the surface is captured.
impl<'a, B: TestSVM> CreateDelegation<'a, B> {
    pub fn fixed(self, amount: u64, expiry_ts: i64) -> (model::Transaction, Pubkey) {
        let nonce_bytes = self.nonce.to_le_bytes().to_vec();
        let expected_subscription_authority_init_id = self.resolved_expected_subscription_authority_init_id();
        self.execute(
            *create_fixed_delegation::DISCRIMINATOR,
            [
                nonce_bytes,
                amount.to_le_bytes().to_vec(),
                expiry_ts.to_le_bytes().to_vec(),
                expected_subscription_authority_init_id.to_le_bytes().to_vec(),
            ]
            .concat(),
        )
    }

    pub fn recurring(
        self,
        amount_per_period: u64,
        period_length_s: u64,
        start_ts: i64,
        expiry_ts: i64,
    ) -> (model::Transaction, Pubkey) {
        let nonce_bytes = self.nonce.to_le_bytes().to_vec();
        let expected_subscription_authority_init_id = self.resolved_expected_subscription_authority_init_id();
        self.execute(
            *create_recurring_delegation::DISCRIMINATOR,
            [
                nonce_bytes,
                amount_per_period.to_le_bytes().to_vec(),
                period_length_s.to_le_bytes().to_vec(),
                start_ts.to_le_bytes().to_vec(),
                expiry_ts.to_le_bytes().to_vec(),
                expected_subscription_authority_init_id.to_le_bytes().to_vec(),
            ]
            .concat(),
        )
    }

    fn execute(self, discriminator: u8, data: Vec<u8>) -> (model::Transaction, Pubkey) {
        let (ix, delegation_pda) = self.build_ix(discriminator, data);
        // `signers[0]` is the fee payer: the sponsor pays when one is set.
        let mut signers = vec![self.delegator];
        if let Some(p) = self.payer {
            signers.insert(0, p);
        }
        (self.svm.send(&[ix], &signers).assert_success(), delegation_pda)
    }
}

pub struct TransferDelegation<'a, B: TestSVM> {
    svm: &'a mut B,
    signer: &'a Keypair,
    delegator: Pubkey,
    mint: Pubkey,
    delegation_pda: Pubkey,
    amount: u64,
    source: Option<Pubkey>,
    receiver: Option<Pubkey>,
    remaining: Vec<AccountMeta>,
}

impl<'a, B: TestSVM> TransferDelegation<'a, B> {
    pub fn new(
        svm: &'a mut B,
        signer: &'a Keypair,
        delegator: Pubkey,
        mint: Pubkey,
        delegation_pda: Pubkey,
    ) -> Self {
        Self {
            svm,
            signer,
            delegator,
            mint,
            delegation_pda,
            amount: 0,
            source: None,
            receiver: None,
            remaining: Vec::new(),
        }
    }

    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = amount;
        self
    }

    pub fn remaining(mut self, remaining: Vec<AccountMeta>) -> Self {
        self.remaining = remaining;
        self
    }

    pub fn to(mut self, receiver: Pubkey) -> Self {
        self.receiver = Some(receiver);
        self
    }

    pub fn from(mut self, source: Pubkey) -> Self {
        self.source = Some(source);
        self
    }

    /// The `TransferFixed` instruction, built but not sent. The caller supplies
    /// the signer (the delegatee) when routing through an observing backend.
    pub fn fixed_ix(self) -> Instruction {
        self.instruction(*transfer_fixed_delegation::DISCRIMINATOR)
    }

    /// The `TransferRecurring` instruction, built but not sent.
    pub fn recurring_ix(self) -> Instruction {
        self.instruction(*transfer_recurring_delegation::DISCRIMINATOR)
    }

    fn build_ix(&self, discriminator: u8) -> Instruction {
        let token_program = self.svm.get_account(&self.mint).unwrap().owner;
        let (subscription_authority_pda, _) = get_subscription_authority_pda(&self.delegator, &self.mint);
        let delegator_ata = self.source.unwrap_or_else(|| {
            get_associated_token_address_with_program_id(&self.delegator, &self.mint, &token_program)
        });

        // Default receiver is the signer's (delegatee's) ATA
        let receiver_ata = self.receiver.unwrap_or_else(|| {
            get_associated_token_address_with_program_id(&self.signer.pubkey(), &self.mint, &token_program)
        });

        let event_authority = Pubkey::new_from_array(event_authority_pda::ID.to_bytes());

        let mut accounts = vec![
            AccountMeta::new(self.delegation_pda, false),
            AccountMeta::new(subscription_authority_pda, false),
            AccountMeta::new(delegator_ata, false),
            AccountMeta::new(receiver_ata, false),
            AccountMeta::new_readonly(self.mint, false),
            AccountMeta::new_readonly(token_program, false),
            AccountMeta::new_readonly(self.signer.pubkey(), true),
            AccountMeta::new_readonly(event_authority, false),
            AccountMeta::new_readonly(PROGRAM_ID, false),
        ];
        accounts.extend(self.remaining.clone());

        Instruction {
            program_id: PROGRAM_ID,
            accounts,
            data: [
                vec![discriminator],
                self.amount.to_le_bytes().to_vec(),
                self.delegator.to_bytes().to_vec(),
                self.mint.to_bytes().to_vec(),
            ]
            .concat(),
        }
    }

    fn instruction(self, discriminator: u8) -> Instruction {
        self.build_ix(discriminator)
    }
}

impl<'a, B: TestSVM> TransferDelegation<'a, B> {
    pub fn fixed(self) -> model::Transaction {
        self.execute(*transfer_fixed_delegation::DISCRIMINATOR)
    }

    pub fn recurring(self) -> model::Transaction {
        self.execute(*transfer_recurring_delegation::DISCRIMINATOR)
    }

    fn execute(self, discriminator: u8) -> model::Transaction {
        let ix = self.build_ix(discriminator);
        self.svm.send(&[ix], &[self.signer]).assert_success()
    }
}

pub struct RevokeDelegation<'a, B: TestSVM> {
    svm: &'a mut B,
    delegator: &'a Keypair,
    signer: Option<&'a Keypair>,
    mint: Pubkey,
    delegatee: Pubkey,
    nonce: u64,
    receiver: Option<Pubkey>,
    custom_pda: Option<Pubkey>,
}

impl<'a, B: TestSVM> RevokeDelegation<'a, B> {
    pub fn new(svm: &'a mut B, delegator: &'a Keypair, mint: Pubkey, delegatee: Pubkey, nonce: u64) -> Self {
        Self { svm, delegator, signer: None, mint, delegatee, nonce, receiver: None, custom_pda: None }
    }

    pub fn signer(mut self, signer: &'a Keypair) -> Self {
        self.signer = Some(signer);
        self
    }

    pub fn receiver(mut self, receiver: Pubkey) -> Self {
        self.receiver = Some(receiver);
        self
    }

    pub fn pda(mut self, pda: Pubkey) -> Self {
        self.custom_pda = Some(pda);
        self
    }

    /// The `RevokeDelegation` instruction, built but not sent. The signing
    /// authority is the explicit `.signer()` or the delegator by default.
    pub fn instruction(&self) -> Instruction {
        let (subscription_authority_pda, _) = get_subscription_authority_pda(&self.delegator.pubkey(), &self.mint);
        let (derived_pda, _) =
            get_delegation_pda(&subscription_authority_pda, &self.delegator.pubkey(), &self.delegatee, self.nonce);
        let delegation_pda = self.custom_pda.unwrap_or(derived_pda);
        let authority = self.signer.unwrap_or(self.delegator);

        let mut accounts = vec![AccountMeta::new(authority.pubkey(), true), AccountMeta::new(delegation_pda, false)];
        if let Some(r) = self.receiver {
            accounts.push(AccountMeta::new(r, false));
        }
        Instruction { program_id: PROGRAM_ID, accounts, data: vec![*revoke_delegation::DISCRIMINATOR] }
    }

}

/// The send-driving `execute` routes through the engine-neutral [`TestSVM::send`]
/// for silent setup sends. The World path builds `.instruction()` and routes the
/// send through the observing backend so the surface is captured.
impl<'a, B: TestSVM> RevokeDelegation<'a, B> {
    pub fn execute(self) -> model::Transaction {
        let authority = self.signer.unwrap_or(self.delegator);
        let ix = self.instruction();
        self.svm.send(&[ix], &[authority]).assert_success()
    }
}

pub struct CloseSubscriptionAuthority<'a, B: TestSVM> {
    svm: &'a mut B,
    user: &'a Keypair,
    mint: Pubkey,
    custom_pda: Option<Pubkey>,
    receiver: Option<Pubkey>,
}

impl<'a, B: TestSVM> CloseSubscriptionAuthority<'a, B> {
    pub fn new(svm: &'a mut B, user: &'a Keypair, mint: Pubkey) -> Self {
        Self { svm, user, mint, custom_pda: None, receiver: None }
    }

    pub fn pda(mut self, pda: Pubkey) -> Self {
        self.custom_pda = Some(pda);
        self
    }

    pub fn receiver(mut self, receiver: Pubkey) -> Self {
        self.receiver = Some(receiver);
        self
    }

    /// The `CloseSubscriptionAuthority` instruction, built but not sent.
    pub fn instruction(&self) -> Instruction {
        let (derived_pda, _) = get_subscription_authority_pda(&self.user.pubkey(), &self.mint);
        let subscription_authority_pda = self.custom_pda.unwrap_or(derived_pda);

        let mut accounts =
            vec![AccountMeta::new(self.user.pubkey(), true), AccountMeta::new(subscription_authority_pda, false)];
        if let Some(receiver) = self.receiver {
            accounts.push(AccountMeta::new(receiver, false));
        }
        Instruction { program_id: PROGRAM_ID, accounts, data: vec![*close_subscription_authority::DISCRIMINATOR] }
    }

}

/// The send-driving `execute` routes through the engine-neutral [`TestSVM::send`]
/// for silent setup sends. The World path builds `.instruction()` and routes the
/// send through the observing backend so the surface is captured.
impl<'a, B: TestSVM> CloseSubscriptionAuthority<'a, B> {
    pub fn execute(self) -> model::Transaction {
        let ix = self.instruction();
        self.svm.send(&[ix], &[self.user]).assert_success()
    }
}

pub struct RevokeSubscriptionAuthority<'a, B: TestSVM> {
    svm: &'a mut B,
    user: &'a Keypair,
    mint: Pubkey,
    custom_ata: Option<Pubkey>,
}

impl<'a, B: TestSVM> RevokeSubscriptionAuthority<'a, B> {
    pub fn new(svm: &'a mut B, user: &'a Keypair, mint: Pubkey) -> Self {
        Self { svm, user, mint, custom_ata: None }
    }

    pub fn ata(mut self, ata: Pubkey) -> Self {
        self.custom_ata = Some(ata);
        self
    }

    /// The `RevokeSubscriptionAuthority` instruction, built but not sent.
    pub fn instruction(&self) -> Instruction {
        let token_program = self.svm.get_account(&self.mint).unwrap().owner;
        let derived_ata = get_associated_token_address_with_program_id(&self.user.pubkey(), &self.mint, &token_program);
        let user_ata = self.custom_ata.unwrap_or(derived_ata);

        let accounts = vec![
            AccountMeta::new_readonly(self.user.pubkey(), true),
            AccountMeta::new(user_ata, false),
            AccountMeta::new_readonly(self.mint, false),
            AccountMeta::new_readonly(token_program, false),
        ];
        Instruction { program_id: PROGRAM_ID, accounts, data: vec![*revoke_subscription_authority::DISCRIMINATOR] }
    }

}

/// The send-driving `execute` routes through the engine-neutral [`TestSVM::send`]
/// for silent setup sends. The World path builds `.instruction()` and routes the
/// send through the observing backend so the surface is captured.
impl<'a, B: TestSVM> RevokeSubscriptionAuthority<'a, B> {
    pub fn execute(self) -> model::Transaction {
        let ix = self.instruction();
        self.svm.send(&[ix], &[self.user]).assert_success()
    }
}

pub struct RevokeAbandonedDelegation<'a, B: TestSVM> {
    svm: &'a mut B,
    payer: &'a Keypair,
    delegator: Pubkey,
    mint: Pubkey,
    delegatee: Pubkey,
    nonce: u64,
    custom_pda: Option<Pubkey>,
    custom_authority: Option<Pubkey>,
}

impl<'a, B: TestSVM> RevokeAbandonedDelegation<'a, B> {
    pub fn new(
        svm: &'a mut B,
        payer: &'a Keypair,
        delegator: Pubkey,
        mint: Pubkey,
        delegatee: Pubkey,
    ) -> Self {
        Self { svm, payer, delegator, mint, delegatee, nonce: 0, custom_pda: None, custom_authority: None }
    }

    pub fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = nonce;
        self
    }

    pub fn pda(mut self, pda: Pubkey) -> Self {
        self.custom_pda = Some(pda);
        self
    }

    pub fn authority(mut self, authority: Pubkey) -> Self {
        self.custom_authority = Some(authority);
        self
    }

    /// The `RevokeAbandonedDelegation` instruction, built but not sent.
    pub fn instruction(&self) -> Instruction {
        let (derived_authority, _) = get_subscription_authority_pda(&self.delegator, &self.mint);
        let subscription_authority_pda = self.custom_authority.unwrap_or(derived_authority);
        let (derived_pda, _) = get_delegation_pda(&derived_authority, &self.delegator, &self.delegatee, self.nonce);
        let delegation_pda = self.custom_pda.unwrap_or(derived_pda);

        let accounts = vec![
            AccountMeta::new(self.payer.pubkey(), true),
            AccountMeta::new(delegation_pda, false),
            AccountMeta::new_readonly(subscription_authority_pda, false),
        ];
        Instruction { program_id: PROGRAM_ID, accounts, data: vec![*revoke_abandoned_delegation::DISCRIMINATOR] }
    }

}

/// The send-driving `execute` routes through the engine-neutral [`TestSVM::send`]
/// for silent setup sends. The World path builds `.instruction()` and routes the
/// send through the observing backend so the surface is captured.
impl<'a, B: TestSVM> RevokeAbandonedDelegation<'a, B> {
    pub fn execute(self) -> model::Transaction {
        let ix = self.instruction();
        self.svm.send(&[ix], &[self.payer]).assert_success()
    }
}

pub struct CreatePlan<'a, B: TestSVM> {
    svm: &'a mut B,
    owner: &'a Keypair,
    data: PlanData,
    destinations_vec: Vec<Pubkey>,
    pullers_vec: Vec<Pubkey>,
    custom_pda: Option<Pubkey>,
}

impl<'a, B: TestSVM> CreatePlan<'a, B> {
    pub fn new(svm: &'a mut B, owner: &'a Keypair, mint: Pubkey) -> Self {
        let zero_addr: pinocchio::Address = [0u8; 32].into();
        Self {
            svm,
            owner,
            data: PlanData {
                plan_id: 0,
                mint: mint.to_bytes().into(),
                terms: PlanTerms { amount: 0, period_hours: 0, created_at: 0 },
                end_ts: 0,
                destinations: [zero_addr; MAX_DESTINATIONS],
                pullers: [zero_addr; MAX_PULLERS],
                metadata_uri: [0u8; 128],
            },
            destinations_vec: vec![],
            pullers_vec: vec![],
            custom_pda: None,
        }
    }

    pub fn plan_id(mut self, plan_id: u64) -> Self {
        self.data.plan_id = plan_id;
        self
    }

    pub fn amount(mut self, amount: u64) -> Self {
        self.data.terms.amount = amount;
        self
    }

    pub fn period_hours(mut self, period_hours: u64) -> Self {
        self.data.terms.period_hours = period_hours;
        self
    }

    pub fn end_ts(mut self, end_ts: i64) -> Self {
        self.data.end_ts = end_ts;
        self
    }

    pub fn destinations(mut self, destinations: Vec<Pubkey>) -> Self {
        self.destinations_vec = destinations;
        self
    }

    pub fn pullers(mut self, pullers: Vec<Pubkey>) -> Self {
        self.pullers_vec = pullers;
        self
    }

    pub fn metadata_uri(mut self, uri: &str) -> Self {
        let bytes = uri.as_bytes();
        let len = bytes.len().min(128);
        self.data.metadata_uri[..len].copy_from_slice(&bytes[..len]);
        self
    }

    pub fn pda(mut self, pda: Pubkey) -> Self {
        self.custom_pda = Some(pda);
        self
    }

    /// The plan PDA this builder targets (custom override or derived).
    pub fn plan_pda(&self) -> Pubkey {
        self.custom_pda.unwrap_or_else(|| get_plan_pda(&self.owner.pubkey(), self.data.plan_id).0)
    }

    /// Construct the `CreatePlan` instruction without sending it, so callers that
    /// want to route the send through an observing backend can reuse the exact
    /// serialization (the raw `PlanData` layout) the suite relies on.
    pub fn instruction(&self) -> Instruction {
        let mut plan = self.data.clone();

        assert!(self.destinations_vec.len() <= MAX_DESTINATIONS, "max {MAX_DESTINATIONS} destinations");
        let mut destinations = [[0u8; 32]; MAX_DESTINATIONS];
        for (i, d) in self.destinations_vec.iter().enumerate() {
            destinations[i] = d.to_bytes();
        }
        plan.destinations = destinations.map(|d| d.into());

        assert!(self.pullers_vec.len() <= MAX_PULLERS, "max {MAX_PULLERS} pullers");
        let mut pullers = [[0u8; 32]; MAX_PULLERS];
        for (i, p) in self.pullers_vec.iter().enumerate() {
            pullers[i] = p.to_bytes();
        }
        plan.pullers = pullers.map(|p| p.into());

        let plan_data_bytes =
            unsafe { std::slice::from_raw_parts(&plan as *const PlanData as *const u8, PlanData::LEN) };

        let mut data = vec![*create_plan::DISCRIMINATOR];
        data.extend_from_slice(plan_data_bytes);

        let mint_pubkey = Pubkey::new_from_array(self.data.mint.to_bytes());
        let token_program = self
            .svm
            .get_account(&mint_pubkey)
            .map(|a| a.owner)
            .unwrap_or(crate::tests::constants::TOKEN_PROGRAM_ID);

        let accounts = vec![
            AccountMeta::new(self.owner.pubkey(), true),
            AccountMeta::new(self.plan_pda(), false),
            AccountMeta::new_readonly(mint_pubkey, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(token_program, false),
        ];

        Instruction { program_id: PROGRAM_ID, accounts, data }
    }

}

/// The send-driving `execute` routes through the engine-neutral [`TestSVM::send`]
/// for silent setup sends. The World path builds `.instruction()` and routes the
/// send through the observing backend so the surface is captured.
impl<'a, B: TestSVM> CreatePlan<'a, B> {
    pub fn execute(self) -> (model::Transaction, Pubkey) {
        let ix = self.instruction();
        let plan_pda = self.plan_pda();
        (self.svm.send(&[ix], &[self.owner]).assert_success(), plan_pda)
    }
}

pub struct UpdatePlan<'a, B: TestSVM> {
    svm: &'a mut B,
    owner: &'a Keypair,
    plan_pda: Pubkey,
    pullers_vec: Vec<Pubkey>,
    data: UpdatePlanData,
}

impl<'a, B: TestSVM> UpdatePlan<'a, B> {
    pub fn new(svm: &'a mut B, owner: &'a Keypair, plan_pda: Pubkey) -> Self {
        let zero_addr: pinocchio::Address = [0u8; 32].into();
        Self {
            svm,
            owner,
            plan_pda,
            pullers_vec: vec![],
            data: UpdatePlanData {
                status: PlanStatus::Active as u8,
                end_ts: 0,
                pullers: [zero_addr; MAX_PULLERS],
                metadata_uri: [0u8; 128],
            },
        }
    }

    pub fn status(mut self, status: PlanStatus) -> Self {
        self.data.status = status as u8;
        self
    }

    pub fn status_raw(mut self, status: u8) -> Self {
        self.data.status = status;
        self
    }

    pub fn end_ts(mut self, end_ts: i64) -> Self {
        self.data.end_ts = end_ts;
        self
    }

    pub fn pullers(mut self, pullers: Vec<Pubkey>) -> Self {
        self.pullers_vec = pullers;
        self
    }

    pub fn metadata_uri(mut self, uri: &str) -> Self {
        let bytes = uri.as_bytes();
        let len = bytes.len().min(128);
        self.data.metadata_uri = [0u8; 128];
        self.data.metadata_uri[..len].copy_from_slice(&bytes[..len]);
        self
    }

    /// Construct the `UpdatePlan` instruction without sending it (see
    /// [`CreatePlan::instruction`] for why this split exists).
    pub fn instruction(&self) -> Instruction {
        let mut update = self.data.clone();

        assert!(self.pullers_vec.len() <= MAX_PULLERS, "max {MAX_PULLERS} pullers");
        let mut pullers = [[0u8; 32]; MAX_PULLERS];
        for (i, p) in self.pullers_vec.iter().enumerate() {
            pullers[i] = p.to_bytes();
        }
        update.pullers = pullers.map(|p| p.into());

        let data_bytes =
            unsafe { std::slice::from_raw_parts(&update as *const UpdatePlanData as *const u8, UpdatePlanData::LEN) };

        let mut data = vec![*update_plan::DISCRIMINATOR];
        data.extend_from_slice(data_bytes);

        let accounts = vec![AccountMeta::new(self.owner.pubkey(), true), AccountMeta::new(self.plan_pda, false)];

        Instruction { program_id: PROGRAM_ID, accounts, data }
    }

}

/// The send-driving `execute` routes through the engine-neutral [`TestSVM::send`]
/// for silent setup sends. The World path builds `.instruction()` and routes the
/// send through the observing backend so the surface is captured.
impl<'a, B: TestSVM> UpdatePlan<'a, B> {
    pub fn execute(self) -> model::Transaction {
        let ix = self.instruction();
        self.svm.send(&[ix], &[self.owner]).assert_success()
    }
}

pub struct DeletePlan<'a, B: TestSVM> {
    svm: &'a mut B,
    owner: &'a Keypair,
    plan_pda: Pubkey,
}

impl<'a, B: TestSVM> DeletePlan<'a, B> {
    pub fn new(svm: &'a mut B, owner: &'a Keypair, plan_pda: Pubkey) -> Self {
        Self { svm, owner, plan_pda }
    }

    /// The `DeletePlan` instruction, built but not sent.
    pub fn instruction(&self) -> Instruction {
        let accounts = vec![AccountMeta::new(self.owner.pubkey(), true), AccountMeta::new(self.plan_pda, false)];
        Instruction { program_id: PROGRAM_ID, accounts, data: vec![*delete_plan::DISCRIMINATOR] }
    }

}

/// The send-driving `execute` routes through the engine-neutral [`TestSVM::send`]
/// for silent setup sends. The World path builds `.instruction()` and routes the
/// send through the observing backend so the surface is captured.
impl<'a, B: TestSVM> DeletePlan<'a, B> {
    pub fn execute(self) -> model::Transaction {
        let ix = self.instruction();
        self.svm.send(&[ix], &[self.owner]).assert_success()
    }
}

pub struct CreateSubscription<'a, B: TestSVM> {
    svm: &'a mut B,
    plan_pda: Pubkey,
    subscriber: Pubkey,
    mint: Pubkey,
    period_start_ts: i64,
    amount_pulled: u64,
    expires_at_ts: i64,
    terms: PlanTerms,
}

impl<'a, B: TestSVM> CreateSubscription<'a, B> {
    pub fn new(
        svm: &'a mut B,
        plan_pda: Pubkey,
        subscriber: Pubkey,
        mint: Pubkey,
        period_start_ts: i64,
    ) -> Self {
        Self {
            svm,
            plan_pda,
            subscriber,
            mint,
            period_start_ts,
            amount_pulled: 0,
            expires_at_ts: 0,
            terms: PlanTerms { amount: 0, period_hours: 0, created_at: 0 },
        }
    }

    pub fn amount_pulled(mut self, amount_pulled: u64) -> Self {
        self.amount_pulled = amount_pulled;
        self
    }

    pub fn expires_at_ts(mut self, expires_at_ts: i64) -> Self {
        self.expires_at_ts = expires_at_ts;
        self
    }

    pub fn terms(mut self, terms: PlanTerms) -> Self {
        self.terms = terms;
        self
    }

    pub fn execute(self) -> Pubkey {
        use crate::{
            state::{common::AccountDiscriminator, versioning::CURRENT_VERSION},
            tests::pda::{get_subscription_authority_pda, get_subscription_pda},
            Header, SubscriptionAuthority, SubscriptionDelegation,
        };

        let (md_pda, _) = get_subscription_authority_pda(&self.subscriber, &self.mint);
        let md_account = self.svm.get_account(&md_pda).unwrap();
        let md = SubscriptionAuthority::load(&md_account.data).unwrap();
        let init_id = md.init_id;

        let (subscription_pda, bump) = get_subscription_pda(&self.plan_pda, &self.subscriber);

        let subscription = SubscriptionDelegation {
            header: Header {
                discriminator: AccountDiscriminator::SubscriptionDelegation as u8,
                version: CURRENT_VERSION,
                bump,
                delegator: self.subscriber.to_bytes().into(),
                delegatee: self.plan_pda.to_bytes().into(),
                payer: self.subscriber.to_bytes().into(),
                init_id,
            },
            terms: self.terms,
            amount_pulled_in_period: self.amount_pulled,
            current_period_start_ts: self.period_start_ts,
            expires_at_ts: self.expires_at_ts,
        };

        let data = unsafe {
            std::slice::from_raw_parts(
                &subscription as *const SubscriptionDelegation as *const u8,
                SubscriptionDelegation::LEN,
            )
        };

        let lamports = rent_exempt_lamports(data.len());
        self.svm.set_account(
            &subscription_pda,
            Account { lamports, data: data.to_vec(), owner: PROGRAM_ID, executable: false, rent_epoch: 0 },
        );

        subscription_pda
    }
}

pub struct TransferSubscription<'a, B: TestSVM> {
    svm: &'a mut B,
    caller: &'a Keypair,
    delegator: Pubkey,
    mint: Pubkey,
    subscription_pda: Pubkey,
    plan_pda: Pubkey,
    amount: u64,
    receiver: Option<Pubkey>,
}

impl<'a, B: TestSVM> TransferSubscription<'a, B> {
    pub fn new(
        svm: &'a mut B,
        caller: &'a Keypair,
        delegator: Pubkey,
        mint: Pubkey,
        subscription_pda: Pubkey,
        plan_pda: Pubkey,
    ) -> Self {
        Self { svm, caller, delegator, mint, subscription_pda, plan_pda, amount: 0, receiver: None }
    }

    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = amount;
        self
    }

    pub fn to(mut self, receiver: Pubkey) -> Self {
        self.receiver = Some(receiver);
        self
    }

    /// The `TransferSubscription` instruction, built but not sent.
    pub fn instruction(&self) -> Instruction {
        let token_program = self.svm.get_account(&self.mint).unwrap().owner;
        let (subscription_authority_pda, _) = get_subscription_authority_pda(&self.delegator, &self.mint);
        let delegator_ata = get_associated_token_address_with_program_id(&self.delegator, &self.mint, &token_program);

        let receiver_ata = self.receiver.unwrap_or_else(|| {
            get_associated_token_address_with_program_id(&self.caller.pubkey(), &self.mint, &token_program)
        });

        let event_authority = Pubkey::new_from_array(event_authority_pda::ID.to_bytes());

        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(self.subscription_pda, false),
                AccountMeta::new_readonly(self.plan_pda, false),
                AccountMeta::new_readonly(subscription_authority_pda, false),
                AccountMeta::new(delegator_ata, false),
                AccountMeta::new(receiver_ata, false),
                AccountMeta::new_readonly(self.caller.pubkey(), true),
                AccountMeta::new_readonly(self.mint, false),
                AccountMeta::new_readonly(token_program, false),
                AccountMeta::new_readonly(event_authority, false),
                AccountMeta::new_readonly(PROGRAM_ID, false),
            ],
            data: [
                vec![*transfer_subscription::DISCRIMINATOR],
                self.amount.to_le_bytes().to_vec(),
                self.delegator.to_bytes().to_vec(),
                self.mint.to_bytes().to_vec(),
            ]
            .concat(),
        }
    }

}

/// The send-driving `execute` routes through the engine-neutral [`TestSVM::send`]
/// for silent setup sends. The World path builds `.instruction()` and routes the
/// send through the observing backend so the surface is captured.
impl<'a, B: TestSVM> TransferSubscription<'a, B> {
    pub fn execute(self) -> model::Transaction {
        let ix = self.instruction();
        self.svm.send(&[ix], &[self.caller]).assert_success()
    }
}

pub struct Subscribe<'a, B: TestSVM> {
    svm: &'a mut B,
    subscriber: &'a Keypair,
    merchant: Pubkey,
    plan_pda: Pubkey,
    plan_id: u64,
    plan_bump: u8,
    mint: Pubkey,
    payer: Option<&'a Keypair>,
}

impl<'a, B: TestSVM> Subscribe<'a, B> {
    pub fn new(
        svm: &'a mut B,
        subscriber: &'a Keypair,
        merchant: Pubkey,
        plan_pda: Pubkey,
        plan_id: u64,
        plan_bump: u8,
        mint: Pubkey,
    ) -> Self {
        Self { svm, subscriber, merchant, plan_pda, plan_id, plan_bump, mint, payer: None }
    }

    pub fn payer(mut self, payer: &'a Keypair) -> Self {
        self.payer = Some(payer);
        self
    }

    /// The subscription PDA this subscribe will create.
    pub fn subscription_pda(&self) -> Pubkey {
        get_subscription_pda(&self.plan_pda, &self.subscriber.pubkey()).0
    }

    /// The `Subscribe` instruction, built but not sent. The optional sponsor
    /// payer is appended as a writable signer; the caller supplies the matching
    /// signer (sponsor first) when one pays.
    pub fn instruction(&self) -> Instruction {
        let (subscription_authority_pda, _) = get_subscription_authority_pda(&self.subscriber.pubkey(), &self.mint);
        let (subscription_pda, _) = get_subscription_pda(&self.plan_pda, &self.subscriber.pubkey());

        let event_authority = Pubkey::new_from_array(event_authority_pda::ID.to_bytes());

        let mut accounts = vec![
            AccountMeta::new(self.subscriber.pubkey(), true),
            AccountMeta::new_readonly(self.merchant, false),
            AccountMeta::new_readonly(self.plan_pda, false),
            AccountMeta::new(subscription_pda, false),
            AccountMeta::new_readonly(subscription_authority_pda, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(event_authority, false),
            AccountMeta::new_readonly(PROGRAM_ID, false),
        ];
        if let Some(p) = self.payer {
            accounts.push(AccountMeta::new(p.pubkey(), true));
        }

        // Snapshot live plan terms to bind subscriber consent.
        let plan_account = self.svm.get_account(&self.plan_pda).unwrap();
        let plan = crate::state::Plan::load(&plan_account.data).unwrap();
        let expected_amount = plan.data.terms.amount;
        let expected_period_hours = plan.data.terms.period_hours;
        let expected_created_at = plan.data.terms.created_at;
        let expected_mint = plan.data.mint;
        let expected_subscription_authority_init_id = self
            .svm
            .get_account(&subscription_authority_pda)
            .and_then(|account| {
                crate::state::SubscriptionAuthority::load(&account.data).ok().map(|authority| authority.init_id)
            })
            .unwrap_or_default();

        let data = [
            vec![*subscribe::DISCRIMINATOR],
            self.plan_id.to_le_bytes().to_vec(),
            vec![self.plan_bump],
            expected_mint.as_ref().to_vec(),
            expected_amount.to_le_bytes().to_vec(),
            expected_period_hours.to_le_bytes().to_vec(),
            expected_created_at.to_le_bytes().to_vec(),
            expected_subscription_authority_init_id.to_le_bytes().to_vec(),
        ]
        .concat();

        Instruction { program_id: PROGRAM_ID, accounts, data }
    }

}

/// The send-driving `execute` routes through the engine-neutral [`TestSVM::send`]
/// for silent setup sends. The World path builds `.instruction()` and routes the
/// send through the observing backend so the surface is captured.
impl<'a, B: TestSVM> Subscribe<'a, B> {
    pub fn execute(self) -> model::Transaction {
        let ix = self.instruction();
        // `signers[0]` is the fee payer: the sponsor pays when one is set.
        let mut signers: Vec<&Keypair> = vec![self.subscriber];
        if let Some(p) = self.payer {
            signers.insert(0, p);
        }
        self.svm.send(&[ix], &signers).assert_success()
    }
}

pub struct CancelSubscription<'a, B: TestSVM> {
    svm: &'a mut B,
    subscriber: &'a Keypair,
    plan_pda: Pubkey,
    subscription_pda: Pubkey,
}

impl<'a, B: TestSVM> CancelSubscription<'a, B> {
    pub fn new(svm: &'a mut B, subscriber: &'a Keypair, plan_pda: Pubkey, subscription_pda: Pubkey) -> Self {
        Self { svm, subscriber, plan_pda, subscription_pda }
    }

    /// The `CancelSubscription` instruction, built but not sent.
    pub fn instruction(&self) -> Instruction {
        let event_authority = Pubkey::new_from_array(event_authority_pda::ID.to_bytes());
        let accounts = vec![
            AccountMeta::new_readonly(self.subscriber.pubkey(), true),
            AccountMeta::new_readonly(self.plan_pda, false),
            AccountMeta::new(self.subscription_pda, false),
            AccountMeta::new_readonly(event_authority, false),
            AccountMeta::new_readonly(PROGRAM_ID, false),
        ];
        Instruction { program_id: PROGRAM_ID, accounts, data: vec![*cancel_subscription::DISCRIMINATOR] }
    }

}

/// The send-driving `execute` routes through the engine-neutral [`TestSVM::send`]
/// for silent setup sends. The World path builds `.instruction()` and routes the
/// send through the observing backend so the surface is captured.
impl<'a, B: TestSVM> CancelSubscription<'a, B> {
    pub fn execute(self) -> model::Transaction {
        let ix = self.instruction();
        self.svm.send(&[ix], &[self.subscriber]).assert_success()
    }
}

pub struct ResumeSubscription<'a, B: TestSVM> {
    svm: &'a mut B,
    subscriber: &'a Keypair,
    plan_pda: Pubkey,
    subscription_pda: Pubkey,
}

impl<'a, B: TestSVM> ResumeSubscription<'a, B> {
    pub fn new(svm: &'a mut B, subscriber: &'a Keypair, plan_pda: Pubkey, subscription_pda: Pubkey) -> Self {
        Self { svm, subscriber, plan_pda, subscription_pda }
    }

    /// The `ResumeSubscription` instruction, built but not sent.
    pub fn instruction(&self) -> Instruction {
        let event_authority = Pubkey::new_from_array(event_authority_pda::ID.to_bytes());
        let accounts = vec![
            AccountMeta::new_readonly(self.subscriber.pubkey(), true),
            AccountMeta::new_readonly(self.plan_pda, false),
            AccountMeta::new(self.subscription_pda, false),
            AccountMeta::new_readonly(event_authority, false),
            AccountMeta::new_readonly(PROGRAM_ID, false),
        ];
        Instruction { program_id: PROGRAM_ID, accounts, data: vec![*resume_subscription::DISCRIMINATOR] }
    }

}

/// The send-driving `execute` routes through the engine-neutral [`TestSVM::send`]
/// for silent setup sends. The World path builds `.instruction()` and routes the
/// send through the observing backend so the surface is captured.
impl<'a, B: TestSVM> ResumeSubscription<'a, B> {
    pub fn execute(self) -> model::Transaction {
        let ix = self.instruction();
        self.svm.send(&[ix], &[self.subscriber]).assert_success()
    }
}

pub struct RevokeSubscription<'a, B: TestSVM> {
    svm: &'a mut B,
    authority: &'a Keypair,
    subscription_pda: Pubkey,
    plan_pda: Pubkey,
    receiver: Option<Pubkey>,
}

impl<'a, B: TestSVM> RevokeSubscription<'a, B> {
    pub fn new(svm: &'a mut B, authority: &'a Keypair, subscription_pda: Pubkey, plan_pda: Pubkey) -> Self {
        Self { svm, authority, subscription_pda, plan_pda, receiver: None }
    }

    pub fn receiver(mut self, receiver: Pubkey) -> Self {
        self.receiver = Some(receiver);
        self
    }

    /// The `RevokeSubscription` instruction, built but not sent. (It shares the
    /// `revoke_delegation` discriminator; the program dispatches on the account
    /// shape.)
    pub fn instruction(&self) -> Instruction {
        let mut accounts = vec![
            AccountMeta::new(self.authority.pubkey(), true),
            AccountMeta::new(self.subscription_pda, false),
            AccountMeta::new_readonly(self.plan_pda, false),
        ];
        if let Some(receiver) = self.receiver {
            accounts.push(AccountMeta::new(receiver, false));
        }
        Instruction { program_id: PROGRAM_ID, accounts, data: vec![*revoke_delegation::DISCRIMINATOR] }
    }

}

/// The send-driving `execute` routes through the engine-neutral [`TestSVM::send`]
/// for silent setup sends. The World path builds `.instruction()` and routes the
/// send through the observing backend so the surface is captured.
impl<'a, B: TestSVM> RevokeSubscription<'a, B> {
    pub fn execute(self) -> model::Transaction {
        let ix = self.instruction();
        self.svm.send(&[ix], &[self.authority]).assert_success()
    }
}
