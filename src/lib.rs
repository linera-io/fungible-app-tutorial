use async_graphql::{InputObject, Request, Response};
use linera_sdk::{
    base::{Amount, ChainId, ContractAbi, Owner, ServiceAbi},
    graphql::GraphQLMutationRoot,
};
use serde::{Deserialize, Serialize};

pub struct FungibleAbi;

impl ContractAbi for FungibleAbi {
    type Operation = Operation;
    type Response = ();
}

impl ServiceAbi for FungibleAbi {
    type Query = Request;
    type QueryResponse = Response;
}

#[derive(Debug, Deserialize, Serialize, GraphQLMutationRoot)]
pub enum Operation {
    Transfer {
        owner: Owner,
        amount: Amount,
        target_account: Account,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Message {
    Credit { amount: Amount, owner: Owner },
}

#[derive(
    Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize, InputObject,
)]
pub struct Account {
    pub chain_id: ChainId,
    pub owner: Owner,
}
