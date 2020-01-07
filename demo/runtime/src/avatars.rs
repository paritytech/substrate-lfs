/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs
use frame_support::{decl_event, decl_module, decl_storage, dispatch};
use pallet_lfs::{Module as LfsModule, Trait as LfsTrait};
use sp_runtime::traits::StaticLookup;
use system::ensure_signed;

/// The module's configuration trait.
pub trait Trait: system::Trait + LfsTrait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	/// Generating callback
	type Callback: From<Call<Self>> + Into<<Self as LfsTrait>::Callback>;
}

pub type AvatarId<T> = <T as LfsTrait>::LfsId;

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
		// We store the LfsId as the Avatar for any AccountId
		Avatars get(fn avatars): map T::AccountId => Option<AvatarId<T>>;
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

		pub fn request_to_change_avatar(origin, key: AvatarId<T>) -> dispatch::DispatchResult {
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
		fn avatar_changed(origin, nonce: u32, key: AvatarId<T>) -> dispatch::DispatchResult {
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

/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use frame_support::{assert_ok, impl_outer_origin, parameter_types, weights::Weight};
	use sp_core::H256;
	use sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, IdentityLookup},
		Perbill,
	};

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	}
	impl system::Trait for Test {
		type Origin = Origin;
		type Call = ();
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
		type ModuleToIndex = ();
	}
	impl Trait for Test {
		type Event = ();
	}
	type TemplateModule = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> sp_io::TestExternalities {
		system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap()
			.into()
	}

	#[test]
	fn it_works_for_default_value() {
		new_test_ext().execute_with(|| {
			// Just a dummy test for the dummy funtion `do_something`
			// calling the `do_something` function with a value 42
			assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
			// asserting that the stored value is equal to what we stored
			assert_eq!(TemplateModule::something(), Some(42));
		});
	}
}
