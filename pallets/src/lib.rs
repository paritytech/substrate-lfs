#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Codec, Decode, Encode};
use sp_runtime::app_crypto::KeyTypeId;
use sp_runtime::{
	traits::{Dispatchable, StaticLookup},
	DispatchError,
};
use sp_std::prelude::*;
use support::{
	decl_event, decl_module, decl_storage, dispatch::DispatchResult, Parameter, StorageValue,
};
use system::offchain::SubmitSignedTransaction;
use system::{ensure_root, ensure_signed};

use sp_lfs_core::LfsReference;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"lfs0");

pub mod sr25519 {
	use super::KEY_TYPE;
	use sp_runtime::app_crypto::{app_crypto, sr25519};
	app_crypto!(sr25519, KEY_TYPE);
}

pub mod ed25519 {
	use super::KEY_TYPE;
	use sp_runtime::app_crypto::{app_crypto, ed25519};
	app_crypto!(ed25519, KEY_TYPE);
}

/// The module's configuration trait.
pub trait Trait: system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	/// Offchain Worker Call
	type OcwCall: From<Call<Self>>;

	/// The callback type to call
	type Callback: Parameter + Dispatchable<Origin = Self::Origin> + Codec + Eq;

	/// Let's define the helper we use to create signed transactions with
	type SubmitTransaction: SubmitSignedTransaction<Self, <Self as Trait>::OcwCall>;
}

#[derive(Encode, Decode)]
/// Calls triggered to the offchain worker
pub enum LfsOffchainEvent {
	/// Represents a query issued
	Query(LfsReference),
	/// Inform the Offchain Worker that the entry has been resolved
	/// meaning, they probably do not want to waste resources responding again
	Resolved(LfsReference),
	/// This entry has been dropped from the internal listing
	Dropped(LfsReference),
}

#[derive(Encode, Decode)]
/// The LFS state
enum LfsEntryState<T: Trait> {
	/// This entry is pending and hasn't been resolved yet
	Pending {
		/// Since when the Block is pending
		since: T::BlockNumber,
		/// callbacks to call once resolved
		listeners: Vec<(
			<T as Trait>::Callback,
			Option<<T::Lookup as StaticLookup>::Source>,
		)>,
	},
	Resolved {
		/// first confirmed to exist
		since: T::BlockNumber,
		/// latest confirmation it exists
		latest: T::BlockNumber,
		/// how many internally still refer to this entry
		ref_count: u32,
	},
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as LFS {
		/// our record of offchain calls triggered in this block
		OcwEvents get(fn ocw_events): Vec<LfsOffchainEvent>;
		/// The current set of keys that may submit pongs
		Authorities get(fn authorities) config(): Vec<T::AccountId>;
		/// The specific LFS entries and states
		Entries: map hasher(blake2_256) LfsReference => Option<LfsEntryState<T>>;
	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event() = default;

		fn on_initialize(_now: T::BlockNumber) {
			// clean offchain calls on every block start
			<Self as Store>::OcwEvents::kill();
		}
		// Respond to an lfs entry query
		pub fn respond(origin, key: LfsReference) -> DispatchResult {
			let author = ensure_signed(origin)?;
			if !Self::is_authority(&author) {
				// No known authority, ignore
				return Ok(())
			};

			let now = <system::Module<T>>::block_number();

			if let Some(entry) = Entries::<T>::get(&key) {
				let replace = match entry {
					LfsEntryState::Pending { listeners, .. } => {
						// inform the outer OcwEventss about this
						<Self as Store>::OcwEvents::mutate(|v| v.push(LfsOffchainEvent::Resolved(key.clone())));
						let mut ref_count = 0u32;
						// inform the listeners
						for callback in listeners {
							if Self::callback(callback) {
								ref_count += 1;
							}
						}
						// replace with resolved
						if ref_count > 0 {
							LfsEntryState::Resolved {
								ref_count,
								since: now.clone(),
								latest: now,
							}
						} else {
							// we were able to resolve, but the result didn't lead to any references staying around
							Entries::<T>::remove(&key);
							return Ok(());
						}
					}
					LfsEntryState::Resolved { ref_count, since, .. } => {
						LfsEntryState::Resolved {
							since,
							ref_count,
							latest: now,
						}
					}
				};

				// replace our entry with the updated version
				Entries::<T>::insert(&key, replace);
			}

			Ok(())
		}

		fn offchain_worker(_now: T::BlockNumber) {
			if T::SubmitTransaction::can_sign() {
				let _ = Self::offchain();
			}
		}

		// Simple authority management: add a new authority to the set of keys that
		// are allowed to respond to lfs queries
		pub fn add_authority(origin, who: <T::Lookup as StaticLookup>::Source ) -> DispatchResult {
			let _me = ensure_root(origin)?;
			let account = T::Lookup::lookup(who)?;

			if !Self::is_authority(&account){
				<Authorities<T>>::mutate(|l| l.push(account));
			}

			Ok(())
		}

		// Simple authority management: remove an authority from the set of keys that
		// are allowed to respond to lfs queries
		pub fn drop_authority(origin, who: <T::Lookup as StaticLookup>::Source ) -> DispatchResult {
			// In practice this should be a bit cleverer, but for this example it is enough
			// that this is protected by a root-call (e.g. through governance like `sudo`).
			let _me = ensure_root(origin)?;
			let account = T::Lookup::lookup(who)?;

			<Authorities<T>>::mutate(|l| l.retain(|i| i != &account));

			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
		/// Triggered on a pong with the corresponding value
		Ack(u8, AccountId),
	}
);

/// The inner functions other modules build upon
impl<T: Trait> Module<T> {
	/// query for an lfs entry
	pub fn query(
		key: LfsReference,
		callback: (
			<T as Trait>::Callback,
			Option<<T::Lookup as StaticLookup>::Source>,
		),
	) -> DispatchResult {
		let now = <system::Module<T>>::block_number();
		let mut issue_query = false;
		let new_entry = match Entries::<T>::get(&key) {
			None => {
				issue_query = true;
				LfsEntryState::Pending {
					since: now,
					listeners: vec![callback],
				}
			}
			Some(mut entry) => {
				match entry {
					LfsEntryState::Pending {
						ref mut listeners, ..
					} => {
						listeners.push(callback);
					}
					LfsEntryState::Resolved {
						ref mut ref_count, ..
					} => {
						if Self::callback(callback) {
							*ref_count += 1;
						}
					}
				};
				entry
			}
		};

		Entries::<T>::insert(&key, new_entry);

		if issue_query {
			// Informing the offchain worker
			<Self as Store>::OcwEvents::mutate(|v| v.push(LfsOffchainEvent::Query(key)));
		}

		Ok(())
	}

