extern crate std;

use super::{Error, SettlementTerms, UptoSettlementKernel, UptoSettlementKernelClient};
use soroban_sdk::{
    symbol_short,
    testutils::{
        Address as _, AuthorizedFunction, AuthorizedInvocation, Ledger as _, MockAuth,
        MockAuthInvoke,
    },
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, Error as HostError, IntoVal, Symbol,
};

#[test]
fn settles_actual_amount_when_allowance_already_exists() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let payer = Address::generate(&env);
    let pay_to = Address::generate(&env);
    let facilitator = Address::generate(&env);
    let asset = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let token = TokenClient::new(&env, &asset);
    let token_admin = StellarAssetClient::new(&env, &asset);
    token_admin.mint(&payer, &200_000);

    let kernel_address = env.register(UptoSettlementKernel, ());
    let kernel = UptoSettlementKernelClient::new(&env, &kernel_address);
    token.approve(&payer, &kernel_address, &100_000, &1_000);

    kernel.settle(
        &SettlementTerms {
            payer: payer.clone(),
            asset,
            pay_to: pay_to.clone(),
            facilitator,
            max_amount: 100_000,
            valid_after_ledger: 0,
            deadline_ledger: 1_000,
        },
        &40_000,
    );

    assert_eq!(token.balance(&payer), 160_000);
    assert_eq!(token.balance(&pay_to), 40_000);
    assert_eq!(token.allowance(&payer, &kernel_address), 60_000);
}

#[test]
fn rejects_an_actual_amount_above_the_buyer_maximum() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let payer = Address::generate(&env);
    let pay_to = Address::generate(&env);
    let facilitator = Address::generate(&env);
    let asset = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let token = TokenClient::new(&env, &asset);
    let token_admin = StellarAssetClient::new(&env, &asset);
    token_admin.mint(&payer, &200_000);

    let kernel_address = env.register(UptoSettlementKernel, ());
    let kernel = UptoSettlementKernelClient::new(&env, &kernel_address);
    token.approve(&payer, &kernel_address, &200_000, &1_000);

    let result = kernel.try_settle(
        &SettlementTerms {
            payer: payer.clone(),
            asset,
            pay_to: pay_to.clone(),
            facilitator,
            max_amount: 100_000,
            valid_after_ledger: 0,
            deadline_ledger: 1_000,
        },
        &100_001,
    );

    assert_eq!(
        result,
        Err(Ok(HostError::from_contract_error(
            Error::ExceedsMaximum as u32,
        ))),
    );
    assert_eq!(token.balance(&payer), 200_000);
    assert_eq!(token.balance(&pay_to), 0);
}

#[test]
fn rejects_a_non_positive_buyer_maximum() {
    let env = Env::default();
    env.mock_all_auths();

    let payer = Address::generate(&env);
    let pay_to = Address::generate(&env);
    let facilitator = Address::generate(&env);
    let asset = Address::generate(&env);
    let kernel_address = env.register(UptoSettlementKernel, ());
    let kernel = UptoSettlementKernelClient::new(&env, &kernel_address);

    let result = kernel.try_settle(
        &SettlementTerms {
            payer,
            asset,
            pay_to,
            facilitator,
            max_amount: 0,
            valid_after_ledger: 0,
            deadline_ledger: 1_000,
        },
        &1,
    );

    assert_eq!(
        result,
        Err(Ok(HostError::from_contract_error(
            Error::InvalidMaximum as u32,
        ))),
    );
}

#[test]
fn rejects_a_non_positive_actual_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let payer = Address::generate(&env);
    let pay_to = Address::generate(&env);
    let facilitator = Address::generate(&env);
    let asset = Address::generate(&env);
    let kernel_address = env.register(UptoSettlementKernel, ());
    let kernel = UptoSettlementKernelClient::new(&env, &kernel_address);

    let result = kernel.try_settle(
        &SettlementTerms {
            payer,
            asset,
            pay_to,
            facilitator,
            max_amount: 100_000,
            valid_after_ledger: 0,
            deadline_ledger: 1_000,
        },
        &0,
    );

    assert_eq!(
        result,
        Err(Ok(HostError::from_contract_error(
            Error::InvalidActualAmount as u32,
        ))),
    );
}

