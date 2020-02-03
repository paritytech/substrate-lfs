use parking_lot::Mutex;
use sp_lfs_cache::Cache;
use sp_lfs_core::LfsId;
use std::collections::HashMap;

/// a simple in-memory HashMap caching system
pub struct InMemoryCache<Key: LfsId> {
	inner: Mutex<HashMap<Key, Vec<u8>>>,
}

impl<Key: LfsId> InMemoryCache<Key> {
	pub fn new() -> Self {
		InMemoryCache {
			inner: Mutex::new(HashMap::new()),
		}
	}
}

impl<Key: LfsId> Cache<Key> for InMemoryCache<Key> {
	fn exists(&self, key: &Key) -> Result<bool, ()> {
		Ok(self.inner.lock().contains_key(key))
	}

	fn get(&self, key: &Key) -> Result<Vec<u8>, ()> {
		self.inner.lock().get(key).ok_or(()).map(|v| v.clone())
	}

	fn insert(&self, key: &Key, data: &Vec<u8>) -> Result<(), ()> {
		self.inner
			.lock()
			.insert(key.clone(), data.to_vec())
			.ok_or(())
			.map(|_| ())
	}

	fn drop(&self, key: &Key) -> Result<(), ()> {
		self.inner.lock().remove(key).ok_or(()).map(|_| ())
	}
}
