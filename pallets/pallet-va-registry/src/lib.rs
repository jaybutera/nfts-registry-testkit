#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, traits::Get};
use frame_system::ensure_signed;
use sp_std::vec::Vec;
use unique_assets::traits::{Unique, Nft, Mintable};
use proofs::Proof;

// TODO: tmp until integrated w/ cent chain
mod proofs;

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
    pub enum Event<T>
    where
        // TODO: Is it possible to get the Id directly from the Module like this?
        //CommodityId = <<pallet_nft::Module<T> as Unique>::Asset as Nft>::Id,
        CommodityId = <T as frame_system::Trait>::Hash,
    {
        Mint(CommodityId),
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
        pub fn tmp_set_anchor(origin, anchor_id: T::Hash, anchor: T::Hash
        ) -> dispatch::DispatchResult {
            Ok(<Anchor<T>>::insert(anchor_id, anchor))
        }

        #[weight = 10_000]
        pub fn mint(origin,
                    owner_account: <T as frame_system::Trait>::AccountId,
                    commodity_info: T::CommodityInfo,
                    anchor_id: T::Hash,
                    proofs: Vec<Proof>,
        ) -> dispatch::DispatchResult {
            ensure_signed(origin)?;

            // -------------
            // Verify proofs

            // Get the doc root
            // TODO: Use this line instead, once integrated with cent chain
            // let anchor_data = <anchor::Module<T>>::get_anchor_by_id(anchor_id).ok_or("Anchor doesn't exist")?;
            /*** Tmp replacement for tests: ***/
            let doc_root = Self::get_document_root(anchor_id)?;

            // Verify the proof against document root
            // TODO: Once integrated w/ cent chain
            //Self::validate_proofs(&doc_root, &proofs, &static_proofs)?;


            // -------
            // Minting

            // Internal nft mint
            let commodity_id = <pallet_nft::Module<T>>::mint(&owner_account, commodity_info)?;

            // Mint event
            Self::deposit_event(RawEvent::Mint(commodity_id));

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
