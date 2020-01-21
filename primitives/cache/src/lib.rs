#![cfg_attr(not(feature = "std"), no_std)]

use sp_externalities::{decl_extension, ExternalitiesExt};
use sp_lfs_core::{LfsId, LfsReference};
use sp_runtime_interface::runtime_interface;

#[cfg(feature = "std")]
pub mod shared;

/// Node-side caching interface
pub trait Cache<Key: LfsId>: Send + Sync {
	/// this cache knows of `key`
	fn exists(&self, key: &Key) -> Result<bool, ()>;
	/// Fetch the data for `key`
	fn get(&self, key: &Key) -> Result<Vec<u8>, ()>;
	// insert the data at `key`
	fn insert(&self, key: &Key, data: &Vec<u8>) -> Result<(), ()>;
	/// store data, receive the resulting key
	fn store(&self, data: &Vec<u8>) -> Result<Key, ()> {
		let key = Key::for_data(data)?;
		self.insert(&key, data).map(|_| key)
	}
	// mark the following key to be okay to drop
	fn drop(&self, key: &Key) -> Result<(), ()>;
}

pub struct FrontedCache<F, B>(F, B);

impl<F, B> FrontedCache<F, B> {
	pub fn new(front: F, back: B) -> Self {
		FrontedCache(front, back)
	}
}

impl<Key, F, B> Cache<Key> for FrontedCache<F, B>
where
	Key: LfsId,
	F: Cache<Key>,
	B: Cache<Key>,
{
	fn exists(&self, key: &Key) -> Result<bool, ()> {
		if self.0.exists(key).unwrap_or(false) {
			return Ok(true);
		}
		self.1.exists(key)
	}

	fn get(&self, key: &Key) -> Result<Vec<u8>, ()> {
		self.0.get(key).or_else(|_| match self.1.get(key) {
			Ok(d) => {
				let _ = self.0.insert(key, &d);
				Ok(d)
			}
			Err(e) => Err(e),
		})
	}

	fn insert(&self, key: &Key, data: &Vec<u8>) -> Result<(), ()> {
		let _ = self.0.insert(key, data);
		self.1.insert(key, data)
	}
	fn drop(&self, key: &Key) -> Result<(), ()> {
		let _ = self.0.drop(key);
		self.1.drop(key)
	}
}

pub trait RuntimeCacheInterface: Send + Sync {
	/// this cache knows of `key`
	fn exists(&self, key: &LfsReference) -> Result<bool, ()>;
	/// Fetch the data for `key`
	fn get(&self, key: &LfsReference) -> Result<Vec<u8>, ()>;
	// insert the data at `key`
	fn insert(&self, key: &LfsReference, data: &Vec<u8>) -> Result<(), ()>;
	// mark the following key to be okay to drop
	fn drop(&self, key: &LfsReference) -> Result<(), ()>;
}

pub struct RuntimeCacheInterfaceWrapper<C, Key>(C, core::marker::PhantomData<Key>);

impl<C, Key> core::convert::From<C> for RuntimeCacheInterfaceWrapper<C, Key>
where
	C: Cache<Key>,
	Key: LfsId,
{
	fn from(cache: C) -> Self {
		Self(cache, core::marker::PhantomData)
	}
}

impl<C, Key> RuntimeCacheInterface for RuntimeCacheInterfaceWrapper<C, Key>
where
	C: Cache<Key>,
	Key: LfsId,
{
	fn exists(&self, key: &LfsReference) -> Result<bool, ()> {
		let k = Key::try_from(key.to_vec()).map_err(|_| ())?;
		self.0.exists(&k)
	}

	fn get(&self, key: &LfsReference) -> Result<Vec<u8>, ()> {
		let k = Key::try_from(key.to_vec()).map_err(|_| ())?;
		self.0.get(&k)
	}

	fn insert(&self, key: &LfsReference, data: &Vec<u8>) -> Result<(), ()> {
		let k = Key::try_from(key.to_vec()).map_err(|_| ())?;
		self.0.insert(&k, data)
	}

	fn drop(&self, key: &LfsReference) -> Result<(), ()> {
		let k = Key::try_from(key.to_vec()).map_err(|_| ())?;
		self.0.drop(&k)
	}
}

decl_extension! {
	pub struct LfsCacheExt(Box<dyn RuntimeCacheInterface>);
}

impl LfsCacheExt {
	pub fn new(cache: Box<dyn RuntimeCacheInterface>) -> Self {
		LfsCacheExt(cache)
	}
}

#[runtime_interface]
pub trait LfsCacheInterface {
	/// Fetch the data for `key`
	fn get(&mut self, key: &LfsReference) -> Result<Vec<u8>, ()> {
		self.extension::<LfsCacheExt>()
			.expect("LFSCacheExtension must be present")
			.0
			.get(key)
	}
	fn exists(&mut self, key: &LfsReference) -> Result<bool, ()> {
		self.extension::<LfsCacheExt>()
			.expect("LFSCacheExtension must be present")
			.0
			.exists(key)
	}
}
