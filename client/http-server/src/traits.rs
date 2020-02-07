use crate::helpers::b64decode;
use hyper::Uri;
use sp_lfs_core::LfsId;

/// This can resolve a path into a set of
/// LfsIds we'd like to check for
pub trait Resolver<L: LfsId>: Clone {
	/// The iterator this resolves to, must yield `LfsdId`s
	type Iterator: core::iter::Iterator<Item = L>;

	/// Given the path, yield the `LfsId`s to look up
	fn resolve(&self, uri: Uri) -> Option<Self::Iterator>;
}

/// Default implementation just takes the entire path,
/// excluding the starting slash, and attempts to decode that
impl<L: LfsId> Resolver<L> for () {
	type Iterator = std::vec::IntoIter<L>;
	fn resolve(&self, uri: Uri) -> Option<Self::Iterator> {
		let (_, pure_path) = uri.path().split_at(1);
		b64decode::<L>(pure_path).map(|id| vec![id].into_iter())
	}
}
