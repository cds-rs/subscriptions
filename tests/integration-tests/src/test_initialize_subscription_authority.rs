//! `initialize_subscription_authority`, converted to the World/scenario pattern.
//!
//! Each test builds a `World`, draws its actors from the cast (`alice` the
//! principal, `sponsor` the payer, `mallory` the adversary), runs setup through
//! the fabrication helpers, and performs the on-chain action through the
//! `init_authority` verb or the observed `send_*`. Every send renders its
//! surface into the test's report under `target/md-reports/`.

use rstest::rstest;
use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use spl_token_2022_interface::extension::ExtensionType;

use litesvm_utils::TestSVM;

use crate::{
    instructions::initialize_subscription_authority,
    tests::{
        constants::{MINT_DECIMALS, PROGRAM_ID, SYSTEM_PROGRAM_ID, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID},
        idl,
        pda::get_subscription_authority_pda,
        utils::{as_pubkey, 
            fetch_account, init_aux_token_account, init_mint, set_transfer_hook_config, ObservedResultExt,
            make_backend, ModelTxExt, World,
        },
    },
    AccountDiscriminator, SubscriptionAuthority, SubscriptionsError,
};

#[test]
fn initialize_subscription_authority() {
    let mut world = World::new(make_backend(), 
        "Initialize a subscription authority",
        "Alice initializes her SubscriptionAuthority and delegates her ATA to it",
    );
    let alice = world.actor("alice");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let user_ata = world.fund_ata(mint, &alice, 1_000_000);

    world.md().step("Alice initializes her subscription authority");
    let (res, subscription_authority_pda, bump) = world.init_authority(&alice, mint, None);
    res.assert_ok();

    let account = world.svm().get_account(&subscription_authority_pda).unwrap();
    let subscription_authority = SubscriptionAuthority::load(&account.data).unwrap();

    world.md().check(
        "the account is tagged SubscriptionAuthority",
        AccountDiscriminator::SubscriptionAuthority as u8,
        subscription_authority.discriminator,
    );
    world.md().check("the authority's user is Alice", alice.pubkey(), as_pubkey(subscription_authority.user.to_bytes()));
    world.md().check("the authority's mint is the USDC mint", mint, as_pubkey(subscription_authority.token_mint.to_bytes()));
    // Default payer is the user when no sponsor is supplied.
    world.md().check("the payer defaults to Alice", alice.pubkey(), as_pubkey(subscription_authority.payer.to_bytes()));
    assert_eq!(subscription_authority.bump, bump);
    assert!(subscription_authority.init_id >= 0);

    // The ATA is now delegated to the authority for the full balance.
    let ata_account = fetch_account::<spl_token_2022_interface::state::Account, _>(world.svm(), &user_ata);
    assert!(ata_account.delegate.is_some());
    world.md().check("the ATA is delegated to the authority", subscription_authority_pda, ata_account.delegate.unwrap());
    world.md().check("the delegated amount is u64::MAX", u64::MAX, ata_account.delegated_amount);
}

#[test]
fn initialize_subscription_authority_rejects_non_canonical_token_account() {
    let mut world = World::new(make_backend(), 
        "Reject a non-canonical token account",
        "an auxiliary (non-ATA) token account is rejected where the canonical ATA is required",
    );
    let alice = world.actor("alice");

    let mint = world.usdc_mint(&alice);
    let aux_token_account = init_aux_token_account(world.svm_mut(), mint, alice.pubkey(), 1_000_000);
    let (subscription_authority_pda, _) = get_subscription_authority_pda(&alice.pubkey(), &mint);

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(alice.pubkey(), true),
            AccountMeta::new(subscription_authority_pda, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(aux_token_account, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        ],
        data: vec![*initialize_subscription_authority::DISCRIMINATOR],
    };

    world.md().step("Alice points the instruction at an auxiliary token account, not her ATA");
    world.send_err(
        &[ix],
        &[&alice],
        "InitSubscriptionAuthority (non-canonical ATA)",
        SubscriptionsError::InvalidAssociatedTokenAccountDerivedAddress,
    );
}

