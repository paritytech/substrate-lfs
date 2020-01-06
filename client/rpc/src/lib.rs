use jsonrpc_core::types::error::Error as ApiError;
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

use sp_lfs_cache::Cache;
use sp_lfs_core::LfsId;

const LOCAL_STORAGE_PREFIX: &'static [u8; 3] = b"lfs";

/// Substrate LFS RPC API
#[rpc]
pub trait LfsApi<Key> {
	#[rpc(name = "lfs_get")]
	fn get(&self, id: Key) -> Result<Vec<u8>>;

	#[rpc(name = "lfs_upload")]
	fn upload(&self, data: Vec<u8>) -> Result<Key>;
}

/// An implementation of System-specific RPC methods.
pub struct LfsRpc<C> {
	cache: C,
}

impl<C> LfsRpc<C> {
	/// Create new `LFS` interface given the cache.
	pub fn new(cache: C) -> Self {
		LfsRpc { cache }
	}
}

impl<C, Key> LfsApi<Key> for LfsRpc<C>
where
	C: Cache<Key> + Sync + Send + Clone + 'static,
	Key: LfsId,
{
	fn get(&self, id: Key) -> Result<Vec<u8>> {
		self.cache
			.clone() // FIXME: why do we have to clone here?
			.get(id)
			.map_err(|_| ApiError::invalid_params("Key not found"))
	}

	fn upload(&self, data: Vec<u8>) -> Result<Key> {
		self.cache
			.clone()  // FIXME: why do we have to clone here?
			.store(&data)
			.map_err(|_| ApiError::invalid_params("Data could not be stored"))
	}
}
