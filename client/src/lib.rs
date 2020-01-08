#![cfg_attr(not(feature = "std"), no_std)]

pub mod lfs_id;

#[cfg(feature = "std")]
pub mod cache;
#[cfg(feature = "std")]
pub mod config;

#[cfg(feature = "jsonrpc")]
pub mod rpc;

#[cfg(feature = "std")]
pub struct DefaultClient {
	cache: cache::ClientCache,
}

#[cfg(feature = "std")]
impl DefaultClient {
	pub fn from_config<F: Fn(std::path::PathBuf) -> Result<std::path::PathBuf, String>>(
		cfg: &config::LfsConfig,
		converter: F,
	) -> Result<Self, String> {
		Ok(DefaultClient {
			cache: cache::from_config(cfg, converter)?,
		})
	}

	#[cfg(feature = "jsonrpc")]
	pub fn make_rpc(&self) -> rpc::LfsRpc<cache::ClientCache> {
		rpc::LfsRpc::new(self.cache.clone())
	}
}
