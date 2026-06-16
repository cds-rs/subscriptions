//! Cantina HIGH finding 3.1.3, reproduced in the DSL.
//!
//! "Attacker can permanently block PDA creation by over-funding the address."
//! On this `dsl-audit` branch the PR5 fix is reverted, so account creation is an
//! unconditional `CreateAccount`. An attacker pre-funds the authority PDA with
//! lamports; the System program then rejects the create, and the legitimate user
//! can never initialize their authority. On the fixed code the program tops up
//! the rent and `Allocate`/`Assign`s in place, so creation succeeds and this test
//! (which asserts creation is blocked) fails: that failure is the fix's proof.

use solana_account::Account;
use solana_pubkey::Pubkey;
use solana_signer::Signer;

use crate::tests::{pda::get_subscription_authority_pda, utils::World};

#[test]
fn finding_3_1_3_prefunded_pda_blocks_creation() {
    let mut world = World::new(
        "AUDIT 3.1.3: a pre-funded PDA blocks authority creation",
        "an attacker pre-funds the authority PDA so Alice can never initialize it (Cantina HIGH 3.1.3)",
    );
    let alice = world.actor("alice");
    let mallory = world.actor("mallory");
    let mint = world.usdc_mint(&alice);
    world.prop(mint, "USDC mint");
    world.fund_ata(mint, &alice, 1_000_000);

    let (pda, _) = get_subscription_authority_pda(&alice.pubkey(), &mint);
    world.prop(pda, "Alice's SubAuthority (griefed)");

    // The attack: Mallory pre-funds the PDA address with lamports.
    world.md().step("Mallory pre-funds Alice's authority PDA address");
    let _ = &mallory; // the griefer is off-chain here; the pre-funding is the attack
    world.svm_mut()
        .set_account(
            pda,
            Account { lamports: 1_000_000, data: vec![], owner: Pubkey::default(), executable: false, rent_epoch: 0 },
        )
        .unwrap();

    world.md().step("Alice tries to initialize her authority");
    let (res, _, _) = world.init_authority(&alice, mint, None);
    let blocked = !res.is_success();

    world.md().note(
        "Finding 3.1.3: the PDA address was pre-funded, so the unconditional CreateAccount on this branch \
         is rejected by the System program and Alice's authority cannot be created — a permanent denial of \
         service against any user whose (deterministic) PDA an attacker front-runs. On the fixed code (PR5) \
         the program tops up the rent and Allocate/Assign-s in place, creation succeeds, and the check below \
         fails: that failure is the regression proof the fix holds.",
    );
    world.md().check("the pre-funded PDA BLOCKS creation (the vulnerability)", true, blocked);
}

