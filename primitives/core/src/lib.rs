pub use codec::{Decode, Encode};
use core::hash::Hash;
use sp_std::prelude::*;

/// Represent a Large File System Id
pub trait LfsId: Encode + Decode + Hash + Eq + Clone{
	fn for_data(data: &Vec<u8>) -> Result<Self, ()>;
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}
}
