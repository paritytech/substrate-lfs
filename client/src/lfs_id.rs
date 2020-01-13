#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::hashing::{blake2_256, keccak_256, sha2_256};
pub use sp_lfs_core::{LfsId as LfsIdT, LfsReference};

use codec::{Decode, Encode};

type Hash256 = [u8; 32];

#[derive(Debug, Encode, Decode, Clone, Hash, Eq)]
/// Our Large File System ID
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum LfsId {
	/// Raw directly showing the data
	/// below a certain length (< 32 bytes), it doesn't make any sense to hash them
	#[codec(index = "0")]
	Raw(Vec<u8>),

	#[codec(index = "10")]
	Blake2(Hash256),
	Sha2(Hash256),
	Sha3(Hash256),
}

impl LfsId {
	pub fn default(data: &Vec<u8>) -> Self {
		Self::blake2(data)
	}
	pub fn blake2(data: &Vec<u8>) -> Self {
		LfsId::Blake2(blake2_256(data))
	}
	pub fn sha2(data: &Vec<u8>) -> Self {
		LfsId::Sha2(sha2_256(data))
	}
	pub fn sha3(data: &Vec<u8>) -> Self {
		LfsId::Sha3(keccak_256(data))
	}
}

impl sp_runtime_interface::pass_by::PassBy for LfsId {
	type PassBy = sp_runtime_interface::pass_by::Codec<Self>;
}

impl core::cmp::PartialEq for LfsId {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(LfsId::Raw(ref s), LfsId::Raw(ref o)) => s == o,
			(LfsId::Blake2(s), LfsId::Blake2(o)) => s == o,
			(LfsId::Sha2(s), LfsId::Sha2(o)) => s == o,
			(LfsId::Sha3(s), LfsId::Sha3(o)) => s == o,
			_ => false,
		}
	}
}

impl core::convert::TryFrom<LfsReference> for LfsId {
	type Error = String;

	fn try_from(value: LfsReference) -> Result<Self, Self::Error> {
		Self::decode(&mut value.as_slice())
			.map_err(|e| format!("Decoding LFS Reference failed: {:}", e))
	}
}

impl LfsIdT for LfsId {
	fn for_data(data: &Vec<u8>) -> Result<Self, ()> {
		if data.len() <= 32 {
			Ok(LfsId::Raw(data.clone()))
		} else {
			Ok(Self::default(data))
		}
	}
}
