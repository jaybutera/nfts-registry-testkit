use crate::proofs::Proof;
use pallet_nft::InRegistry;
use frame_support::dispatch;
use codec::{Decode, Encode};

// Registries are identified using a nonce in storage
pub type RegistryId = u128;

// A vector of bytes, conveniently named like it is in Solidity
pub type bytes = Vec<u8>;

// A convenience rename from pallet_nft's id type
pub type AssetId<T> = pallet_nft::CommodityId<T>;

// Metadata for a registry instance
#[derive(Encode, Decode, Clone, PartialEq, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RegistryInfo {
    pub owner_can_burn: bool,
    pub fields: Vec<bytes>,
}

/// Contains all data about an instance of an NFT.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct AssetInfo {
    pub registry_id: RegistryId,
}

// Registry id must be a field within the data, because an assets id
// is a hash of its content, and its registry is part of its uniquely
// identifying information.
impl InRegistry for AssetInfo {
    fn registry_id(&self) -> RegistryId {
        self.registry_id
    }
}

/// Info needed to provide proofs to mint
#[derive(Encode, Decode, Clone, PartialEq, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MintInfo<Hash> {
    pub anchor_id: Hash,
    pub proofs: Vec<Proof>,
    pub values: Vec<bytes>,
}

/// A general interface for registries that require some sort of verification to mint their
/// underlying NFTs. A substrate module can implement this trait.
pub trait VerifierRegistry {
    /// This should typically match the implementing substrate Module trait's AccountId type.
    type AccountId;
    /// The id type of a registry.
    type RegistryId;
    /// Metadata for an instance of a registry.
    type RegistryInfo;
    /// The id type of an NFT.
    type AssetId;
    /// The data that defines the NFT held by a registry. Asset info must contain its
    /// associated registry id.
    type AssetInfo: InRegistry;
    /// All data necessary to determine if a requested mint is valid or not.
    type MintInfo;

    /// Create a new instance of a registry with the associated registry info.
    fn create_registry(info: &Self::RegistryInfo) -> Result<Self::RegistryId, dispatch::DispatchError>;

    /// Use the mint info to verify whether the mint is a valid action.
    /// If so, use the asset info to mint an asset.
    fn mint(owner_account: Self::AccountId,
            asset_info: Self::AssetInfo,
            mint_info: Self::MintInfo,
    ) -> Result<Self::AssetId, dispatch::DispatchError>;
}
