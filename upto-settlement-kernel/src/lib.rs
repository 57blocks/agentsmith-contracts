#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error,
    token::Client as TokenClient, Address, Env, IntoVal,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettlementTerms {
    pub payer: Address,
    pub asset: Address,
    pub pay_to: Address,
    pub facilitator: Address,
    pub max_amount: i128,
    pub valid_after_ledger: u32,
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
    pub fn settle(env: Env, terms: SettlementTerms, actual_amount: i128) {
        validate_settlement(&env, &terms, actual_amount);

        terms
            .payer
            .require_auth_for_args((terms.clone(),).into_val(&env));
        terms
            .facilitator
            .require_auth_for_args((terms.clone(), actual_amount).into_val(&env));

        let kernel = env.current_contract_address();
        let token = TokenClient::new(&env, &terms.asset);
        token.approve(
            &terms.payer,
            &kernel,
            &terms.max_amount,
            &terms.deadline_ledger,
        );
        token.transfer_from(&kernel, &terms.payer, &terms.pay_to, &actual_amount);
    }
}

#[cfg(test)]
mod test;
