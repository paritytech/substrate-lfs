use frame_support::storage::generator::StorageDoubleMap;
use frame_system::Trait as SystemTrait;
use hyper::Uri;
use pallet_lfs_user_data as pallet;
use sc_client::Client;
use sc_client_api::{backend, CallExecutor};
use sp_core::storage::{StorageData, StorageKey};
use sp_lfs_core::LfsId;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::marker::PhantomData;
use std::sync::Arc;

use crate::Resolver;

#[derive(Clone)]
enum NextResolveStep {
	Glob,
	NotFound,
	End,
}

pub struct UserDataResolveIterator<L, B, E, Block: BlockT, RA, T: pallet::Trait> {
	client: Arc<Client<B, E, Block, RA>>,
	best_block: BlockId<Block>,
	root_key: T::AccountId,
	_marker: PhantomData<(T, L)>,
	uri: Uri,
	next: NextResolveStep,
}

impl<L, B, E, Block, RA, T> UserDataResolveIterator<L, B, E, Block, RA, T>
where
	B: backend::Backend<Block>,
	E: CallExecutor<Block>,
	Block: BlockT,
	L: LfsId,
	T: pallet::Trait,
{
	fn new(
		client: Arc<Client<B, E, Block, RA>>,
		best_block: BlockId<Block>,
		root_key: T::AccountId,
		uri: Uri,
	) -> Self {
		Self {
			client,
			best_block,
			root_key,
			uri,
			next: NextResolveStep::Glob,
			_marker: Default::default(),
		}
	}

	fn lookup(&self, key: &StorageKey) -> Option<L> {
		self.client
			.storage(&self.best_block, key)
			.map(|o| {
				o.map(|d| {
					L::try_from(d.0)
						.map_err(|_| println!("UserData Entry {:?} holds a non-key.", key))
						.ok()
				})
				.flatten()
			})
			.ok()
			.flatten()
	}
}

impl<L, B, E, Block, RA, T> core::iter::Iterator for UserDataResolveIterator<L, B, E, Block, RA, T>
where
	B: backend::Backend<Block>,
	E: CallExecutor<Block>,
	Block: BlockT,
	L: LfsId,
	T: pallet::Trait,
{
	type Item = L;

	fn next(&mut self) -> Option<Self::Item> {
		let mut next = self.next.clone();

		loop {
			let (key, after) = match next {
				NextResolveStep::Glob => (
					pallet::UserData::<T>::storage_double_map_final_key(
						&self.root_key,
						b".*".to_vec(),
					),
					NextResolveStep::NotFound,
				),
				NextResolveStep::NotFound => (
					pallet::UserData::<T>::storage_double_map_final_key(
						&self.root_key,
						b"_404".to_vec(),
					),
					NextResolveStep::End,
				),
				NextResolveStep::End => {
					// we are done.
					break;
				}
			};

			if let Some(l) = self.lookup(&StorageKey(key)) {
				self.next = after;
				return Some(l);
			}

			next = after
		}

		None
	}
}

/// Resolve uri via on-chain UserData
pub struct UserDataResolver<B, E, Block: BlockT, RA, T> {
	client: Arc<Client<B, E, Block, RA>>,
	_marker: PhantomData<T>,
}

impl<B, E, Block, RA, T> UserDataResolver<B, E, Block, RA, T>
where
	B: backend::Backend<Block>,
	E: CallExecutor<Block>,
	Block: BlockT,
	T: pallet::Trait,
{
	pub fn new(client: Arc<Client<B, E, Block, RA>>) -> Self {
		UserDataResolver {
			client,
			_marker: Default::default(),
		}
	}
}

impl<B, E, Block, RA, T, L> Resolver<L> for UserDataResolver<B, E, Block, RA, T>
where
	B: backend::Backend<Block>,
	E: CallExecutor<Block>,
	Block: BlockT,
	T: pallet::Trait,
	L: LfsId,
{
	/// The iterator this resolves to, must yield `LfsdId`s
	type Iterator = Box<UserDataResolveIterator<L, B, E, Block, RA, T>>;

	/// Given the uri, yield the `LfsId`s to look up
	fn resolve(&self, uri: Uri) -> Option<Self::Iterator> {
		Some(Box::new(UserDataResolveIterator::new(
			self.client.clone(),
			BlockId::Hash(self.client.chain_info().best_hash),
			T::AccountId::default(),
			uri,
		)))
	}
}
