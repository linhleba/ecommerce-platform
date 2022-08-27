use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, AccountId, near_bindgen, Balance, PanicOnDefault, BorshStorageKey, Promise, PromiseOrValue};
use near_sdk::collections::{LookupMap};

mod order;
use order::*;

pub type OrderId = String;

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[near_bindgen]
struct EcommerceContract {
    pub owner_id: AccountId,
    pub orders: LookupMap<OrderId, Order>
}

#[derive(BorshDeserialize, BorshSerialize, BorshStorageKey)]
enum StorageKey {
    OrderKey
}

#[near_bindgen]
impl EcommerceContract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self { 
            owner_id, 
            orders: LookupMap::new(StorageKey::OrderKey)
        }
    }

    #[payable]
    pub fn pay_order(&mut self, order_id: OrderId, order_amount: U128) -> PromiseOrValue<U128> {
  
        assert!(env::attached_deposit() >= order_amount.0, "ERROR_DEPOSIT_NOT_ENOUGH");

        // Store user info payment
        let order: Order = Order { 
            order_id: order_id.clone(), 
            payer_id: env::signer_account_id(), 
            amount: order_amount.0, 
            received_amount: env::attached_deposit(), 
            is_completed: true, 
            is_refund: false, 
            created_at: env::block_timestamp()
        };

        self.orders.insert(&order_id, &order);

        if env::attached_deposit() > order_amount.0 {
            Promise::new(env::signer_account_id()).transfer(env::attached_deposit() - order_amount.0);
            PromiseOrValue::Value(U128(env::attached_deposit() - order_amount.0))
        } else {
            PromiseOrValue::Value(U128(0))
        }
    }

    pub fn get_order(&self, order_id: OrderId) -> Order {
        self.orders.get(&order_id).expect("NOT_FOUND_ORDER_ID")
    }

    pub fn refund(&mut self, order_id: OrderId) -> bool {
        let order : Order = self.orders.get(&order_id).unwrap();
        assert!(!order.is_refund, "This transaction was refunded");
        Promise::new(env::signer_account_id()).transfer(order.amount);
        let _newOrder:Order = Order { 
            order_id: order.order_id, 
            payer_id: env::signer_account_id(), 
            amount: order.amount, 
            received_amount: order.received_amount, 
            is_completed: order.is_completed, 
            is_refund: true , 
            created_at: order.created_at
        };
        self.orders.insert(&order_id, &_newOrder);
        true
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use near_sdk::test_utils::{VMContextBuilder, accounts};
    use near_sdk::{testing_env, MockedBlockchain};

    fn get_context(is_view: bool) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
        .current_account_id(accounts(0))
        .signer_account_id(accounts(0))
        .predecessor_account_id(accounts(0))
        .is_view(is_view);

        builder
    }

    #[test]
    fn test_pay_order() {
        let mut context = get_context(false);
        let alice: AccountId = accounts(0);

        context.account_balance(1000)
        .predecessor_account_id(alice.clone())
        .attached_deposit(1000)
        .signer_account_id(alice.clone());

        testing_env!(context.build());

        let mut contract = EcommerceContract::new(alice.clone());
        let order_amount = U128(1000);
        contract.pay_order("order_1".to_owned(), order_amount);

        let order = contract.get_order("order_1".to_owned());

        assert_eq!(order.order_id, "order_1".to_owned());
        assert_eq!(order.amount, order_amount.0);
        assert_eq!(order.payer_id, alice);
        assert!(order.is_completed);
    }

    #[test]
    #[should_panic(expected = "ERROR_DEPOSIT_NOT_ENOUGH")]
    fn test_pay_order_with_lack_balance() {
        let mut context = get_context(false);
        let alice: AccountId = accounts(0);

        context.account_balance(1000)
        .predecessor_account_id(alice.clone())
        .attached_deposit(1000)
        .signer_account_id(alice.clone());

        testing_env!(context.build());

        let mut contract = EcommerceContract::new(alice.clone());
        let order_amount = U128(2000);
        contract.pay_order("order_1".to_owned(), order_amount);
    }
}