#[test]
fn initialize_subscription_authority_with_sponsor() {
    let mut world = World::new(make_backend(), 
        "Initialize with a sponsor",
        "a sponsor pays rent and the fee; Alice's lamports stay untouched",
    );
    let alice = world.actor("alice");
    let sponsor = world.actor("sponsor");

    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    let user_ata = world.fund_ata(mint, &alice, 1_000_000);

    let user_balance_before = world.svm().get_account(&alice.pubkey()).unwrap().lamports;
    let sponsor_balance_before = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;

    world.md().step("Alice initializes her authority; the sponsor pays");
    let (res, subscription_authority_pda, _bump) = world.init_authority(&alice, mint, Some(&sponsor));
    res.assert_ok();

    let account = world.svm().get_account(&subscription_authority_pda).unwrap();
    let md = SubscriptionAuthority::load(&account.data).unwrap();
    world.md().check("the authority's user is Alice", alice.pubkey(), as_pubkey(md.user.to_bytes()));
    world.md().check("the authority's payer is the sponsor", sponsor.pubkey(), as_pubkey(md.payer.to_bytes()));

    // Sponsor pays both rent and the transaction fee. User must not be charged.
    let user_balance_after = world.svm().get_account(&alice.pubkey()).unwrap().lamports;
    let sponsor_balance_after = world.svm().get_account(&sponsor.pubkey()).unwrap().lamports;
    world.md().check("Alice's lamports are untouched", user_balance_before, user_balance_after);
    world.md().check("the sponsor was charged", true, sponsor_balance_after < sponsor_balance_before);

    // Verify Approve still went through with user as the ATA authority.
    let ata_account = fetch_account::<spl_token_2022_interface::state::Account, _>(world.svm(), &user_ata);
    assert_eq!(ata_account.delegate.unwrap(), subscription_authority_pda);
    assert_eq!(ata_account.delegated_amount, u64::MAX);
}

#[rstest]
#[case::no_extensions(&[], None)]
#[case::confidential_transfer(&[ExtensionType::ConfidentialTransferMint], None)]
#[case::non_transferable(&[ExtensionType::NonTransferable], None)]
#[case::permanent_delegate(&[ExtensionType::PermanentDelegate], None)]
#[case::transfer_fee(&[ExtensionType::TransferFeeConfig], None)]
#[case::transfer_hook_unconfigured(&[ExtensionType::TransferHook], None)]
#[case::pausable(&[ExtensionType::Pausable], None)]
#[case::close_authority(&[ExtensionType::MintCloseAuthority], None)]
#[case::mixed_allowed(&[ExtensionType::TransferFeeConfig, ExtensionType::TransferHook], None)]
#[case::mixed_allowed_confidential(&[ExtensionType::MintCloseAuthority, ExtensionType::ConfidentialTransferMint], None)]
fn initialize_subscription_authority_token_2022(
    #[case] extensions: &[ExtensionType],
    #[case] expected_error: Option<SubscriptionsError>,
) {
    let mut world = World::new(make_backend(), 
        &format!("Initialize over a Token-2022 mint ({extensions:?})"),
        "the authority initializes over a Token-2022 mint with the given extensions",
    );
    let alice = world.actor("alice");

    let mint =
        init_mint(world.svm_mut(), TOKEN_2022_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, Some(alice.pubkey()), extensions);
    world.prop(mint, "USDC mint (T22)");
    let user_ata = world.fund_ata(mint, &alice, 1_000_000);

    world.md().step("Alice initializes her authority over the Token-2022 mint");
    let (res, subscription_authority_pda, bump) = world.init_authority(&alice, mint, None);

    match expected_error {
        Some(err) => res.assert_err(err),
        None => {
            res.assert_ok();

            let account = world.svm().get_account(&subscription_authority_pda).unwrap();
            let subscription_authority = SubscriptionAuthority::load(&account.data).unwrap();

            assert_eq!(subscription_authority.discriminator, AccountDiscriminator::SubscriptionAuthority as u8);
            assert_eq!(subscription_authority.user.to_bytes(), alice.pubkey().to_bytes());
            assert_eq!(subscription_authority.token_mint.to_bytes(), mint.to_bytes());
            assert_eq!(subscription_authority.bump, bump);
            assert!(subscription_authority.init_id >= 0);

            let ata_account = fetch_account::<spl_token_2022_interface::state::Account, _>(world.svm(), &user_ata);
            assert!(ata_account.delegate.is_some());
            assert_eq!(ata_account.delegate.unwrap(), subscription_authority_pda);
            assert_eq!(ata_account.delegated_amount, u64::MAX);
        }
    }
}