#[test]
fn rejects_settlement_after_the_deadline() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_sequence_number(99);

    let payer = Address::generate(&env);
    let pay_to = Address::generate(&env);
    let facilitator = Address::generate(&env);
    let asset = Address::generate(&env);
    let kernel_address = env.register(UptoSettlementKernel, ());
    let kernel = UptoSettlementKernelClient::new(&env, &kernel_address);

    let result = kernel.try_settle(
        &SettlementTerms {
            payer,
            asset,
            pay_to,
            facilitator,
            max_amount: 100_000,
            valid_after_ledger: 0,
            deadline_ledger: 98,
        },
        &40_000,
    );

    assert_eq!(
        result,
        Err(Ok(HostError::from_contract_error(Error::Expired as u32))),
    );
}

#[test]
fn rejects_settlement_before_the_valid_after_ledger() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_sequence_number(99);

    let payer = Address::generate(&env);
    let pay_to = Address::generate(&env);
    let facilitator = Address::generate(&env);
    let asset = Address::generate(&env);
    let kernel_address = env.register(UptoSettlementKernel, ());
    let kernel = UptoSettlementKernelClient::new(&env, &kernel_address);

    let result = kernel.try_settle(
        &SettlementTerms {
            payer,
            asset,
            pay_to,
            facilitator,
            max_amount: 100_000,
            valid_after_ledger: 100,
            deadline_ledger: 200,
        },
        &40_000,
    );

    assert_eq!(
        result,
        Err(Ok(HostError::from_contract_error(
            Error::NotYetValid as u32,
        ))),
    );
}

#[test]
fn rejects_an_inverted_validity_window() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_sequence_number(100);

    let payer = Address::generate(&env);
    let pay_to = Address::generate(&env);
    let facilitator = Address::generate(&env);
    let asset = Address::generate(&env);
    let kernel_address = env.register(UptoSettlementKernel, ());
    let kernel = UptoSettlementKernelClient::new(&env, &kernel_address);

    let result = kernel.try_settle(
        &SettlementTerms {
            payer,
            asset,
            pay_to,
            facilitator,
            max_amount: 100_000,
            valid_after_ledger: 200,
            deadline_ledger: 100,
        },
        &40_000,
    );

    assert_eq!(
        result,
        Err(Ok(HostError::from_contract_error(
            Error::InvalidValidityWindow as u32,
        ))),
    );
}

#[test]
fn atomically_approves_and_settles_when_no_allowance_exists() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let payer = Address::generate(&env);
    let pay_to = Address::generate(&env);
    let facilitator = Address::generate(&env);
    let asset = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let token = TokenClient::new(&env, &asset);
    StellarAssetClient::new(&env, &asset).mint(&payer, &200_000);

    let kernel_address = env.register(UptoSettlementKernel, ());
    let kernel = UptoSettlementKernelClient::new(&env, &kernel_address);
    let terms = SettlementTerms {
        payer: payer.clone(),
        asset,
        pay_to: pay_to.clone(),
        facilitator,
        max_amount: 100_000,
        valid_after_ledger: 0,
        deadline_ledger: 1_000,
    };

    kernel.settle(&terms, &40_000);

    assert_eq!(token.balance(&payer), 160_000);
    assert_eq!(token.balance(&pay_to), 40_000);
    assert_eq!(token.allowance(&payer, &kernel_address), 60_000);
}

#[test]
fn settlement_binds_buyer_maximum_and_facilitator_actual_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let payer = Address::generate(&env);
    let pay_to = Address::generate(&env);
    let facilitator = Address::generate(&env);
    let asset = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    StellarAssetClient::new(&env, &asset).mint(&payer, &200_000);

    let kernel_address = env.register(UptoSettlementKernel, ());
    let kernel = UptoSettlementKernelClient::new(&env, &kernel_address);
    let terms = SettlementTerms {
        payer: payer.clone(),
        asset: asset.clone(),
        pay_to,
        facilitator: facilitator.clone(),
        max_amount: 100_000,
        valid_after_ledger: 0,
        deadline_ledger: 1_000,
    };

    kernel.settle(&terms, &40_000);

    assert_eq!(
        env.auths(),
        std::vec![
            (
                payer.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        kernel_address.clone(),
                        Symbol::new(&env, "settle"),
                        (terms.clone(),).into_val(&env),
                    )),
                    sub_invocations: std::vec![AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            asset,
                            symbol_short!("approve"),
                            (payer, kernel_address.clone(), 100_000_i128, 1_000_u32,)
                                .into_val(&env),
                        )),
                        sub_invocations: std::vec![],
                    }],
                },
            ),
            (
                facilitator,
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        kernel_address,
                        Symbol::new(&env, "settle"),
                        (terms, 40_000_i128).into_val(&env),
                    )),
                    sub_invocations: std::vec![],
                },
            ),
        ],
    );
}

