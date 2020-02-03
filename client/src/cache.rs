use crate::config::LfsConfig;
use crate::lfs_id::LfsId;
use sc_lfs_simple_cache::{LruCache, SimpleDiskCache};
use sp_lfs_cache::{shared::SharedCache, FrontedCache};
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
