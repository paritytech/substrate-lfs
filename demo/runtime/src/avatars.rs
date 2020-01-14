/// A runtime module to manage Avatars per accounts, using `LfsReference`s
///
use frame_support::{decl_event, decl_module, decl_storage, dispatch};
use pallet_lfs::{Module as LfsModule, Trait as LfsTrait};
use sp_lfs_core::LfsReference;
use sp_runtime::traits::StaticLookup;
use system::ensure_signed;

/// The module's configuration trait.
pub trait Trait: system::Trait + LfsTrait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	/// Generating callback
	type Callback: From<Call<Self>> + Into<<Self as LfsTrait>::Callback>;
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
		// We store the LfsId as the Avatar for any AccountId
		Avatars get(fn avatars): map T::AccountId => Option<LfsReference>;
		AvatarsChangeNonce get(fn nonce): map T::AccountId => Option<u32>;
	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event() = default;

		pub fn request_to_change_avatar(origin, key: LfsReference) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			let nonce = Self::nonce(&who).unwrap_or(0) + 1;
			let call: <T as Trait>::Callback = Call::avatar_changed(nonce, key.clone()).into();

			// store first
			AvatarsChangeNonce::<T>::insert(&who, nonce);
			// this maybe fire directly, if the key is already known!
			LfsModule::<T>::query(key, (call.into(), Some(<T::Lookup as StaticLookup>::unlookup(who))))?;

			Ok(())
		}

		// callback called once the LFS is confirmed
		fn avatar_changed(origin, nonce: u32, key: LfsReference) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			if Some(nonce) == Self::nonce(&who) {
				if let Some(old_key) = Avatars::<T>::get(&who) {
					// There was an entry stored, inform LFS to drop the reference (count)
					let _ = LfsModule::<T>::drop(old_key);
				}
				// then overwrite the entry with the new value
				Avatars::<T>::insert(&who, key);
				// and inform the public, that the users avatar changed
				Self::deposit_event(RawEvent::AvatarChanged(who))
			}
			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
		AvatarChanged(AccountId),
	}
);
