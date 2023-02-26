#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod erc20 {
    use ink::storage::Mapping;

    // main struct, this will be stored on blockchain
    #[ink(storage)]
    pub struct Erc20 {
        total_supply: Balance,
        balance_map: Mapping<AccountId, Balance>,
        allowances: Mapping<(AccountId, AccountId), Balance> // key is (spender, owner), Balance is the maximum amount allowed
    }

    #[ink(event)]
    pub struct Transfer {
        from: Option<AccountId>,
        to: Option<AccountId>,
        value: Balance
    }

    #[ink(event)]
    pub struct Approval {
        owner: AccountId,
        spender: AccountId,
        max_spending: Balance
    }

    // enum to handle emitting and returning error
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
        InsuffcientAllowance
    }

    impl Erc20 {
        #[ink(message)]
        pub fn allowance_of (&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowances.get((spender, owner)).unwrap_or_default()
        }

        #[ink(message)]
        pub fn approve (&mut self, spender: AccountId, max_spending: Balance) -> Result<(), Error> {
            let owner = self.env().caller();
            self.allowances.insert((&spender, &owner), &max_spending);
            self.env().emit_event(Approval {
                owner,
                spender,
                max_spending
            });

            Ok(())
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balance_map.get(owner).unwrap_or_default()
        }

        fn check_transfer (&self, from: &AccountId, value: Balance) -> Result<(), Error> {
            let from_balance = self.balance_map.get(from).unwrap_or(0);
            
            if from_balance < value {
                Err(Error::InsufficientBalance)
            } else {
                Ok(())
            }
        }

        #[ink(message)]
        pub fn get_total_supply(&self) -> Balance {
            self.total_supply
        }

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
        pub fn transfer (&mut self, to: AccountId, value: Balance) -> Result<(), Error> {
            let from = self.env().caller();

            self.transfer_from_to(from, to, value)
        }

        fn transfer_from_to (&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<(), Error> {
            // Firstly, check balance of From account
            let check_transfer = self.check_transfer(&from, value);

            match check_transfer {
                Ok(()) => {
                    let from_balance = self.balance_of(from);
                    let current_to_balance = self.balance_of(to);

                    self.balance_map.insert(from, &(from_balance - value));
                    self.balance_map.insert(to, &(current_to_balance + value));
    
                    Ok(())
                }

                Err(Error::InsufficientBalance) => {
                    Err(Error::InsufficientBalance)
                }

                Err(Error::InsuffcientAllowance) => {
                    Err(Error::InsuffcientAllowance)
                }
            }
        }

        #[ink(message)]
        pub fn transfer_with_allowance (&mut self, owner: AccountId, to: AccountId, value: Balance) -> Result<(), Error> {
            let spender = self.env().caller();
            let allowance_of_caller = self.allowances.get((&spender, &owner)).unwrap_or_default();

            if allowance_of_caller < value {
                return Err(Error::InsuffcientAllowance);
            } else {
                // Deduct the allowances
                let remaining_allowance = match ((value - allowance_of_caller) > 0) {
                    true => value - allowance_of_caller,
                    false => 0
                };
                self.allowances.insert((&spender, &owner), &remaining_allowance);

                return self.transfer_from_to(owner, to, value);
            }
        }
    }
}
