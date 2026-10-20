#![no_std]

//! Upto settlement kernel.
//!
//! Settles an "up to" payment: the payer authorizes a maximum amount once,
//! and the facilitator later settles any actual amount within that maximum.
//! Settlement is atomic: the kernel approves itself on the payer's behalf and
//! pulls the actual amount to the payee in the same invocation.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error,
    token::Client as TokenClient, Address, Env, IntoVal,
};

/// Terms the payer signs. The actual settled amount is not part of the terms;
/// it is supplied (and signed) by the facilitator at settlement time.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettlementTerms {
    /// Account whose funds are debited.
    pub payer: Address,
    /// Token contract (SEP-41) used for the payment.
    pub asset: Address,
    /// Account that receives the settled amount.
    pub pay_to: Address,
    /// Account allowed to choose the actual amount and submit settlement.
    pub facilitator: Address,
    /// Upper bound on the settled amount, in the token's atomic units.
    pub max_amount: i128,
    /// First ledger (inclusive) at which settlement is allowed.
    pub valid_after_ledger: u32,
    /// Last ledger (inclusive) at which settlement is allowed; also used as
    /// the expiration ledger of the inline token approval.
    pub deadline_ledger: u32,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    InvalidMaximum = 1,
    InvalidActualAmount = 2,
    ExceedsMaximum = 3,
    Expired = 4,
    NotYetValid = 5,
    InvalidValidityWindow = 6,
}

#[contract]
pub struct UptoSettlementKernel;

/// Rejects terms or amounts that must never settle, before any authorization
/// or token call is made.
fn validate_settlement(env: &Env, terms: &SettlementTerms, actual_amount: i128) {
    if terms.max_amount <= 0 {
        panic_with_error!(env, Error::InvalidMaximum);
    }
    if actual_amount <= 0 {
        panic_with_error!(env, Error::InvalidActualAmount);
    }
    if actual_amount > terms.max_amount {
        panic_with_error!(env, Error::ExceedsMaximum);
    }
    if terms.valid_after_ledger > terms.deadline_ledger {
        panic_with_error!(env, Error::InvalidValidityWindow);
    }
    if env.ledger().sequence() < terms.valid_after_ledger {
        panic_with_error!(env, Error::NotYetValid);
    }
    if env.ledger().sequence() > terms.deadline_ledger {
        panic_with_error!(env, Error::Expired);
    }
}

#[contractimpl]
impl UptoSettlementKernel {
    /// Settles `actual_amount` (at most `terms.max_amount`) from the payer to
    /// `terms.pay_to` in a single atomic invocation.
    ///
    /// Authorization:
    /// - The payer signs `settle(terms)` only, so one signature covers any
    ///   amount up to `max_amount`. The payer's authorization tree must also
    ///   include the nested `approve` call on the token made below.
    /// - The facilitator signs `settle(terms, actual_amount)`, binding the
    ///   exact amount being settled.
    ///
    /// Any failure (validation, auth, or token transfer) reverts the whole
    /// invocation, including the inline approval.
    pub fn settle(env: Env, terms: SettlementTerms, actual_amount: i128) {
        validate_settlement(&env, &terms, actual_amount);

        // The payer authorizes the terms but not the actual amount, which is
        // decided later by the facilitator. Hence explicit args here.
        terms
            .payer
            .require_auth_for_args((terms.clone(),).into_val(&env));
        // The facilitator authorizes the full invocation args
        // `(terms, actual_amount)`; plain `require_auth` binds exactly those.
        terms.facilitator.require_auth();

        let kernel = env.current_contract_address();
        let token = TokenClient::new(&env, &terms.asset);
        // Grant the kernel an allowance of `max_amount` until the deadline.
        // This overwrites any existing allowance, so a pre-existing approval
        // cannot be used to settle without this nested payer authorization.
        token.approve(
            &terms.payer,
            &kernel,
            &terms.max_amount,
            &terms.deadline_ledger,
        );
        // Pull only the actual amount; the facilitator's signature binds it.
        token.transfer_from(&kernel, &terms.payer, &terms.pay_to, &actual_amount);
    }
}

#[cfg(test)]
mod test;
