use near_sdk::collections::LookupMap;
use near_contract_standards::fungible_token::{
    core::FungibleTokenCore,
    metadata::{FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC},
    resolver::FungibleTokenResolver,
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{
    assert_one_yocto, env, ext_contract, log, near_bindgen, AccountId, Balance, Gas,
    Promise, PanicOnDefault, PromiseOrValue, serde_json::json
};
use std::str;

//-- Sputnik DAO remote upgrade requires BLOCKCHAIN_INTERFACE low-level access
#[cfg(target_arch = "wasm32")]
use near_sdk::env::BLOCKCHAIN_INTERFACE;

const TGAS: Gas = 1_000_000_000_000;
const GAS_FOR_RESOLVE_TRANSFER: Gas = 5 * TGAS;
const GAS_FOR_FT_TRANSFER_CALL: Gas = 25 * TGAS;
const NO_DEPOSIT: Balance = 0;
const NANOSECONDS: u64 = 1_000_000_000;
type U128String = U128;

near_sdk::setup_alloc!();

mod internal;
mod storage_nep_145;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MetaToken {
    metadata: LazyOption<FungibleTokenMetadata>,
    pub accounts: LookupMap<AccountId, Balance>,
    pub owner_id: AccountId,
    pub minters: Vec<AccountId>,
    pub total_supply: Balance
}

#[near_bindgen]
impl MetaToken {
    /// Initializes the contract with the given total supply owned by the given `owner_id`.
    #[init]
    pub fn init_contract(owner_id: AccountId) -> Self {
        //validate default metadata
        // internal::default_ft_metadata().assert_valid();
        Self {
            owner_id: owner_id.clone(),
            metadata: LazyOption::new(b"m".to_vec(), None),
            accounts: LookupMap::new(b"a".to_vec()),
            minters: vec![owner_id],
            total_supply: 0
        }
    }

    // Obtener dueño del contrato
    pub fn get_owner_id(&self) -> AccountId {
        return self.owner_id.clone();
    }
    
    // Cambiar dueño del contrato
    pub fn set_owner_id(&mut self, owner_id: AccountId) {
        self.assert_owner_calling();
        assert!(env::is_valid_account_id(owner_id.as_bytes()));
        self.owner_id = owner_id.into();
    }

    // Minar tokens
    pub fn mint(&mut self, account_id: &AccountId, amount: U128String) {
        self.assert_minter(env::predecessor_account_id());
        self.mint_into(account_id, amount.0);
    }

    // Agregar nuevo minero
    pub fn add_minter(&mut self, account_id: AccountId) -> String {
        self.assert_owner_calling();
        if let Some(_) = self.minters.iter().position(|x| *x == account_id) {
            //found
            panic!("already in the list");
        }
        self.minters.push(account_id);
        "Minero agregado".to_string()
    }

    // Remover minero
    pub fn remove_minter(&mut self, account_id: &AccountId) -> String {
        self.assert_owner_calling();
        if let Some(inx) = self.minters.iter().position(|x| x == account_id) {
            //found
            let _removed = self.minters.swap_remove(inx);
        } else {
            panic!("not a minter")
        }
        "Minero removido".to_string()
    }

    // Consultar lista de mineros
    pub fn get_minters(self) -> Vec<AccountId> {
        self.minters
    }

    /// sets metadata_reference
    #[payable]
    pub fn set_metadata_reference(&mut self, reference: String, reference_hash: String) {
        assert_one_yocto();
        self.assert_owner_calling();
        let mut m = self.internal_get_ft_metadata();
        m.reference = Some(reference);
        m.reference_hash = Some(reference_hash.as_bytes().to_vec().into());
        m.assert_valid();
        self.metadata.set(&m);
    }
}

//----------------------------------------------
// ft metadata standard
// Q: Is ignoring storage costs the only reason for the re-implementation?
// A: making the user manage storage costs adds too much friction to account creation
// it's better to impede sybil attacks by other means
#[near_bindgen]
impl FungibleTokenCore for MetaToken {
    fn ft_transfer(&mut self, receiver_id: ValidAccountId, amount: U128, memo: Option<String>) {
        let sender_id = env::predecessor_account_id();
        let amount: Balance = amount.into();
        self.internal_transfer(&sender_id, receiver_id.as_ref(), amount, memo);
    }

    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: ValidAccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();
        let amount: Balance = amount.into();
        self.internal_transfer(&sender_id, receiver_id.as_ref(), amount, memo);
        // Initiating receiver's call and the callback
        // ext_fungible_token_receiver::ft_on_transfer(
        ext_ft_receiver::ft_on_transfer(
            sender_id.clone(),
            amount.into(),
            msg,
            receiver_id.as_ref(),
            NO_DEPOSIT,
            env::prepaid_gas() - GAS_FOR_FT_TRANSFER_CALL - GAS_FOR_RESOLVE_TRANSFER, // assign rest of gas to callback
        )
        .then(ext_self::ft_resolve_transfer(
            sender_id,
            receiver_id.into(),
            amount.into(),
            &env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_RESOLVE_TRANSFER,
        ))
        .into()
    }

    fn ft_total_supply(&self) -> U128 {
        self.total_supply.into()
    }

    fn ft_balance_of(&self, account_id: ValidAccountId) -> U128 {
        self.accounts.get(account_id.as_ref()).unwrap_or(0).into()
    }
}

#[near_bindgen]
impl FungibleTokenResolver for MetaToken {
    /// Returns the amount of burned tokens in a corner case when the sender
    /// has deleted (unregistered) their account while the `ft_transfer_call` was still in flight.
    /// Returns (Used token amount, Burned token amount)
    #[private]
    fn ft_resolve_transfer(
        &mut self,
        sender_id: ValidAccountId,
        receiver_id: ValidAccountId,
        amount: U128,
    ) -> U128 {
        let sender_id: AccountId = sender_id.into();
        let (used_amount, burned_amount) =
            self.int_ft_resolve_transfer(&sender_id, receiver_id, amount);
        if burned_amount > 0 {
            log!("{} tokens burned", burned_amount);
        }
        return used_amount.into();
    }
}

#[near_bindgen]
impl FungibleTokenMetadataProvider for MetaToken {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.internal_get_ft_metadata()
    }
}

#[ext_contract(ext_ft_receiver)]
pub trait FungibleTokenReceiver {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;
}

#[ext_contract(ext_self)]
trait FungibleTokenResolver {
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128;
}
