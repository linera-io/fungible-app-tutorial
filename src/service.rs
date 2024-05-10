#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::FungibleToken;
use async_graphql::{EmptySubscription, Object, Request, Response, Schema};
use fungible::Operation;
use linera_sdk::{
    base::{Amount, Owner, WithServiceAbi},
    graphql::GraphQLMutationRoot,
    views::MapView,
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
    type Parameters = ();

    async fn new(state: Self::State, runtime: ServiceRuntime<Self>) -> Result<Self, Self::Error> {
        Ok(FungibleTokenService {
            state: Arc::new(state),
            runtime: Arc::new(Mutex::new(runtime)),
        })
    }

    async fn handle_query(&self, request: Request) -> Result<Response, Self::Error> {
        let schema =
            Schema::build(self.clone(), Operation::mutation_root(), EmptySubscription).finish();
        let response = schema.execute(request).await;
        Ok(response)
    }
}

#[Object]
impl FungibleTokenService {
    async fn accounts(&self) -> &MapView<Owner, Amount> {
        &self.state.accounts
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
