use parking_lot::Mutex;
use sp_lfs_cache::Cache;
use sp_lfs_core::LfsId;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// a simple in-memory HashMap caching system
pub struct SimpleInMemoryCache<Key: LfsId> {
	inner: Mutex<HashMap<Key, Vec<u8>>>,
}

impl<Key: LfsId> SimpleInMemoryCache<Key> {
	pub fn new() -> Self {
		SimpleInMemoryCache {
			inner: Mutex::new(HashMap::new()),
		}
	}
}

impl<Key: LfsId> Cache<Key> for SimpleInMemoryCache<Key> {
	fn exists(self, key: Key) -> Result<bool, ()> {
		Ok(self.inner.lock().contains_key(&key))
	}

	fn get(self, key: Key) -> Result<Vec<u8>, ()> {
		self.inner.lock().get(&key).ok_or(()).map(|v| v.clone())
	}

	fn put(self, key: Key, data: Vec<u8>) -> Result<(), ()> {
		self.inner.lock().insert(key, data);
		Ok(())
	}
}

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
	fn make_local_path<Key: LfsId>(self, key: &Key) -> PathBuf {
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
	fn exists(self, key: Key) -> Result<bool, ()> {
		let path = self.make_local_path(&key);
		Ok(path.as_path().exists())
	}

	fn get(self, key: Key) -> Result<Vec<u8>, ()> {
		let path = self.make_local_path(&key);
		fs::read(path).map_err(|_| ())
	}

	fn put(self, key: Key, data: Vec<u8>) -> Result<(), ()> {
		let path = self.make_local_path(&key);
		fs::write(path, data).map_err(|_| ())
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}
}
