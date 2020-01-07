#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, EncodeLike};
use core::hash::Hash;
use sp_std::{fmt::Debug, prelude::*};

/// Represent a Large File System Reference
pub trait LfsId: Encode + EncodeLike + Debug + Decode + Hash + Eq + Clone {
	/// Generate the LfsId for the given data
	fn for_data(data: &Vec<u8>) -> Result<Self, ()>;
}
