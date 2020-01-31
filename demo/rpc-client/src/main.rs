// Copyright 2019-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

#![warn(missing_docs)]

//! Example substrate RPC client code.
//!
//! This module shows how you can write a Rust RPC client that connects to a running
//! substrate node and use staticly typed RPC wrappers.

use frame_support::storage::generator::StorageMap;
use futures::Future;
use hyper::rt;
use jsonrpc_core_client::{transports::http, RpcChannel};
use lfs_demo_runtime::{
	avatars, BlockNumber as Number, Hash, Header, Runtime, Signature, SignedBlock, SignedExtra,
	SignedPayload, UncheckedExtrinsic, VERSION,
};
use parity_scale_codec::{Decode, Encode};
use sc_lfs::{lfs_id::LfsId, rpc::LfsClient};
use sc_rpc::{author::AuthorClient, chain::ChainClient, state::StateClient};
use sp_core::crypto::Pair;
use sp_keyring::AccountKeyring;
use sp_rpc::{list::ListOrValue, number::NumberOrHex};
use sp_runtime::{
	generic::Era,
	traits::{IdentifyAccount, Verify},
};
use sp_storage::{StorageData, StorageKey};

fn main() {
	env_logger::init();

	rt::run(rt::lazy(|| {
		let uri = "http://localhost:9933";
		let key = AccountKeyring::Alice;

		http::connect(uri)
			.and_then(|channel: RpcChannel| {
				// let's upload the image via RPC
				let client = LfsClient::<LfsId>::new(channel.clone());
				client
					.upload(include_bytes!("./avataaars.png").to_vec())
					.map(|r| (channel, r))
			})
			.map(move |(channel, remote_id)| {
				// get the current nonce via RPC
				let nonce_key = frame_system::AccountNonce::<Runtime>::storage_map_final_key(
					key.clone().to_account_id(),
				);
				let nonce = <Runtime as frame_system::Trait>::Index::decode(
					&mut &StateClient::<Hash>::new(channel.clone())
						.storage(StorageKey(nonce_key), None)
						.wait()
						.expect("RPC doesn't fail")
						.unwrap_or(StorageData(vec![0, 0, 0, 0]))
						.0[..],
				)
				.expect("Nonce is always an Index");

				let genesis_hash = {
					if let ListOrValue::Value(Some(h)) =
						ChainClient::<Number, Hash, Header, SignedBlock>::new(channel.clone())
							.block_hash(Some(ListOrValue::Value(NumberOrHex::Number(0))))
							.wait()
							.expect("Genesis Block exists")
					{
						h
					} else {
						panic!("No genesis hash found on remote chain!");
					}
				};

				(channel, remote_id, genesis_hash, nonce)
			})
			.map(move |(channel, remote_id, genesis_hash, nonce)| {
				// submit the reference ID as our avatar entry
				let call = avatars::Call::<Runtime>::request_to_change_avatar(remote_id.into());

				let tip = 0;
				let extra: SignedExtra = (
					frame_system::CheckVersion::<Runtime>::new(),
					frame_system::CheckGenesis::<Runtime>::new(),
					frame_system::CheckEra::<Runtime>::from(Era::Immortal),
					frame_system::CheckNonce::<Runtime>::from(nonce),
					frame_system::CheckWeight::<Runtime>::new(),
					pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
				);
				let raw_payload = SignedPayload::from_raw(
					call.into(),
					extra,
					(
						VERSION.spec_version,
						genesis_hash.clone(),
						genesis_hash,
						(),
						(),
						(),
					), // additional extras
				);
				let signature = raw_payload.using_encoded(|payload| key.pair().sign(payload));
				let (call, extra, _) = raw_payload.deconstruct();
				let account = <Signature as Verify>::Signer::from(key.public()).into_account();

				let extrinsic =
					UncheckedExtrinsic::new_signed(call, account.into(), signature.into(), extra);

				let client = AuthorClient::<Hash, Hash>::new(channel.clone());
				let _ = client
					.submit_extrinsic(extrinsic.encode().into())
					.wait()
					.map_err(|e| {
						println!("Error: {:?}", e);
					})
					.map(|_| {
						println!("Transaction submitted!");
					});
			})
			.map_err(|e| {
				println!("Error: {:?}", e);
			})
	}))
}