	/// indicate that you are not using a previously resolved reference anymore
	pub fn drop(key: LfsReference) -> DispatchResult {
		if let Some(mut entry) = Entries::<T>::get(&key) {
			if let LfsEntryState::Resolved {
				ref mut ref_count, ..
			} = entry
			{
				*ref_count -= 1;
				if *ref_count == 0 {
					Entries::<T>::remove(&key);
					// Informing the offchain worker
					<Self as Store>::OcwEvents::mutate(|v| {
						v.push(LfsOffchainEvent::Dropped(key.clone()))
					});
				} else {
					Entries::<T>::insert(&key, entry);
				}
			}
		}
		Ok(())
	}

	// test
	fn callback(
		callback: (
			<T as Trait>::Callback,
			Option<<T::Lookup as StaticLookup>::Source>,
		),
	) -> bool {
		let (cb, who) = callback;
		let origin = if let Some(who) = who {
			if let Ok(sign) = T::Lookup::lookup(who) {
				system::RawOrigin::Signed(sign).into()
			} else {
				sp_runtime::print("Callback not issued, Lookup failed");
				return false;
			}
		} else {
			system::RawOrigin::Root.into()
		};

		match cb.dispatch(origin) {
			Ok(_) => true,
			Err(e) => {
				let e: DispatchError = e.into();
				sp_runtime::print(e);
				false
			}
		}
	}
}

// We've moved the  helper functions outside of the main decleration for briefety.
impl<T: Trait> Module<T> {
	/// The main entry point, called with account we are supposed to sign with
	fn offchain() {
		for e in <Self as Store>::OcwEvents::get() {
			match e {
				LfsOffchainEvent::Query(key) => {
					sp_io::misc::print_utf8(b"Received query, sending response");
					match sp_lfs_cache::lfs_cache_interface::exists(&key) {
						Ok(true) => {
							sp_io::misc::print_utf8(b"Found in local cache");
							let call = Call::respond(key);
							let _ = T::SubmitTransaction::submit_signed(call);
						}
						_ => {
							sp_io::misc::print_utf8(b"Not found");
						}
					}
				}
				_ => {}
			}
		}
	}

	/// Helper that confirms whether the given `AccountId` can sign `pong` transactions
	fn is_authority(who: &T::AccountId) -> bool {
		Self::authorities().into_iter().find(|i| i == who).is_some()
	}
}
