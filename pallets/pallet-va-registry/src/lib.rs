#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, traits::Get};
use frame_system::ensure_signed;
use sp_std::vec::Vec;
use unique_assets::traits::{Mintable};

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

    fn createRegistry(info: &Self::RegistryInfo) -> Self::RegistryId;

    /// Use the mint info to verify whether the mint is a valid action.
    /// If so, use the asset info to mint an asset.
    fn mint(asset_info: &Self::AssetInfo,
            mint_info:  &Self::MintInfo,
    ) -> Result<Self::AssetId, ()>;
}


pub trait Trait: frame_system::Trait + pallet_nft::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as TemplateModule {
        /// This is a dummy store for testing verification in template node
        Anchor get(fn get_anchor_by_id): map hasher(identity) T::Hash => Option<T::Hash>;
    }
}

decl_event!(
    pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
        Tmp(AccountId),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        DocumentNotAnchored,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        #[weight = 10_000]
        pub fn mint(origin,
                    owner_account: <T as frame_system::Trait>::AccountId,
                    commodity_info: T::CommodityInfo,
                    anchor_id: T::Hash,
                    proofs: Vec<u8>,
        ) -> dispatch::DispatchResult {
            ensure_signed(origin)?;

            // -------------
            // Verify proofs

            // Get the doc root
            let doc_root = Self::get_document_root(anchor_id)?;

            // Verify the proof against document root
            //Self::validate_proofs(&doc_root, &proofs, &static_proofs)?;


            // -------
            // Minting

            // Internal nft mint
            <pallet_nft::Module<T>>::mint(&owner_account, commodity_info)?;
            //<Self as Nft<_>>::mint(&owner_account, commodity_info)
            //    .dispatch(system::RawOrigin::Root.into())?;

            // Mint event
            // Self::deposit_event(RawEvent::Mint(..));

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn get_document_root(anchor_id: T::Hash) -> Result<T::Hash, dispatch::DispatchError> {
        //match <anchor::Module<T>>::get_anchor_by_id(*anchor_id) {
        match Self::get_anchor_by_id(anchor_id) {
            //Some(anchor_data) => Ok(anchor_data.doc_root),
            Some(anchor_data) => Ok(anchor_data),
            None => Err(Error::<T>::DocumentNotAnchored.into()),
        }
    }
}
