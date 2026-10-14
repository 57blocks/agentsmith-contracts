#![no_std]

use soroban_sdk::{
    auth::{Context, CustomAccountInterface},
    contract, contractimpl, contracttype,
    crypto::Hash,
    Bytes, BytesN, Env, Error, Vec,
};

#[contracttype]
enum DataKey {
    Owner,
}

/// Minimal account contract used only by the live integration test.
///
/// It intentionally has no recovery, upgrade, policy, or key-rotation support
/// and must not hold production funds.
#[contract]
pub struct E2eSimpleAccount;

#[contractimpl]
impl E2eSimpleAccount {
    pub fn __constructor(env: Env, owner: BytesN<32>) {
        env.storage().instance().set(&DataKey::Owner, &owner);
    }
}

#[contractimpl]
impl CustomAccountInterface for E2eSimpleAccount {
    type Signature = BytesN<64>;
    type Error = Error;

    fn __check_auth(
        env: Env,
        signature_payload: Hash<32>,
        signature: BytesN<64>,
        _auth_contexts: Vec<Context>,
    ) -> Result<(), Self::Error> {
        let owner = env
            .storage()
            .instance()
            .get::<_, BytesN<32>>(&DataKey::Owner)
            .unwrap();
        let message = Bytes::from_array(&env, &signature_payload.to_bytes().to_array());
        env.crypto().ed25519_verify(&owner, &message, &signature);
        Ok(())
    }
}

#[cfg(test)]
mod test;
