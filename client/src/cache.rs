use crate::config::LfsConfig;
use crate::lfs_id::LfsId;
use sc_lfs_simple_cache::{LruCache, SimpleDiskCache};
use sp_externalities::decl_extension;
use sp_lfs_cache::{shared::SharedCache, Cache, FrontedCache};
use sp_runtime_interface::runtime_interface;
use std::any::{Any, TypeId};
use std::path::PathBuf;

pub type ClientCache = SharedCache<FrontedCache<LruCache<LfsId>, SimpleDiskCache>>;

pub fn from_config<F>(cfg: &LfsConfig, path_reverter: F) -> Result<ClientCache, String>
where
	F: Fn(PathBuf) -> Result<PathBuf, String>,
{
	let path_buf = {
		let path = cfg.cache.path.clone();
		if path.is_relative() {
			path_reverter(path)?
		} else {
			path
		}
	};
	let path = path_buf.as_path();

	if !path.exists() {
		std::fs::create_dir_all(path)
			.map_err(|e| format!("Creating lfs directory failed: {}", e))?;
	}

	let disk = SimpleDiskCache::new(path_buf)?;

	Ok(SharedCache::new(FrontedCache::new(
		LruCache::<LfsId>::new(cfg.cache.mem_limit),
		disk,
	)))
}

decl_extension! {
	pub struct LfsCache(ClientCache);
}

#[runtime_interface]
pub trait LfsCacheInterface {
	/// Fetch the data for `key`
	fn get(&mut self, key: &LfsId) -> Result<Vec<u8>, ()> {
		match self
			.extension_by_type_id(TypeId::of::<LfsCache>())
			.map(|e| Any::downcast_mut::<LfsCache>(e))
		{
			Some(Some(cache)) => cache.0.get(key),
			_ => Err(()),
		}
	}
}
