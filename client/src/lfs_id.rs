pub use sp_lfs_core::{LfsId as LfsIdT, LfsReference};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use codec::{Decode, Encode};

#[derive(Debug, Encode, Decode, Clone, Hash, Eq)]
/// Our Large File System ID
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum LfsId {
	/// Raw directly showing the data
	/// below a certain length (< 32 bytes), it doesn't make any sense to hash them
	#[codec(index = "0")]
	Raw(Vec<u8>),
}

impl sp_runtime_interface::pass_by::PassBy for LfsId {
    type PassBy = sp_runtime_interface::pass_by::Codec<Self>;
}

impl core::cmp::PartialEq for LfsId {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(LfsId::Raw(ref s), LfsId::Raw(ref o)) => s == o,
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
		if data.len() < 32 {
			Ok(LfsId::Raw(data.clone()))
		} else {
			Err(())
		}
	}
}
