#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::FungibleToken;
use crate::state::InsufficientBalanceError;
use async_trait::async_trait;
use fungible::{Account, Message, Operation};
use linera_sdk::base::{Amount, Owner};
use linera_sdk::{base::WithContractAbi, Contract, ContractRuntime, ViewStateStorage};
use thiserror::Error;

linera_sdk::contract!(FungibleTokenContract);

impl WithContractAbi for FungibleTokenContract {
    type Abi = fungible::FungibleAbi;
}

pub struct FungibleTokenContract {
    state: FungibleToken,
    runtime: ContractRuntime<Self>,
}

#[async_trait]
impl Contract for FungibleTokenContract {
    type Error = Error;
    type Storage = ViewStateStorage<Self>;
    type State = FungibleToken;
    type Message = Message;

    async fn new(
        state: FungibleToken,
        runtime: ContractRuntime<Self>,
    ) -> Result<Self, Self::Error> {
        Ok(FungibleTokenContract { state, runtime })
    }

    fn state_mut(&mut self) -> &mut Self::State {
        &mut self.state
    }

    async fn initialize(
        &mut self,
        argument: Self::InitializationArgument,
    ) -> Result<(), Self::Error> {
        // Validate that the application parameters were configured correctly.
        let _ = self.runtime.application_parameters();

        if let Some(owner) = self.runtime.authenticated_signer() {
            self.state_mut().initialize_accounts(owner, argument).await
        }
        Ok(())
    }

    async fn execute_operation(
        &mut self,
        operation: Self::Operation,
    ) -> Result<Self::Response, Self::Error> {
        match operation {
            Operation::Transfer {
                owner,
                amount,
                target_account,
            } => {
                Self::check_account_authentication(self.runtime.authenticated_signer(), owner)?;
                self.state_mut().debit(owner, amount).await?;
                Ok(self
                    .finish_transfer_to_account(amount, target_account)
                    .await)
            }
        }
    }

    async fn execute_message(&mut self, message: Message) -> Result<(), Self::Error> {
        match message {
            Message::Credit { amount, owner } => {
                self.state.credit(owner, amount).await;
                Ok(())
            }
        }
    }
}

#[allow(dead_code)]
impl FungibleTokenContract {
    fn check_account_authentication(
        authenticated_signed: Option<Owner>,
        owner: Owner,
    ) -> Result<(), Error> {
        if authenticated_signed == Some(owner) {
            return Ok(());
        }
        Err(Error::IncorrectAuthentication)
    }

    async fn finish_transfer_to_account(&mut self, amount: Amount, account: Account) {
        if account.chain_id == self.runtime.chain_id() {
            self.state.credit(account.owner, amount).await;
        } else {
            let message = Message::Credit {
                owner: account.owner,
                amount,
            };
            self.runtime
                .prepare_message(message)
                .with_authentication()
                .send_to(account.chain_id)
        }
    }
}

/// An error that can occur during the contract execution.
#[derive(Debug, Error)]
pub enum Error {
    /// Failed to deserialize BCS bytes
    #[error("Failed to deserialize BCS bytes")]
    BcsError(#[from] bcs::Error),

    /// Failed to deserialize JSON string
    #[error("Failed to deserialize JSON string")]
    JsonError(#[from] serde_json::Error),

    #[error("Incorrect Authentication")]
    IncorrectAuthentication, // Add more error variants here.

    #[error("Insufficient Balance")]
    InsufficientBalance(#[from] InsufficientBalanceError),

    #[error("Sessions not supported")]
    SessionsNotSupported,
}
