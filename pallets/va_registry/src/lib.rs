#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, traits::Get};
use frame_system::ensure_signed;
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

trait VerifierRegistry {
    type RegistryId;
    type RegistryInfo;
    type AssetId;
    type AssetInfo;
    type MintInfo;

    fn createRegistry(&Self::RegistryInfo) -> Self::RegistryId;

    /// Use the mint info to verify whether the mint is a valid action.
    /// If so, use the asset info to mint an asset.
    fn mint(asset_info: &Self::AssetInfo,
            mint_info:  &Self::MintInfo,
    ) -> Result<Self::AssetId, ()>;
}


pub trait Trait: frame_system::Trait /* + nft::Mintable */ {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as TemplateModule {
        //Something get(fn something): Option<u32>;
    }
}

decl_event!(
    pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
        Tmp(AccountId),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        NoneValue,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        #[weight = 10_000]
        pub fn mint(origin,
                    owner_account: T::AccountId,
                    commodity_info: T::CommodityInfo,
                    proofs: Vec<u8>,
        ) -> dispatch::DispatchResult {
            ensure_signed(origin)?;

            // Verify proofs
            // ..

            // Mint
            <Self as Mintable<_>>::mint(&owner_account, commodity_info)?;

            // Mint event
            // Self::deposit_event(RawEvent::Mint(..));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn do_something(origin, something: u32) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;

            // Update storage.
            Something::put(something);

            // Emit an event.
            Self::deposit_event(RawEvent::SomethingStored(something, who));
            // Return a successful DispatchResult
            Ok(())
        }
    }
}
