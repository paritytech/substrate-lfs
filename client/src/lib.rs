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

impl DefaultClient {
	/// get a reference to the inner client cache
	pub fn cache(&self) -> &cache::ClientCache {
		&self.cache
	}
}
pub use sp_lfs_cache::lfs_cache_interface;

pub struct LfsExtensionsFactory(cache::ClientCache);
impl sc_client_api::execution_extensions::ExtensionsFactory for LfsExtensionsFactory {
	fn extensions_for(
		&self,
		capabilities: sp_core::offchain::Capabilities,
	) -> sp_externalities::Extensions {
		let mut exts = sp_externalities::Extensions::new();
		if capabilities != sp_core::offchain::Capabilities::none() {
			// only offer feature in offchain workers
			let inner: sp_lfs_cache::RuntimeCacheInterfaceWrapper<_, _> = self.0.clone().into();
			exts.register(sp_lfs_cache::LfsCacheExt::new(Box::new(inner)));
		}
		exts
	}
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

	pub fn make_externalities_extension_factory(&self) -> Box<LfsExtensionsFactory> {
		Box::new(LfsExtensionsFactory(self.cache.clone()))
	}
}
