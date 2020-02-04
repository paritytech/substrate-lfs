#![cfg_attr(not(feature = "std"), no_std)]
/// A runtime module to manage user data per accounts, using `LfsReference`s
///
use frame_support::{decl_event, decl_module, decl_storage, dispatch};
use pallet_lfs::{Module as LfsModule, Trait as LfsTrait};
use sp_lfs_core::LfsReference;
use sp_std::prelude::*;
use system::{ensure_root, ensure_signed};

/// Local alias for a named storage entry
pub type EntryKey = Vec<u8>;

pub mod guard {

	/// Type which regulates which keys are accepted
	pub trait KeyGuardian {
		/// Is this key allowed as an entry?
		fn is_allowed(_key: &[u8]) -> bool {
			false
		}
	}
	impl KeyGuardian for () {}

	pub struct DefaultUserKeys;
	impl KeyGuardian for DefaultUserKeys {
		fn is_allowed(key: &[u8]) -> bool {
			match key {
				b"settings" | b"avatar" | b"profile" | b"colors" | b"backdrop" => true,
				_ => false,
			}
		}
	}
}

use crate::guard::KeyGuardian;

/// The module's configuration trait.
pub trait Trait: system::Trait + LfsTrait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	/// Generating callback
	type Callback: From<Call<Self>> + Into<<Self as LfsTrait>::Callback>;
	/// The type that regulates, which keys are accepted
	type KeyGuard: KeyGuardian;
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
		// We store the LfsId as the Avatar for any AccountId
		UserData get(fn user_data): double_map hasher(twox_128) T::AccountId, hasher(blake2_256) EntryKey => Option<LfsReference>;
		UserDataChangeNonce get(fn nonce): double_map hasher(twox_128) T::AccountId, hasher(blake2_256) EntryKey => Option<u32>;
	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event() = default;

		pub fn update(origin, key: EntryKey, lfs_entry: LfsReference) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			if T::KeyGuard::is_allowed(&key) {
				Self::request_to_update(who, key, lfs_entry)
			} else {
				// we still eat your tokes for trying!
				Err("Key not allowed".into())
			}
		}

		pub fn root_update(origin, key: EntryKey, lfs_entry: LfsReference) -> dispatch::DispatchResult {
			let _ = ensure_root(origin)?;
			Self::request_to_update(T::AccountId::default(), key, lfs_entry)
		}

		// callback called once the LFS is confirmed
		fn data_changed(
			origin,
			who: T::AccountId,
			key: EntryKey,
			nonce: u32,
			lfs_entry: LfsReference,
		) -> dispatch::DispatchResult {
			let _ = ensure_root(origin)?;

			if Some(nonce) == Self::nonce(&who, &key) {
				if let Some(old_lfs_entry) = UserData::<T>::get(&who, &key) {
					// There was an entry stored, inform LFS to drop the lfs_entryerence (count)
					let _ = LfsModule::<T>::drop(old_lfs_entry);
				}
				// then overwrite the entry with the new value
				UserData::<T>::insert(&who, &key, lfs_entry);
				// and inform the public, that the users avatar changed
				Self::deposit_event(RawEvent::UserDataChanged(who, key))
			} else {
				// not the correct one, drop the entry from our list as we won't be using it
				let _ = LfsModule::<T>::drop(lfs_entry);
			}
			Ok(())
		}
	}
}

/// The inner functions other modules build upon
impl<T: Trait> Module<T> {
	fn request_to_update(
		who: T::AccountId,
		key: EntryKey,
		lfs_entry: LfsReference,
	) -> dispatch::DispatchResult {
		let nonce = Self::nonce(&who, &key).unwrap_or(0) + 1;
		let call: <T as Trait>::Callback =
			Call::data_changed(who.clone(), key.clone(), nonce, lfs_entry.clone()).into();

		// store first
		UserDataChangeNonce::<T>::insert(&who, &key, nonce);
		// this maybe fire directly, if the lfs_entry is already known!
		LfsModule::<T>::query(lfs_entry, (call.into(), None))?;

		Ok(())
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
		UserDataChanged(AccountId, EntryKey),
	}
);
