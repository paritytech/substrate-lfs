use sp_lfs_cache::Cache;
use sp_lfs_core::LfsId;
use std::fs;
use std::path::PathBuf;

/// a super simplistic disk cache
pub struct SimpleDiskCache {
	path: PathBuf,
}

impl SimpleDiskCache {
	pub fn new(path: PathBuf) -> Result<Self, String> {
		if !path.as_path().is_dir() {
			return Err(format!(
				"{:?} is not an accessible directory",
				path.as_path()
			));
		}
		Ok(SimpleDiskCache { path })
	}
	fn make_local_path<Key: LfsId>(&self, key: &Key) -> PathBuf {
		let encoded = base64::encode_config(&key.encode(), base64::URL_SAFE);
		let mut path = self.path.clone();
		path.push(encoded);
		path
	}
}

impl<Key> Cache<Key> for SimpleDiskCache
where
	Key: LfsId,
{
	fn exists(&self, key: &Key) -> Result<bool, ()> {
		let path = self.make_local_path(key);
		Ok(path.as_path().exists())
	}

	fn get(&self, key: &Key) -> Result<Vec<u8>, ()> {
		let path = self.make_local_path(key);
		fs::read(path).map_err(|_| ())
	}

	fn insert(&self, key: &Key, data: &Vec<u8>) -> Result<(), ()> {
		let path = self.make_local_path(key);
		fs::write(path, data).map_err(|_| ())
	}
}
