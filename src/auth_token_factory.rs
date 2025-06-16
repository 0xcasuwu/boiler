//! Auth Token Factory
//!
//! A factory contract that deploys auth tokens at the expected location.
//! This factory is deployed at block 6, tx 0xffee (AUTH_TOKEN_FACTORY_ID).

use alkanes_runtime::runtime::AlkaneResponder;
use alkanes_runtime::{declare_alkane, message::MessageDispatch};
#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_support::{context::Context, parcel::AlkaneTransfer, response::CallResponse};
use anyhow::{anyhow, Result};

/// Auth Token Factory - deploys auth tokens on demand
#[derive(Default)]
pub struct AuthTokenFactory(());

#[derive(MessageDispatch)]
enum AuthTokenFactoryMessage {
    /// Deploy a new auth token with the specified amount
    #[opcode(0)]
    DeployAuthToken { amount: u128 },
}

impl AuthTokenFactory {
    /// Deploy a new auth token with the specified amount
    fn deploy_auth_token(&self, amount: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Deploy auth token using precompiled bytecode
        let auth_token_id = self.deploy_contract(
            include_bytes!("precompiled/alkanes_std_auth_token_build.rs"),
            &[0u128, amount], // opcode 0 (Initialize) with amount parameter
        )?;

        // Return the deployed auth token as a transfer
        response.alkanes.0.push(AlkaneTransfer {
            id: auth_token_id,
            value: amount,
        });

        Ok(response)
    }
}

impl AlkaneResponder for AuthTokenFactory {}

declare_alkane! {
    impl AlkaneResponder for AuthTokenFactory {
        type Message = AuthTokenFactoryMessage;
    }
}