#[test]
fn initialize_subscription_authority_allows_active_transfer_hook() {
    let mut world = World::new(make_backend(), 
        "Allow an active transfer hook",
        "a mint carrying a configured transfer hook is accepted",
    );
    let alice = world.actor("alice");

    let mint = init_mint(
        world.svm_mut(),
        TOKEN_2022_PROGRAM_ID,
        MINT_DECIMALS,
        1_000_000_000,
        Some(alice.pubkey()),
        &[ExtensionType::TransferHook],
    );
    set_transfer_hook_config(world.svm_mut(), mint, None, Some(Pubkey::new_unique()));
    let user_ata = world.fund_ata(mint, &alice, 1_000_000);

    world.md().step("Alice initializes over a mint with an active transfer hook");
    let (res, _pda, _bump) = world.init_authority(&alice, mint, None);
    res.assert_ok();

    let ata_account = fetch_account::<spl_token_2022_interface::state::Account, _>(world.svm(), &user_ata);
    assert!(ata_account.delegate.is_some());
}

#[test]
fn initialize_subscription_authority_allows_mutable_inactive_transfer_hook() {
    let mut world = World::new(make_backend(), 
        "Allow a mutable inactive transfer hook",
        "a mint with a mutable but unset transfer hook is accepted",
    );
    let alice = world.actor("alice");

    let mint = init_mint(
        world.svm_mut(),
        TOKEN_2022_PROGRAM_ID,
        MINT_DECIMALS,
        1_000_000_000,
        Some(alice.pubkey()),
        &[ExtensionType::TransferHook],
    );
    set_transfer_hook_config(world.svm_mut(), mint, Some(alice.pubkey()), None);
    let user_ata = world.fund_ata(mint, &alice, 1_000_000);

    world.md().step("Alice initializes over a mint with a mutable inactive transfer hook");
    let (res, _pda, _bump) = world.init_authority(&alice, mint, None);
    res.assert_ok();

    let ata_account = fetch_account::<spl_token_2022_interface::state::Account, _>(world.svm(), &user_ata);
    assert!(ata_account.delegate.is_some());
}

#[test]
fn wrong_token_program_returns_error() {
    let mut world = World::new(make_backend(), 
        "Reject a forged token program",
        "passing a non-token account in the token-program slot is rejected",
    );
    let alice = world.actor("alice");

    let mint = world.usdc_mint(&alice);
    let user_ata = world.fund_ata(mint, &alice, 1_000_000);
    let (subscription_authority_pda, _bump) = get_subscription_authority_pda(&alice.pubkey(), &mint);

    let fake_token_program = alice.pubkey();

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(alice.pubkey(), true),
            AccountMeta::new(subscription_authority_pda, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(user_ata, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(fake_token_program, false),
        ],
        data: vec![*initialize_subscription_authority::DISCRIMINATOR],
    };

    world.md().step("Alice passes her own pubkey in the token-program slot");
    let res = world.send(&[ix], &[&alice], "InitSubscriptionAuthority (forged token program)");
    world.md().check("the transaction is refused", false, res.is_success());
}

