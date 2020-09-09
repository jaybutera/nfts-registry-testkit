#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_module, decl_storage, decl_event, decl_error,
    ensure, dispatch, traits::Get};
use frame_system::ensure_signed;
use sp_std::{vec::Vec, cmp::Eq};
use pallet_nft::InRegistry;
use unique_assets::traits::{Unique, Nft, Mintable};
use codec::{Decode, Encode};
use proofs::Proof;


// TODO: tmp until integrated w/ cent chain
mod proofs;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// Registries are identified using a nonce in storage
type RegistryId = u128;

// TODO: Get this from pallet_nft
type AssetId<T> = <T as frame_system::Trait>::Hash;

// Metadata for a registry instance
#[derive(Encode, Decode, Clone, PartialEq, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RegistryInfo {
    owner_can_burn: bool,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct AssetInfo {
    registry_id: RegistryId,
}

impl InRegistry for AssetInfo {
    fn registry_id(&self) -> RegistryId {
        self.registry_id
    }
}

// Info needed to provide proofs to mint
#[derive(Encode, Decode, Clone, PartialEq, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MintInfo<Hash> {
    anchor_id: Hash,
    proofs: Vec<Proof>,
}

pub trait VerifierRegistry {
    type AccountId;
    type RegistryId;
    type RegistryInfo;
    type AssetId;
    // Asset info must contain its associated registry id
    type AssetInfo: InRegistry;
    type MintInfo;

    fn create_registry(info: &Self::RegistryInfo) -> Result<Self::RegistryId, dispatch::DispatchError>;

    /// Use the mint info to verify whether the mint is a valid action.
    /// If so, use the asset info to mint an asset.
    fn mint(owner_account: Self::AccountId,
            asset_info: Self::AssetInfo,
            mint_info: Self::MintInfo,
    ) -> Result<Self::AssetId, dispatch::DispatchError>;
}


pub trait Trait: frame_system::Trait + pallet_nft::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as VARegistry {
        /// This is a dummy store for testing verification in template node.
        Anchor get(fn get_anchor_by_id): map hasher(identity) T::Hash => Option<T::Hash>;
        /// Nonce for generating new registry ids.
        RegistryNonce: RegistryId;
        /// A mapping of all created registries and their metadata.
        Registries: map hasher(blake2_128_concat) RegistryId => RegistryInfo;
        /// A list of asset ids for each registry.
        // TODO: Try a map of BTreeSets as well, and do a benchmark comparison
        NftLists: double_map hasher(identity) RegistryId, hasher(identity) AssetId<T> => ();
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
        RegistryDoesNotExist,
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
        pub fn create_registry(origin,
                               info: RegistryInfo,
        ) -> dispatch::DispatchResult {
            ensure_signed(origin)?;

            let registry_id = <Self as VerifierRegistry>::create_registry(&info)?;

            // Emit event
            // ...

            Ok(())
        }

        #[weight = 10_000]
        pub fn mint(origin,
                    owner_account: <T as frame_system::Trait>::AccountId,
                    commodity_info: T::CommodityInfo,
                    mint_info: MintInfo<<T as frame_system::Trait>::Hash>,
        ) -> dispatch::DispatchResult {
            ensure_signed(origin)?;

            // Internal mint validates proofs and modifies state or returns error
            let commodity_id = <Self as VerifierRegistry>::mint(owner_account,
                                                                commodity_info,
                                                                mint_info)?;

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

    fn create_new_registry_id() -> Result<RegistryId, dispatch::DispatchError> {
        let id = <RegistryNonce>::get();

        // Check for overflow on index
        let nplus1 = <RegistryNonce>::get().checked_add(1)
            .ok_or("Overflow when updating registry nonce.")?;

        // Update the nonce
        <RegistryNonce>::put( nplus1 );

        Ok(id)
    }
}

impl<T: Trait> VerifierRegistry for Module<T> {
    type AccountId    = <T as frame_system::Trait>::AccountId;
    type RegistryId   = RegistryId;
    type RegistryInfo = RegistryInfo;
    // TODO: Can these types be connected to pallet::nft?
    type AssetId   = AssetId<T>;//<T as frame_system::Trait>::Hash;
    type AssetInfo = <T as pallet_nft::Trait>::CommodityInfo;
    type MintInfo  = MintInfo<<T as frame_system::Trait>::Hash>;

    // Registries with identical RegistryInfo may exist
    fn create_registry(info: &Self::RegistryInfo) -> Result<Self::RegistryId, dispatch::DispatchError> {
        // Generate registry id as nonce
        let id = Self::create_new_registry_id()?;

        // Insert registry in storage
        Registries::insert(id.clone(), info);

        Ok(id)
    }

    fn mint(owner_account: <T as frame_system::Trait>::AccountId,
            commodity_info: T::CommodityInfo,
            mint_info: MintInfo<<T as frame_system::Trait>::Hash>,
    ) -> Result<Self::AssetId, dispatch::DispatchError> {
        // Check that given registry exists
        let registry_id = commodity_info.registry_id();
        ensure!(
            Registries::contains_key(registry_id),
            Error::<T>::RegistryDoesNotExist
        );

        // -------------
        // Verify proofs

        // Get the doc root
        // TODO: Use this line instead, once integrated with cent chain
        // let anchor_data = <anchor::Module<T>>::get_anchor_by_id(anchor_id).ok_or("Anchor doesn't exist")?;
        /*** Tmp replacement for tests: ***/
        let doc_root = Self::get_document_root(mint_info.anchor_id)?;

        // Verify the proof against document root
        // TODO: Once integrated w/ cent chain
        //Self::validate_proofs(&doc_root, &proofs, &static_proofs)?;


        // -------
        // Minting

        // Internal nft mint
        let commodity_id = <pallet_nft::Module<T>>::mint(&owner_account, commodity_info)?;

        // Place asset id in registry map
        NftLists::<T>::insert(registry_id, commodity_id, ());

        Ok(commodity_id)
    }
}
