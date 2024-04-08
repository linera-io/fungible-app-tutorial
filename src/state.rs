use linera_sdk::base::{Amount, Owner};
use linera_sdk::views::{linera_views, MapView, RootView, ViewStorageContext};
use thiserror::Error;

#[derive(RootView)]
#[view(context = "ViewStorageContext")]
pub struct FungibleToken {
    pub accounts: MapView<Owner, Amount>,
}

#[allow(dead_code)]
impl FungibleToken {
    pub async fn initialize_accounts(&mut self, account: Owner, amount: Amount) {
        log::info!("Initialising {owner} with {amount} tokens.");
        self.accounts
            .insert(&owner, amount)
            .expect("Error in insert statement")
    }

    pub async fn balance(&self, account: &Owner) -> Amount {
        self.accounts
            .get(account)
            .await
            .expect("Failure in retrieval")
            .unwrap_or(Amount::ZERO)
    }

    pub async fn credit(&mut self, account: Owner, amount: Amount) {
        log::info!("Owner {account} received {amount} tokens");
        let mut balance = self.balance(&account).await;
        balance.saturating_add_assign(amount);
        self.accounts
            .insert(&account, balance)
            .expect("Failed to insert")
    }

    pub async fn debit(
        &mut self,
        account: Owner,
        amount: Amount,
    ) -> Result<(), InsufficientBalanceError> {
        log::info!("Owner {account} was debited {amount} tokens");
        let mut balance = self.balance(&account).await;
        balance
            .try_sub_assign(amount)
            .map_err(|_| InsufficientBalanceError)?;
        self.accounts
            .insert(&account, balance)
            .expect("Failed to insert");
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Error)]
#[error("Insufficient balance for transfer")]
pub struct InsufficientBalanceError;
