#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, EncodeLike};
use core::{convert::TryFrom, hash::Hash};
use sp_std::{fmt::Debug, prelude::*};

/// To the runtime LFS References are just an opaque encoded value
pub type LfsReference = Vec<u8>;

/// Represent a Large File System Reference
pub trait LfsId:
	Encode + EncodeLike + Debug + Decode + TryFrom<LfsReference> + Hash + Eq + Clone
{
	/// Generate the LfsId for the given data
	fn for_data(data: &Vec<u8>) -> Result<Self, ()>;
}
