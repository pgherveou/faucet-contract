#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::primitives::AccountId;
use sp_runtime::MultiAddress;

#[derive(scale::Encode)]
enum RuntimeCall {
    /// [See here for more.](https://substrate.stackexchange.com/questions/778/how-to-get-pallet-index-u8-of-a-pallet-in-runtime)
    #[codec(index = 8u8)]
    Assets(AssetsCall),
}

#[derive(scale::Encode)]
enum AssetsCall {
    #[codec(index = 8)]
    Transfer {
        #[codec(compact)]
        id: u32,
        target: MultiAddress<AccountId, ()>,
        #[codec(compact)]
        amount: u128,
    },
}

#[ink::contract]
mod faucet {
    use crate::{AssetsCall, RuntimeCall};
    use ink::env::debug_println;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum FaucetError {
        DripTooOften,
        TransferFailed,
    }

    use ink::storage::Mapping;
    #[ink(storage)]
    pub struct Faucet {
        last_access_time: Mapping<AccountId, Timestamp>,
    }

    impl Faucet {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                last_access_time: Default::default(),
            }
        }

        #[ink(message)]
        pub fn last_access_time(&self) -> Timestamp {
            let caller = self.env().caller();
            self.last_access_time.get(caller).unwrap_or_default()
        }

        #[ink(message)]
        pub fn drip(&mut self) -> Result<(), FaucetError> {
            let now = self.env().block_timestamp();
            let caller = self.env().caller();
            let last_access_time = self.last_access_time.get(caller).unwrap_or_default();
            let elapsed_time = now - last_access_time;

            debug_println!("elapsed_time:  {elapsed_time}");
            if elapsed_time < 10 * 1000 {
                return Err(FaucetError::DripTooOften);
            }

            self.last_access_time.insert(self.env().caller(), &now);
            self.env()
                .call_runtime(&RuntimeCall::Assets(AssetsCall::Transfer {
                    id: 0,
                    target: self.env().caller().into(),
                    amount: 1,
                }))
                .map_err(|_| FaucetError::TransferFailed)
        }
    }
}
