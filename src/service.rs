#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::FungibleToken;
use async_graphql::{
    EmptySubscription, InputObject, Object, Request, Response, Schema, SimpleObject,
};
use fungible::Operation;
use linera_sdk::base::{AccountOwner, Amount, ChainId};
use linera_sdk::{
    base::WithServiceAbi,
    views::{MapView, View, ViewStorageContext},
    Service, ServiceRuntime, ViewStateStorage,
};
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Clone)]
pub struct FungibleTokenService {
    state: Arc<FungibleToken>,
    #[allow(unused)]
    runtime: Arc<Mutex<ServiceRuntime<Self>>>,
}

linera_sdk::service!(FungibleTokenService);

impl WithServiceAbi for FungibleTokenService {
    type Abi = fungible::FungibleAbi;
}

impl Service for FungibleTokenService {
    type Error = ServiceError;
    type Storage = ViewStateStorage<Self>;
    type State = FungibleToken;

    async fn new(state: Self::State, runtime: ServiceRuntime<Self>) -> Result<Self, Self::Error> {
        Ok(FungibleTokenService {
            state: Arc::new(state),
            runtime: Arc::new(Mutex::new(runtime)),
        })
    }

    async fn handle_query(&self, request: Request) -> Result<Response, Self::Error> {
        let schema = Schema::build(self.clone(), MutationRoot, EmptySubscription).finish();
        let response = schema.execute(request).await;
        Ok(response)
    }
}

#[Object]
impl FungibleTokenService {
    async fn accounts(&self) -> MapView<AccountOwner, Amount> {
        let mut accounts = MapView::load(ViewStorageContext::default())
            .await
            .expect("Failed to create an empty `MapView`");

        self.state
            .accounts
            .for_each_index_value(|owner, amount| {
                accounts
                    .insert(&owner.into(), amount)
                    .expect("Failed to insert account into temporary map");
                Ok(())
            })
            .await
            .expect("Failed to get map of accounts");

        accounts
    }

    async fn ticker_symbol(&self) -> Result<String, async_graphql::Error> {
        Ok("MYTKN".to_owned())
    }
}

#[derive(Clone, Copy, Debug, InputObject, SimpleObject)]
pub struct Account {
    chain_id: ChainId,
    owner: AccountOwner,
}

struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn transfer(
        &self,
        owner: AccountOwner,
        amount: Amount,
        target_account: Account,
    ) -> Vec<u8> {
        let AccountOwner::User(owner) = owner else {
            panic!("Application accounts aren't supported");
        };

        let Account {
            chain_id: target_chain_id,
            owner: AccountOwner::User(target_owner),
        } = target_account
        else {
            panic!("Application accounts aren't supported");
        };

        let target_account = fungible::Account {
            chain_id: target_chain_id,
            owner: target_owner,
        };

        bcs::to_bytes(&Operation::Transfer {
            owner,
            amount,
            target_account,
        })
        .expect("Invalid operation")
    }
}

/// An error that can occur while querying the service.
#[derive(Debug, Error)]
pub enum ServiceError {
    /// Invalid query argument; could not deserialize request.
    #[error("Invalid query argument; could not deserialize request")]
    InvalidQuery(#[from] serde_json::Error),
    // Add error variants here.
}
