#![cfg_attr(not(feature = "std"), no_std)]

use app_crypto::KeyTypeId;
use codec::{Decode, Encode};
/// A runtime module for SRML
use rstd::prelude::*;
use sp_runtime::traits::StaticLookup;
use support::{decl_event, decl_module, decl_storage, dispatch::DispatchResult, StorageValue};
use system::offchain::SubmitSignedTransaction;
use system::{ensure_root, ensure_signed};

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

	/// A dispatchable call type. We need to define it for the offchain worker to
	/// reference the `pong` function it wants to call.
	type Call: From<Call<Self>>;

	/// Let's define the helper we use to create signed transactions with
	type SubmitTransaction: SubmitSignedTransaction<Self, <Self as Trait>::Call>;
}

#[derive(Encode, Decode, Debug, PartialEq)]
/// Calls triggered to the offchain worker
enum OffchainCall {
	Ping(u8), // -> Expected to call back Pong(u8)
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
		pub fn send_ping(origin, nonce: u8) -> DispatchResult {
			// Ensure we are charging the sender
			let _who = ensure_signed(origin)?;

			// Informing the offchain worker
			<Self as Store>::Ofc::mutate(|v| v.push(OffchainCall::Ping(nonce)));

			Ok(())
		}

		// The pong from the offchain worker
		pub fn pong(origin, nonce: u8) -> DispatchResult {
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
			if T::SubmitTransaction::can_sign() {
				// FIXME: reenable once https://github.com/paritytech/substrate/pull/4200 is merged
				sp_io::misc::print_utf8(b"trigger offchain");
				let _ = Self::offchain();
			} else {
				sp_io::misc::print_utf8(b"Not authority");
			}
		}

		// Simple authority management: add a new authority to the set of keys that
		// are allowed to respond with `pong`.
		pub fn add_authority(origin, who: <T::Lookup as StaticLookup>::Source ) -> DispatchResult {
			// In practice this should be a bit cleverer, but for this example it is enough
			// that this is protected by a root-call (e.g. through governance like `sudo`).
			let _me = ensure_root(origin)?;
			let account = T::Lookup::lookup(who)?;

			if !Self::is_authority(&account){
				<Authorities<T>>::mutate(|l| l.push(account));
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
		/// Triggered on a pong with the corresponding value
		Ack(u8, AccountId),
	}
);

// We've moved the  helper functions outside of the main decleration for briefety.
impl<T: Trait> Module<T> {
	/// The main entry point, called with account we are supposed to sign with
	fn offchain() {
		for e in <Self as Store>::Ofc::get() {
			match e {
				OffchainCall::Ping(nonce) => {
					sp_io::misc::print_utf8(b"Received ping, sending pong");
					let call = Call::pong(nonce);
					let _ = T::SubmitTransaction::submit_signed(call);
				} // there would be potential other calls
			}
		}
	}

	/// Helper that confirms whether the given `AccountId` can sign `pong` transactions
	fn is_authority(who: &T::AccountId) -> bool {
		Self::authorities().into_iter().find(|i| i == who).is_some()
	}
}
