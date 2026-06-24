//! `initialize_subscription_authority`, bound to the litesvm engine.
//!
//! The bodies live in `scenarios::suite::initialize_subscription_authority`
//! (engine-neutral, generic over `B: TestSVM`). This module is the litesvm
//! binding: `bind_scenarios!` emits one `#[test]` per plain scenario, each
//! calling the generic body with a fresh `make_backend()`. The rendered reports
//! are byte-identical to the old `test_initialize_subscription_authority.rs`,
//! since the titles, intents, and bodies are unchanged.
//!
//! The Token-2022 case is an `rstest`-parametrized test, which `bind_scenarios!`
//! cannot emit (the macro produces one bare `#[test] fn`). Its `#[rstest]` shim
//! stays here, dispatching every case into the generic worker
//! `initialize_subscription_authority_token_2022_case`; the case names and
//! per-case report titles match the pre-lift suite.

use rstest::rstest;
use spl_token_2022_interface::extension::ExtensionType;

use scenarios::suite::initialize_subscription_authority::initialize_subscription_authority_token_2022_case;
use crate::SubscriptionsError;

scenarios::bind_scenarios!(
    crate::tests::utils::make_backend;
    initialize_subscription_authority;
    initialize_subscription_authority,
    initialize_subscription_authority_rejects_non_canonical_token_account,
    initialize_subscription_authority_with_sponsor,
    initialize_subscription_authority_allows_active_transfer_hook,
    initialize_subscription_authority_allows_mutable_inactive_transfer_hook,
    wrong_token_program_returns_error,
    initialize_subscription_authority_with_prefunded_pda,
    initialize_subscription_authority_with_overfunded_pda,
    writable_accounts_must_be_writable,
    signer_accounts_must_be_signers,
    non_signer_payer_rejected,
);

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
    initialize_subscription_authority_token_2022_case(crate::tests::utils::make_backend(), extensions, expected_error);
}
