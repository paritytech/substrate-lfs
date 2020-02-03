use crate::Cache;
use sp_lfs_core::LfsId;
use std::ops::Deref;
use std::sync::Arc;

pub struct SharedCache<C>(Arc<C>);

impl<C> SharedCache<C> {
	pub fn new(cache: C) -> Self {
		SharedCache(Arc::new(cache))
	}
}

impl<C> std::clone::Clone for SharedCache<C> {
	fn clone(&self) -> Self {
		SharedCache(self.0.clone())
	}
}

impl<C, Key> Cache<Key> for SharedCache<C>
where
	C: Cache<Key>,
	Key: LfsId,
{
	fn exists(&self, key: &Key) -> Result<bool, ()> {
		self.0.exists(key)
	}
	fn get(&self, key: &Key) -> Result<Vec<u8>, ()> {
		self.0.get(key)
	}
	fn insert(&self, key: &Key, data: &Vec<u8>) -> Result<(), ()> {
		self.0.insert(key, data)
	}
	fn drop(&self, key: &Key) -> Result<(), ()> {
		self.0.deref().drop(key)
	}
}