#[test]
fn failed_transfer_rolls_back_the_inline_approval() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let payer = Address::generate(&env);
    let pay_to = Address::generate(&env);
    let facilitator = Address::generate(&env);
    let asset = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let token = TokenClient::new(&env, &asset);
    StellarAssetClient::new(&env, &asset).mint(&payer, &10_000);

    let kernel_address = env.register(UptoSettlementKernel, ());
    let kernel = UptoSettlementKernelClient::new(&env, &kernel_address);
    let result = kernel.try_settle(
        &SettlementTerms {
            payer: payer.clone(),
            asset,
            pay_to: pay_to.clone(),
            facilitator,
            max_amount: 100_000,
            valid_after_ledger: 0,
            deadline_ledger: 1_000,
        },
        &40_000,
    );

    assert!(result.is_err());
    assert_eq!(token.balance(&payer), 10_000);
    assert_eq!(token.balance(&pay_to), 0);
    assert_eq!(token.allowance(&payer, &kernel_address), 0);
}

/* Authorization tests.
 *
 * The tests above run under `mock_all_auths`, which approves every
 * `require_auth` call and therefore cannot show that a wrong or missing
 * authorization is rejected. The tests below install specific authorization
 * entries with `mock_auths`, putting the host into enforcing mode: anything the
 * Kernel asks for that is not in the installed set fails. */

struct Fixture {
    env: Env,
    payer: Address,
    pay_to: Address,
    facilitator: Address,
    asset: Address,
    kernel: Address,
}

/// A funded buyer, a registered SEP-41 asset and a fresh Kernel, with every
/// authorization mocked so the setup itself does not need entries. Tests that
/// exercise the authorization model call `mock_auths` afterwards, which
/// replaces the mock with an enforcing entry set.
fn fixture() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let payer = Address::generate(&env);
    let pay_to = Address::generate(&env);
    let facilitator = Address::generate(&env);
    let asset = env.register_stellar_asset_contract_v2(admin).address();
    StellarAssetClient::new(&env, &asset).mint(&payer, &200_000);
    let kernel = env.register(UptoSettlementKernel, ());

    Fixture {
        env,
        payer,
        pay_to,
        facilitator,
        asset,
        kernel,
    }
}

impl Fixture {
    fn terms(&self) -> SettlementTerms {
        SettlementTerms {
            payer: self.payer.clone(),
            asset: self.asset.clone(),
            pay_to: self.pay_to.clone(),
            facilitator: self.facilitator.clone(),
            max_amount: 100_000,
            valid_after_ledger: 0,
            deadline_ledger: 1_000,
        }
    }

    fn token(&self) -> TokenClient<'_> {
        TokenClient::new(&self.env, &self.asset)
    }

    fn client(&self) -> UptoSettlementKernelClient<'_> {
        UptoSettlementKernelClient::new(&self.env, &self.kernel)
    }

    fn approval(&self, terms: &SettlementTerms) -> MockAuthInvoke<'_> {
        MockAuthInvoke {
            contract: &self.asset,
            fn_name: "approve",
            args: (
                self.payer.clone(),
                self.kernel.clone(),
                terms.max_amount,
                terms.deadline_ledger,
            )
                .into_val(&self.env),
            sub_invokes: &[],
        }
    }

    /// No value moved, and the Kernel holds nothing of its own.
    fn assert_no_settlement(&self) {
        assert_eq!(self.token().balance(&self.payer), 200_000);
        assert_eq!(self.token().balance(&self.pay_to), 0);
        assert_eq!(self.token().balance(&self.kernel), 0);
    }
}

#[test]
fn settle_rejects_a_missing_facilitator_authorization() {
    let f = fixture();
    let terms = f.terms();
    let approval = f.approval(&terms);

    f.env.mock_auths(&[MockAuth {
        address: &f.payer,
        invoke: &MockAuthInvoke {
            contract: &f.kernel,
            fn_name: "settle",
            args: (terms.clone(),).into_val(&f.env),
            sub_invokes: core::slice::from_ref(&approval),
        },
    }]);

    assert!(f.client().try_settle(&terms, &40_000).is_err());
    f.assert_no_settlement();
}

