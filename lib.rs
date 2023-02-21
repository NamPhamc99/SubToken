#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod erc20 {
    use ink::storage::Mapping;

    // main struct, this will be stored on blockchain
    #[ink(storage)]
    pub struct Erc20 {
        total_supply: Balance,
        balance_map: Mapping<AccountId, Balance>,
        allowances: Mapping<(AccountId, AccountId), Balance>
    }

    #[ink(event)]
    pub struct Transfer {
        from: Option<AccountId>,
        to: Option<AccountId>,
        value: Balance
    }

    // enum to handle emitting and returning error
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    // #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
        InsuffcientAllowance
    }

    impl Erc20 {
        #[ink(constructor)]
        pub fn new(total_supply: Balance) -> Self {
            let mut balance_map = Mapping::default();

            let caller: AccountId = Self::env().caller();
            balance_map.insert(caller, &total_supply);

            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: total_supply
            });

            Self {
                total_supply,
                balance_map,
                allowances: Default::default()
            }
        }

        #[ink(message)]
        pub fn get_total_supply(&self) -> Balance {
            self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balance_map.get(owner).unwrap_or_default()
        }

        #[ink(message)]
        pub fn allowance_of (&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowances.get((spender, owner)).unwrap_or_default()
        }

        #[ink(message)]
        pub fn transfer (&mut self, to: AccountId, value: Balance) -> Result<(), Error> {
            let from = self.env().caller();
            // Firstly, check balance of From account
            let from_balance = self.balance_of(from);

            if from_balance < value {
                // If insufficient balance, throws new error and emits event
                return Err(Error::InsufficientBalance)
            }

            self.balance_map.insert(from, &(from_balance - value));

            let current_to_balance = self.balance_of(to);
            
            self.balance_map.insert(to, &(current_to_balance + value));

            Ok(())
        }
    }
}
