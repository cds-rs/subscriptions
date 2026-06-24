//! `initialize_subscription_authority`, bound to the quasar-svm engine.
//!
//! The bodies live in `scenarios::suite::initialize_subscription_authority`
//! (engine-neutral, generic over `B: TestSVM`); this is the quasar binding, the
//! analogue of the litesvm `test_initialize_subscription_authority_bound.rs`.
//! `bind_scenarios!` emits one `#[test]` per plain scenario, each calling the
//! generic body with a fresh `make_quasar_backend()`.
//!
//! The sponsor-funded case asserts Alice's lamports are untouched (fee-
//! independent: the sponsor pays) and that the sponsor was charged (it pays the
//! PDA's rent regardless of the fee), so both hold under quasar's `Fee: 0`.
//!
//! The Token-2022 case is an `rstest`-parametrized test, which `bind_scenarios!`
//! cannot emit; its `#[rstest]` shim stays here, dispatching every case into the
//! generic worker. quasar handles the Token-2022 mints these cases construct, so
//! they run unchanged (the same `init_mint`/`set_transfer_hook_config` helpers
//! drive any `B: TestSVM`).

use rstest::rstest;
use spl_token_2022_interface::extension::ExtensionType;

use scenarios::suite::initialize_subscription_authority::initialize_subscription_authority_token_2022_case;
use scenarios::SubscriptionsError;
use subscriptions_quasar_spike::make_quasar_backend;

scenarios::bind_scenarios!(
    make_quasar_backend;
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
    initialize_subscription_authority_token_2022_case(make_quasar_backend(), extensions, expected_error);
}