#[test]
fn settle_rejects_a_facilitator_authorization_for_a_different_amount() {
    let f = fixture();
    let terms = f.terms();
    let approval = f.approval(&terms);

    f.env.mock_auths(&[
        MockAuth {
            address: &f.payer,
            invoke: &MockAuthInvoke {
                contract: &f.kernel,
                fn_name: "settle",
                args: (terms.clone(),).into_val(&f.env),
                sub_invokes: core::slice::from_ref(&approval),
            },
        },
        MockAuth {
            address: &f.facilitator,
            invoke: &MockAuthInvoke {
                contract: &f.kernel,
                fn_name: "settle",
                args: (terms.clone(), 10_000_i128).into_val(&f.env),
                sub_invokes: &[],
            },
        },
    ]);

    assert!(f.client().try_settle(&terms, &40_000).is_err());
    f.assert_no_settlement();
}

#[test]
fn settle_rejects_a_redirected_payee() {
    let f = fixture();
    let signed = f.terms();
    let thief = Address::generate(&f.env);
    let redirected = SettlementTerms {
        pay_to: thief.clone(),
        ..f.terms()
    };
    let approval = f.approval(&signed);

    f.env.mock_auths(&[
        MockAuth {
            address: &f.payer,
            invoke: &MockAuthInvoke {
                contract: &f.kernel,
                fn_name: "settle",
                args: (signed,).into_val(&f.env),
                sub_invokes: core::slice::from_ref(&approval),
            },
        },
        MockAuth {
            address: &f.facilitator,
            invoke: &MockAuthInvoke {
                contract: &f.kernel,
                fn_name: "settle",
                args: (redirected.clone(), 40_000_i128).into_val(&f.env),
                sub_invokes: &[],
            },
        },
    ]);

    assert!(f.client().try_settle(&redirected, &40_000).is_err());
    assert_eq!(f.token().balance(&thief), 0);
    f.assert_no_settlement();
}

#[test]
fn settle_rejects_a_maximum_raised_above_the_signed_terms() {
    let f = fixture();
    let signed = SettlementTerms {
        max_amount: 50_000,
        ..f.terms()
    };
    let raised = f.terms();
    let approval = f.approval(&signed);

    f.env.mock_auths(&[
        MockAuth {
            address: &f.payer,
            invoke: &MockAuthInvoke {
                contract: &f.kernel,
                fn_name: "settle",
                args: (signed,).into_val(&f.env),
                sub_invokes: core::slice::from_ref(&approval),
            },
        },
        MockAuth {
            address: &f.facilitator,
            invoke: &MockAuthInvoke {
                contract: &f.kernel,
                fn_name: "settle",
                args: (raised.clone(), 80_000_i128).into_val(&f.env),
                sub_invokes: &[],
            },
        },
    ]);

    assert!(f.client().try_settle(&raised, &80_000).is_err());
    f.assert_no_settlement();
}

#[test]
fn settlement_rejects_authorizations_for_the_old_method_name() {
    let f = fixture();
    let terms = f.terms();
    let approval = f.approval(&terms);

    f.env.mock_auths(&[
        MockAuth {
            address: &f.payer,
            invoke: &MockAuthInvoke {
                contract: &f.kernel,
                fn_name: "settle_with_approval",
                args: (terms.clone(),).into_val(&f.env),
                sub_invokes: core::slice::from_ref(&approval),
            },
        },
        MockAuth {
            address: &f.facilitator,
            invoke: &MockAuthInvoke {
                contract: &f.kernel,
                fn_name: "settle_with_approval",
                args: (terms.clone(), 40_000_i128).into_val(&f.env),
                sub_invokes: &[],
            },
        },
    ]);

    assert!(f.client().try_settle(&terms, &40_000).is_err());
    f.assert_no_settlement();
    assert_eq!(f.token().allowance(&f.payer, &f.kernel), 0);
}

#[test]
fn the_old_entry_point_is_not_exported() {
    let f = fixture();
    let result = f.env.try_invoke_contract::<(), HostError>(
        &f.kernel,
        &Symbol::new(&f.env, "settle_with_approval"),
        (f.terms(), 40_000_i128).into_val(&f.env),
    );

    assert!(result.is_err());
    f.assert_no_settlement();
}

#[test]
fn settlement_rejects_a_buyer_authorization_without_the_nested_approval() {
    let f = fixture();
    let terms = f.terms();

    f.env.mock_auths(&[
        MockAuth {
            address: &f.payer,
            invoke: &MockAuthInvoke {
                contract: &f.kernel,
                fn_name: "settle",
                args: (terms.clone(),).into_val(&f.env),
                sub_invokes: &[],
            },
        },
        MockAuth {
            address: &f.facilitator,
            invoke: &MockAuthInvoke {
                contract: &f.kernel,
                fn_name: "settle",
                args: (terms.clone(), 40_000_i128).into_val(&f.env),
                sub_invokes: &[],
            },
        },
    ]);

    assert!(f.client().try_settle(&terms, &40_000).is_err());
    f.assert_no_settlement();
    assert_eq!(f.token().allowance(&f.payer, &f.kernel), 0);
}

