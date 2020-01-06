use sp_lfs_core::LfsId;

/// Node-side caching interface
pub trait Cache<Key> {
	/// this cache knows of `key`
	fn exists(self, key: Key) -> Result<bool, ()>;
	/// Fetch the data for `key`
	fn get(self, key: Key) -> Result<Vec<u8>, ()>;
	/// store data, retrieve key
	fn store(self, data: &Vec<u8>) -> Result<Key, ()>;
}
