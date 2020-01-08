use sp_lfs_core::LfsId;

#[cfg(feature = "std")]
pub mod shared;

/// Node-side caching interface
pub trait Cache<Key: LfsId>: std::marker::Sized {
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
