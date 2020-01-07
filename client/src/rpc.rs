use jsonrpc_core::types::error::Error as ApiError;
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

use crate::lfs_id::LfsId;
use sp_lfs_cache::Cache;

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

impl<C> LfsApi<LfsId> for LfsRpc<C>
where
	C: Cache<LfsId> + Sync + Send + Clone + 'static,
{
	fn get(&self, id: LfsId) -> Result<Vec<u8>> {
		if let Lfsd::Raw(data) = id {
			return Ok(data);
		}

		self.cache
			.clone() // FIXME: why do we have to clone here?
			.get(id)
			.map_err(|_| ApiError::invalid_params("Key not found"))
	}

	fn upload(&self, data: Vec<u8>) -> Result<LfsId> {
		self.cache
			.clone() // FIXME: why do we have to clone here?
			.store(&data)
			.map_err(|_| ApiError::invalid_params("Data could not be stored"))
	}
}
