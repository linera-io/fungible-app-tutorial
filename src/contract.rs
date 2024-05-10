#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::FungibleToken;
use fungible::{Account, Message, Operation};
use linera_sdk::{
    base::{Amount, Owner, WithContractAbi},
    views::{RootView, View, ViewStorageContext},
    Contract, ContractRuntime,
};

linera_sdk::contract!(FungibleTokenContract);

impl WithContractAbi for FungibleTokenContract {
    type Abi = fungible::FungibleAbi;
}

pub struct FungibleTokenContract {
    state: FungibleToken,
    runtime: ContractRuntime<Self>,
}

impl Contract for FungibleTokenContract {
    type Parameters = ();
    type InstantiationArgument = Amount;
    type Message = Message;

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = FungibleToken::load(ViewStorageContext::from(runtime.key_value_store()))
            .await
            .expect("Failed to load state");
        FungibleTokenContract { state, runtime }
    }

    async fn instantiate(&mut self, amount: Self::InstantiationArgument) {
        // Validate that the application parameters were configured correctly.
        let _ = self.runtime.application_parameters();

        if let Some(owner) = self.runtime.authenticated_signer() {
            self.state.initialize_accounts(owner, amount).await;
        }
    }

    async fn execute_operation(&mut self, operation: Self::Operation) -> Self::Response {
        match operation {
            Operation::Transfer {
                owner,
                amount,
                target_account,
            } => {
                self.check_account_authentication(owner);
                self.state.debit(owner, amount).await;
                self.finish_transfer_to_account(amount, target_account)
                    .await;
            }
        }
    }

    async fn execute_message(&mut self, message: Message) {
        match message {
            Message::Credit { amount, owner } => self.state.credit(owner, amount).await,
        }
    }

    async fn store(mut self) {
        self.state.save().await.expect("Failed to persist state");
    }
}

impl FungibleTokenContract {
    fn check_account_authentication(&mut self, owner: Owner) {
        assert_eq!(
            self.runtime.authenticated_signer(),
            Some(owner),
            "Incorrect authentication"
        );
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
                .send_to(account.chain_id);
        }
    }
}

#[cfg(test)]
#[cfg(target_arch = "wasm32")]
pub mod tests {
    use super::*;
    use futures::FutureExt;
    use linera_sdk::views::{View, ViewStorageContext};
    use std::str::FromStr;

    use webassembly_test::webassembly_test;

    #[webassembly_test]
    pub fn init() {
        let initial_amount = Amount::from_str("50_000").unwrap();
        let fungible = create_and_init(initial_amount);
        assert_eq!(
            fungible.balance(&creator()).now_or_never().unwrap(),
            initial_amount
        )
    }

    fn create_and_init(amount: Amount) -> FungibleToken {
        linera_sdk::test::mock_key_value_store();
        let store = ViewStorageContext::default();
        let mut fungible_token = FungibleToken::load(store).now_or_never().unwrap().unwrap();

        fungible_token
            .initialize_accounts(creator(), amount)
            .now_or_never()
            .unwrap();

        fungible_token
    }

    fn creator() -> Owner {
        "1c02a28d03e846b113de238d8880df3c9c802143b73aea5d173466701bee1786"
            .parse()
            .unwrap()
    }
}
