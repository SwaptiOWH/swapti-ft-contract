use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{AccountId, Balance, PromiseResult};

use crate::*;

const ONE_NEAR: Balance = 1_000_000_000_000_000_000_000_000;
pub const MIN_TRANSFER_UNIT: u128 = 1000;
const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/jpeg;base64,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";


pub fn default_ft_metadata() -> FungibleTokenMetadata {
    FungibleTokenMetadata {
        spec: FT_METADATA_SPEC.to_string(),
        name: "$SWAPTI".to_string(),
        symbol: "ST".to_string(),
        icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
        reference: None,
        reference_hash: None,
        decimals: 24,
    }
}

impl MetaToken {
    pub fn assert_owner_calling(&self) {
        assert!(
            env::predecessor_account_id() == self.owner_id,
            "can only be called by the owner"
        );
    }

    pub fn assert_minter(&self, account_id: String) {
        assert!(self.minters.contains(&account_id), "not a minter");
    }

    pub fn internal_get_ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap_or(default_ft_metadata())
    }

    pub fn internal_unwrap_balance_of(&self, account_id: &AccountId) -> Balance {
        self.accounts.get(&account_id).unwrap_or(0)
    }

    pub fn mint_into(&mut self, account_id: &AccountId, amount: Balance) {
        let balance = self.internal_unwrap_balance_of(account_id);
        self.internal_update_account(&account_id, balance + amount);
        self.total_supply += amount;
    }

    pub fn internal_burn(&mut self, account_id: &AccountId, amount: u128) {
        let balance = self.internal_unwrap_balance_of(account_id);
        assert!(balance >= amount);
        self.internal_update_account(&account_id, balance - amount);
        assert!(self.total_supply >= amount);
        self.total_supply -= amount;
    }

    pub fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        amount: Balance,
        memo: Option<String>,) {
        assert_ne!(
            sender_id, receiver_id,
            "Sender and receiver should be different"
        );

        let sender_balance = self.internal_unwrap_balance_of(sender_id);
        assert!(
            amount == sender_balance || amount > ONE_NEAR / MIN_TRANSFER_UNIT,
            "The amount should be at least 1/{}",
            MIN_TRANSFER_UNIT
        );

        // remove from sender
        let sender_balance = self.internal_unwrap_balance_of(sender_id);
        assert!(
            amount <= sender_balance,
            "The account doesn't have enough balance {}",
            sender_balance
        );
        let balance_left = sender_balance - amount;
        self.internal_update_account(&sender_id, balance_left);

        // add to receiver
        let receiver_balance = self.internal_unwrap_balance_of(receiver_id);
        self.internal_update_account(&receiver_id, receiver_balance + amount);

        log!("Transfer {} from {} to {}", amount, sender_id, receiver_id);
        if let Some(memo) = memo {
            log!("Memo: {}", memo);
        }
    }

    pub fn internal_update_account(&mut self, account_id: &AccountId, balance: u128) {
        self.accounts.insert(account_id, &balance); //insert_or_update
    }

    pub fn int_ft_resolve_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: ValidAccountId,
        amount: U128,
    ) -> (u128, u128) {
        let sender_id: AccountId = sender_id.into();
        let receiver_id: AccountId = receiver_id.into();
        let amount: Balance = amount.into();

        // Get the unused amount from the `ft_on_transfer` call result.
        let unused_amount = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(value) => {
                if let Ok(unused_amount) = near_sdk::serde_json::from_slice::<U128>(&value) {
                    std::cmp::min(amount, unused_amount.0)
                } else {
                    amount
                }
            }
            PromiseResult::Failed => amount,
        };

        if unused_amount > 0 {
            let receiver_balance = self.accounts.get(&receiver_id).unwrap_or(0);
            if receiver_balance > 0 {
                let refund_amount = std::cmp::min(receiver_balance, unused_amount);
                self.accounts
                    .insert(&receiver_id, &(receiver_balance - refund_amount));

                if let Some(sender_balance) = self.accounts.get(&sender_id) {
                    self.accounts
                        .insert(&sender_id, &(sender_balance + refund_amount));
                    log!(
                        "Refund {} from {} to {}",
                        refund_amount,
                        receiver_id,
                        sender_id
                    );
                    return (amount - refund_amount, 0);
                } else {
                    // Sender's account was deleted, so we need to burn tokens.
                    self.total_supply -= refund_amount;
                    log!("The account of the sender was deleted");
                    return (amount, refund_amount);
                }
            }
        }
        (amount, 0)
    }
}
