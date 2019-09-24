#![cfg_attr(not(feature = "std"), no_std)]

/// A runtime module for SRML

use rstd::prelude::*;
use app_crypto::RuntimeAppPublic;
use support::{decl_module, decl_storage, decl_event, StorageValue, dispatch::Result};
use system::{ensure_signed, ensure_root};
use system::offchain::SubmitSignedTransaction;
use codec::{Encode, Decode};

pub const KEY_TYPE: app_crypto::KeyTypeId = app_crypto::KeyTypeId(*b"lfso");

/// The module's configuration trait.
pub trait Trait: system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	/// A dispatchable call type. We need to define it for the offchain worker to 
	/// reference the `pong` function it wants to call.
	type Call: From<Call<Self>>;

	/// Let's define the helper we use to create signed transactions with
	type SubmitTransaction: SubmitSignedTransaction<Self, <Self as Trait>::Call>;

	/// The local keytype
	type KeyType: RuntimeAppPublic + From<Self::AccountId> + Into<Self::AccountId> + Clone;
}

#[derive(Encode, Decode, Debug)]
/// Calls triggered to the offchain worker
enum OffchainCall {
	Ping(u8) // -> Expected to call back Pong(u8)
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as LFS {
		/// our record of offchain calls triggered in this block
		Ofc: Vec<OffchainCall>;
		/// The current set of keys that may submit pongs
		Authorities get(authorities): Vec<T::AccountId>;
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
			<Self as Store>::Ofc::kill();
		}

		// The main entry point: send the ping to the offchain worker
		pub fn send_ping(origin, nonce: u8) -> Result {
			// Ensure we are charging the sender
			let _who = ensure_signed(origin)?;

			// Informing the offchain worker
			<Self as Store>::Ofc::mutate(|v| v.push(OffchainCall::Ping(nonce)));

			Ok(())
		}

		// The pong from the offchain worker
		pub fn pong(origin, nonce: u8) -> Result {
			let author = ensure_signed(origin)?;

			// we would be reacting here, but at the moment, we only
			// issue the `Ack`-Event to show it passed.
			
			if Self::is_authority(&author) {
				Self::deposit_event(RawEvent::Ack(nonce, author));
			}

			Ok(())
		}


		// Runs after every block within the context and current state of said block.
		fn offchain_worker(_now: T::BlockNumber) {
			// As `pongs` are only accepted by authorities, we only run this code,
			// if a valid local key is found, we could submit them with.
			if let Some(key) = Self::authority_id() {
				runtime_io::print_utf8(b"trigger offchain");
				Self::offchain(&key);
			} else {
				runtime_io::print_utf8(b"Not authority");
			}
		}

		// Simple authority management: add a new authority to the set of keys that
		// are allowed to respond with `pong`.
		pub fn add_authority(origin, who: T::AccountId) -> Result {
			// In practice this should be a bit cleverer, but for this example it is enough
			// that this is protected by a root-call (e.g. through governance like `sudo`).
			let _me = ensure_root(origin)?;

			if !Self::is_authority(&who){
				<Authorities<T>>::mutate(|l| l.push(who));
			}

			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		/// Triggered on a pong with the corresponding value
		Ack(u8, AccountId),
	}
);



// We've moved the  helper functions outside of the main decleration for briefety.
impl<T: Trait> Module<T> {

	/// The main entry point, called with account we are supposed to sign with
	fn offchain(key: &T::AccountId) {
		for e in <Self as Store>::Ofc::get() {
			match e {
				OffchainCall::Ping(nonce) => {
					Self::ping(key, nonce)
				}
				// there would be potential other calls
			}
		}
	}

	fn ping(key: &T::AccountId, nonce: u8) {
		runtime_io::print_utf8(b"Received ping, sending pong");
		let call = Call::pong(nonce);
		let _ = T::SubmitTransaction::sign_and_submit(call, key.clone().into());
	}

	/// Helper that confirms whether the given `AccountId` can sign `pong` transactions
	fn is_authority(who: &T::AccountId) -> bool {
		Self::authorities().into_iter().find(|i| i == who).is_some()
	}

	/// Find a local `AccountId` we can sign with, that is allowed to `pong`
	fn authority_id() -> Option<T::AccountId> {
		// Find all local keys accessible to this app through the localised KeyType.
		// Then go through all keys currently stored on chain and check them against
		// the list of local keys until a match is found, otherwise return `None`.
		let local_keys = T::KeyType::all().iter().map(
				|i| (*i).clone().into()
			).collect::<Vec<T::AccountId>>();

		Self::authorities().into_iter().find_map(|authority| {
			if local_keys.contains(&authority) {
				Some(authority)
			} else {
				None
			}
		})
	}
}


/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::with_externalities;
	use primitives::{H256, Blake2Hasher};
	use support::{impl_outer_origin, impl_outer_dispatch, assert_ok, parameter_types};
	use system::offchain::TransactionSubmitter;
	use system;
	use sr_primitives::{traits::{BlakeTwo256, IdentityLookup, Verify}, AnySignature, testing::Header};
	use sr_primitives::generic::UncheckedExtrinsic as GenericUncheckedExtrinsic;
	use sr_primitives::weights::Weight;
	use sr_primitives::Perbill;

	type Index = u32;
	type Signature = AnySignature;
	type AccountId = <Signature as Verify>::Signer;
	type UncheckedExtrinsic =  GenericUncheckedExtrinsic<(), Call, (), ()>;

	mod lfs_crypto {
		use super::KEY_TYPE;
		use primitives::sr25519;
		app_crypto::app_crypto!(sr25519, KEY_TYPE);

		impl From<Signature> for super::Signature {
			fn from(a: Signature) -> Self {
				sr25519::Signature::from(a).into()
			}
		}
}

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	impl_outer_dispatch! {
		pub enum Call for Test where origin: Origin {
			lfs::LfsModule,
		}
	}

	type LfsAccount = lfs_crypto::Public;
	type SubmitTransaction = TransactionSubmitter<LfsAccount, Call, UncheckedExtrinsic>;

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
		type Call = Call;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = AccountId;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type WeightMultiplierUpdate = ();
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
	}
	impl super::Trait for Test {
		type Event = ();
		type Call = Call;
		type SubmitTransaction = SubmitTransaction;
		type KeyType = LfsAccount;
	}
	type LfsModule = Module<Test>;


	/// Lastly we also need to implement the CreateTransaction signer for the runtime
	impl system::offchain::CreateTransaction<Test, UncheckedExtrinsic> for Test {
		type Signature = Signature;

		fn create_transaction<F: system::offchain::Signer<AccountId, Self::Signature>>(
			call: Call,
			account: AccountId,
			index: Index,
		) -> Option<(Call, <UncheckedExtrinsic as sr_primitives::traits::Extrinsic>::SignaturePayload)> {
			None
		}
	}

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

	#[test]
	fn smoketest_of_ping() {
		with_externalities(&mut new_test_ext(), || {
			
			assert_ok!(LfsModule::ping(Origin::signed(1), 42));
			assert_ok!(LfsModule::ping(Origin::signed(1), 8));
			assert_eq!(<LfsModule as Store>::Ofc::get(), [OffchainCall::Ping(42), OffchainCall::Ping(8)]);
		});
	}
}
