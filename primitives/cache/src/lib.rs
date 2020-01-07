use sp_lfs_core::LfsId;

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
}

pub struct MultiCache<C>(Vec<C>);

impl<C> MultiCache<C> {
	fn new(providers: Vec<C>) -> Self {
		MultiCache(providers)
	}
}

impl<Key, C> Cache<Key> for MultiCache<C>
where
	Key: LfsId,
	C: Cache<Key>,
{
	fn exists(&self, key: &Key) -> Result<bool, ()> {
		Ok(self.0.iter().any(|p| p.exists(key).unwrap_or(false)))
	}
	/// Fetch the data for `key`
	fn get(&self, key: &Key) -> Result<Vec<u8>, ()> {
		let mut prev: Vec<&C> = vec![];
		self.0
			.iter()
			.find_map(|p| match p.get(key) {
				Ok(d) => {
					prev.iter().for_each(|i| {
						let _ = i.insert(key, &d);
					});

					Some(d)
				}
				_ => {
					prev.push(p);
					None
				}
			})
			.ok_or(())
	}
	// insert the data at `key`
	fn insert(&self, key: &Key, data: &Vec<u8>) -> Result<(), ()> {
		self.0
			.iter()
			.map(|p| p.insert(key, data))
			.find_map(|r| r.ok())
			.ok_or(())
	}
}
