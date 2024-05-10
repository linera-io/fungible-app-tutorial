use linera_sdk::base::{Amount, Owner};
use linera_sdk::views::{linera_views, MapView, RootView, ViewStorageContext};

#[derive(RootView)]
#[view(context = "ViewStorageContext")]
pub struct FungibleToken {
    pub accounts: MapView<Owner, Amount>,
}

#[allow(dead_code)]
impl FungibleToken {
    pub async fn initialize_accounts(&mut self, account: Owner, amount: Amount) {
        log::info!("Initialising {account} with {amount} tokens.");
        self.accounts
            .insert(&account, amount)
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

    pub async fn debit(&mut self, account: Owner, amount: Amount) {
        log::info!("Owner {account} was debited {amount} tokens");
        let mut balance = self.balance(&account).await;
        balance
            .try_sub_assign(amount)
            .expect("Insufficient balance for transfer");
        self.accounts
            .insert(&account, balance)
            .expect("Failed to insert");
    }
}
