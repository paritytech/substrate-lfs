use lru::LruCache;
use parking_lot::Mutex;
use sp_lfs_core::LfsId;

/// a simple in-memory HashMap caching system
pub struct Cache<Key: LfsId> {
	inner: Mutex<LruCache<Key, Vec<u8>>>,
}

impl<Key: LfsId> Cache<Key> {
	pub fn new(cap: usize) -> Self {
		Cache {
			inner: Mutex::new(LruCache::new(cap)),
		}
	}
}

impl<Key: LfsId> sp_lfs_cache::Cache<Key> for Cache<Key> {
	fn exists(&self, key: &Key) -> Result<bool, ()> {
		Ok(self.inner.lock().contains(key))
	}

	fn get(&self, key: &Key) -> Result<Vec<u8>, ()> {
		self.inner.lock().get(key).ok_or(()).map(|v| v.clone())
	}

	fn insert(&self, key: &Key, data: &Vec<u8>) -> Result<(), ()> {
		self.inner
			.lock()
			.put(key.clone(), data.to_vec())
			.ok_or(())
			.map(|_| ())
	}
}
