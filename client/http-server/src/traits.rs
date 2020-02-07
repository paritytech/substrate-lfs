use base64;
use sp_lfs_core::LfsId;
use hyper::Uri;

/// This can resolve a path into a set of
/// LfsIds we'd like to check for
pub trait Resolver<L: LfsId> {
	/// The iterator this resolves to, must yield `LfsdId`s
	type Iterator: core::iter::Iterator<Item = L>;

	/// Given the path, yield the `LfsId`s to look up
	fn resolve(&self, uri: Uri) -> Option<Self::Iterator>;

	/// helper to decode a base64 encoded string to Lfs
	fn b64decode_key<'a>(&self, id: &'a str) -> Option<L> {
		base64::decode_config(id, base64::URL_SAFE)
			.ok()
			.map(|id| L::try_from(id).ok())
			.flatten()
	}
}

/// Default implementation just takes the entire path,
/// excluding the starting slash, and attempts to decode that
impl<L: LfsId> Resolver<L> for () {
	type Iterator = std::vec::IntoIter<L>;
	fn resolve(&self, uri: Uri) -> Option<Self::Iterator> {
		let (_, pure_path) = uri.path().split_at(1);
		self.b64decode_key(pure_path).map(|id| vec![id].into_iter())
	}
}