/// Pre-funding a SubscriptionAuthority PDA with lamports (a griefing attempt)
/// must not prevent the legitimate user from creating the account.
#[test]
fn initialize_subscription_authority_with_prefunded_pda() {
    let mut world = World::new(make_backend(), 
        "Survive a pre-funded PDA",
        "a griefer pre-funds the authority PDA; Alice can still initialize it",
    );
    let alice = world.actor("alice");

    let mint = world.usdc_mint(&alice);
    let user_ata = world.fund_ata(mint, &alice, 1_000_000);

    // Mallory pre-funds the PDA address with lamports.
    let (subscription_authority_pda, _) = get_subscription_authority_pda(&alice.pubkey(), &mint);
    world.svm_mut()
        .set_account(
            &subscription_authority_pda,
            Account { lamports: 1_000, data: vec![], owner: Pubkey::default(), executable: false, rent_epoch: 0 },
        );

    world.md().step("Despite the pre-funded PDA, Alice initializes her authority");
    let (res, _, bump) = world.init_authority(&alice, mint, None);
    res.assert_ok();

    let account = world.svm().get_account(&subscription_authority_pda).unwrap();
    let subscription_authority = SubscriptionAuthority::load(&account.data).unwrap();

    assert_eq!(subscription_authority.discriminator, AccountDiscriminator::SubscriptionAuthority as u8);
    assert_eq!(subscription_authority.user.to_bytes(), alice.pubkey().to_bytes());
    assert_eq!(subscription_authority.token_mint.to_bytes(), mint.to_bytes());
    assert_eq!(subscription_authority.bump, bump);
    assert!(subscription_authority.init_id >= 0);

    let ata_account = fetch_account::<spl_token_2022_interface::state::Account, _>(world.svm(), &user_ata);
    assert!(ata_account.delegate.is_some());
    assert_eq!(ata_account.delegate.unwrap(), subscription_authority_pda);
    assert_eq!(ata_account.delegated_amount, u64::MAX);
}

#[test]
fn initialize_subscription_authority_with_overfunded_pda() {
    let mut world = World::new(make_backend(), 
        "Survive an over-funded PDA",
        "a griefer over-funds the authority PDA; Alice can still initialize it",
    );
    let alice = world.actor("alice");

    let mint = world.usdc_mint(&alice);
    let user_ata = world.fund_ata(mint, &alice, 1_000_000);

    let (subscription_authority_pda, _) = get_subscription_authority_pda(&alice.pubkey(), &mint);
    world.svm_mut()
        .set_account(
            &subscription_authority_pda,
            Account { lamports: 10_000_000, data: vec![], owner: Pubkey::default(), executable: false, rent_epoch: 0 },
        );

    world.md().step("Despite the over-funded PDA, Alice initializes her authority");
    let (res, _, bump) = world.init_authority(&alice, mint, None);
    res.assert_ok();

    let account = world.svm().get_account(&subscription_authority_pda).unwrap();
    let subscription_authority = SubscriptionAuthority::load(&account.data).unwrap();

    assert_eq!(subscription_authority.discriminator, AccountDiscriminator::SubscriptionAuthority as u8);
    assert_eq!(subscription_authority.user.to_bytes(), alice.pubkey().to_bytes());
    assert_eq!(subscription_authority.token_mint.to_bytes(), mint.to_bytes());
    assert_eq!(subscription_authority.bump, bump);

    let ata_account = fetch_account::<spl_token_2022_interface::state::Account, _>(world.svm(), &user_ata);
    assert!(ata_account.delegate.is_some());
    assert_eq!(ata_account.delegate.unwrap(), subscription_authority_pda);
    assert_eq!(ata_account.delegated_amount, u64::MAX);
}

