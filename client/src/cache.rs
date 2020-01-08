use crate::config::{CacheConfig, LfsConfig};
use crate::lfs_id::LfsId;
use sc_lfs_simple_cache::{LruCache, SimpleDiskCache};
use sp_lfs_cache::FrontedCache;
use std::path::PathBuf;

pub fn from_config(
	cfg: LfsConfig,
	base_path: PathBuf,
) -> Result<FrontedCache<LruCache<LfsId>, SimpleDiskCache>, String> {
	let path_buf = {
		if cfg.cache.path.is_relative() {
			base_path.clone().join(cfg.cache.path)
		} else {
			cfg.cache.path
		}
	};
	let path = path_buf.as_path();

	if !path.exists() {
		std::fs::create_dir_all(path)
			.map_err(|e| format!("Creating lfs directory failed: {}", e))?;
	}

	let disk = SimpleDiskCache::new(path_buf)?;

	Ok(FrontedCache::new(
		LruCache::<LfsId>::new(cfg.cache.mem_limit),
		disk,
	))
}
