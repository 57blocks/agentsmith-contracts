extern crate std;

use super::E2eSimpleAccount;
use ed25519_dalek::{Signer, SigningKey};
use soroban_sdk::{vec, BytesN, Env, IntoVal};

#[test]
fn owner_signature_authorizes_the_contract_account() {
    let env = Env::default();
    let signing_key = SigningKey::from_bytes(&[7; 32]);
    let owner = BytesN::from_array(&env, signing_key.verifying_key().as_bytes());
    let account = env.register(E2eSimpleAccount, (owner,));
    let payload = BytesN::from_array(&env, &[9; 32]);
    let signature = BytesN::from_array(&env, &signing_key.sign(&[9; 32]).to_bytes());

    assert_eq!(
        env.try_invoke_contract_check_auth::<soroban_sdk::InvokeError>(
            &account,
            &payload,
            signature.into_val(&env),
            &vec![&env],
        ),
        Ok(()),
    );
}
