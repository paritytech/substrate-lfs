use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

use sp_lfs_core::LfsId;

const LOCAL_STORAGE_PREFIX: &'static [u8; 3] = b"lfs";

/// Substrate LFS RPC API
#[rpc]
pub trait LfsApi<Key> {
	#[rpc(name = "lfs_get")]
	fn get(&self, id: Key) -> Result<()>;

	#[rpc(name = "lfs_upload")]
	fn upload(&self) -> Result<Key>;
}