#[test]
fn writable_accounts_must_be_writable() {
    let writable = idl::writable_account_indices("initSubscriptionAuthority");

    let mut world = World::new(make_backend(), 
        "Writable accounts must be writable",
        "flipping any account the instruction writes to read-only is rejected",
    );
    let alice = world.actor("alice");
    let sponsor = world.actor("sponsor");

    let mint = world.usdc_mint(&alice);
    let user_ata = world.fund_ata(mint, &alice, 1_000_000);
    let (subscription_authority_pda, _) = get_subscription_authority_pda(&alice.pubkey(), &mint);

    for (idx, name, is_signer) in &writable {
        let mut accounts = vec![
            AccountMeta::new(alice.pubkey(), true),
            AccountMeta::new(subscription_authority_pda, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(user_ata, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        ];

        // Flip the writable account to read-only, preserving its signer flag.
        let pubkey = accounts[*idx].pubkey;
        accounts[*idx] = AccountMeta::new_readonly(pubkey, *is_signer);

        let ix = Instruction {
            program_id: PROGRAM_ID,
            accounts,
            data: vec![*initialize_subscription_authority::DISCRIMINATOR],
        };

        world.send_err(
            &[ix],
            &[&sponsor, &alice],
            &format!("InitSubscriptionAuthority ({name} forced read-only)"),
            SubscriptionsError::AccountNotWritable,
        );
    }
}

#[test]
fn signer_accounts_must_be_signers() {
    let signers = idl::signer_account_indices("initSubscriptionAuthority");

    let mut world = World::new(make_backend(), 
        "Signer accounts must sign",
        "flipping any required signer to non-signer is rejected",
    );
    let alice = world.actor("alice");
    let sponsor = world.actor("sponsor");

    let mint = world.usdc_mint(&alice);
    let user_ata = world.fund_ata(mint, &alice, 1_000_000);
    let (subscription_authority_pda, _) = get_subscription_authority_pda(&alice.pubkey(), &mint);

    for (idx, name, is_writable) in &signers {
        let mut accounts = vec![
            AccountMeta::new(alice.pubkey(), true),
            AccountMeta::new(subscription_authority_pda, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(user_ata, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        ];

        // Flip the signer to non-signer, preserving its writable flag.
        let pubkey = accounts[*idx].pubkey;
        accounts[*idx] =
            if *is_writable { AccountMeta::new(pubkey, false) } else { AccountMeta::new_readonly(pubkey, false) };

        let ix = Instruction {
            program_id: PROGRAM_ID,
            accounts,
            data: vec![*initialize_subscription_authority::DISCRIMINATOR],
        };

        world.send_err(
            &[ix],
            &[&sponsor],
            &format!("InitSubscriptionAuthority ({name} forced non-signer)"),
            SubscriptionsError::NotSigner,
        );
    }
}

/// A trailing account is interpreted as the optional sponsor payer; a non-signer
/// extra must be rejected because the payer slot requires a signer.
#[test]
fn non_signer_payer_rejected() {
    let mut world = World::new(make_backend(), 
        "Reject a non-signer payer",
        "a trailing non-signer account in the optional payer slot is rejected",
    );
    let alice = world.actor("alice");

    let mint = world.usdc_mint(&alice);
    let user_ata = world.fund_ata(mint, &alice, 1_000_000);
    let (subscription_authority_pda, _bump) = get_subscription_authority_pda(&alice.pubkey(), &mint);

    // A pubkey that is NOT a signer in this transaction.
    let extra_account = Pubkey::new_unique();

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(alice.pubkey(), true),
            AccountMeta::new(subscription_authority_pda, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(user_ata, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            AccountMeta::new_readonly(extra_account, false),
        ],
        data: vec![*initialize_subscription_authority::DISCRIMINATOR],
    };

    world.md().step("Alice appends a non-signer in the optional payer slot");
    world.send_err(
        &[ix],
        &[&alice],
        "InitSubscriptionAuthority (non-signer payer)",
        SubscriptionsError::NotSigner,
    );
}
