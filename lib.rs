#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::env::Environment;
use ink::primitives::AccountId;
use sp_runtime::MultiAddress;

#[ink::chain_extension]
pub trait FetchAsset {
    type ErrorCode = FetchAssetError;

    #[ink(extension = 1)]
    fn transfer_token(token_id: u32, amount: u128);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum FetchAssetError {
    FetchAssetFailed,
}

impl ink::env::chain_extension::FromStatusCode for FetchAssetError {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::FetchAssetFailed),
            _ => panic!("encountered unknown status code"),
        }
    }
}

impl Environment for CustomEnvironment {
    const MAX_EVENT_TOPICS: usize = <ink::env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <ink::env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink::env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink::env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink::env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink::env::DefaultEnvironment as Environment>::Timestamp;

    type ChainExtension = FetchAsset;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CustomEnvironment {}

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

#[ink::contract(env = crate::CustomEnvironment)]
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

        #[ink(message)]
        pub fn drip_chain_extension(&mut self) -> Result<(), FaucetError> {
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
                .extension()
                .transfer_token(0, 1)
                .map_err(|_| FaucetError::TransferFailed)
        }
    }
}