#[test]
fn existing_allowance_does_not_bypass_the_nested_approval_authorization() {
    let f = fixture();
    let terms = f.terms();
    f.token().approve(&f.payer, &f.kernel, &100_000, &1_000);

    f.env.mock_auths(&[
        MockAuth {
            address: &f.payer,
            invoke: &MockAuthInvoke {
                contract: &f.kernel,
                fn_name: "settle",
                args: (terms.clone(),).into_val(&f.env),
                sub_invokes: &[],
            },
        },
        MockAuth {
            address: &f.facilitator,
            invoke: &MockAuthInvoke {
                contract: &f.kernel,
                fn_name: "settle",
                args: (terms.clone(), 40_000_i128).into_val(&f.env),
                sub_invokes: &[],
            },
        },
    ]);

    assert!(f.client().try_settle(&terms, &40_000).is_err());
    f.assert_no_settlement();
    assert_eq!(f.token().allowance(&f.payer, &f.kernel), 100_000);
}

#[test]
fn settlement_accepts_exactly_the_expected_authorization_tree() {
    let f = fixture();
    let terms = f.terms();

    f.env.mock_auths(&[
        MockAuth {
            address: &f.payer,
            invoke: &MockAuthInvoke {
                contract: &f.kernel,
                fn_name: "settle",
                args: (terms.clone(),).into_val(&f.env),
                sub_invokes: &[MockAuthInvoke {
                    contract: &f.asset,
                    fn_name: "approve",
                    args: (f.payer.clone(), f.kernel.clone(), 100_000_i128, 1_000_u32)
                        .into_val(&f.env),
                    sub_invokes: &[],
                }],
            },
        },
        MockAuth {
            address: &f.facilitator,
            invoke: &MockAuthInvoke {
                contract: &f.kernel,
                fn_name: "settle",
                args: (terms.clone(), 40_000_i128).into_val(&f.env),
                sub_invokes: &[],
            },
        },
    ]);

    f.client().settle(&terms, &40_000);

    assert_eq!(f.token().balance(&f.payer), 160_000);
    assert_eq!(f.token().balance(&f.pay_to), 40_000);
    assert_eq!(f.token().balance(&f.kernel), 0);
}

#[test]
fn a_settled_authorization_cannot_be_replayed() {
    let f = fixture();
    let terms = f.terms();
    let approval = f.approval(&terms);

    f.env.mock_auths(&[
        MockAuth {
            address: &f.payer,
            invoke: &MockAuthInvoke {
                contract: &f.kernel,
                fn_name: "settle",
                args: (terms.clone(),).into_val(&f.env),
                sub_invokes: core::slice::from_ref(&approval),
            },
        },
        MockAuth {
            address: &f.facilitator,
            invoke: &MockAuthInvoke {
                contract: &f.kernel,
                fn_name: "settle",
                args: (terms.clone(), 40_000_i128).into_val(&f.env),
                sub_invokes: &[],
            },
        },
    ]);

    f.client().settle(&terms, &40_000);
    assert_eq!(f.token().balance(&f.pay_to), 40_000);

    // The same entries are still installed. The host nonce has been consumed,
    // so a second settlement on the same authorization must fail.
    assert!(f.client().try_settle(&terms, &40_000).is_err());
    assert_eq!(f.token().balance(&f.pay_to), 40_000);
    assert_eq!(f.token().balance(&f.payer), 160_000);
}

/* Characterization test, not an endorsement. SEP-41 `approve` sets rather than
 * adds, so the approval path destroys whatever allowance the buyer already
 * granted the Kernel and shortens its expiry to the settlement deadline. This
 * pins the behaviour so a change to it is a deliberate one. */
#[test]
fn settlement_overwrites_a_larger_pre_existing_allowance() {
    let f = fixture();
    f.token().approve(&f.payer, &f.kernel, &500_000, &5_000);

    f.client().settle(&f.terms(), &40_000);

    assert_eq!(f.token().allowance(&f.payer, &f.kernel), 60_000);

    f.env.ledger().set_sequence_number(1_001);
    assert_eq!(f.token().allowance(&f.payer, &f.kernel), 0);
}
