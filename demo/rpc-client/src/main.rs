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
use futures::{future::join_all, Future};
use hyper::rt;
use jsonrpc_core_client::{transports::http, RpcChannel};
use lfs_demo_runtime::{
	BlockNumber as Number, Hash, Header, Runtime, Signature, SignedBlock, SignedExtra,
	SignedPayload, UncheckedExtrinsic, VERSION,
};
use pallet_lfs_user_data as user_data;
use pallet_sudo as sudo;
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

use std::path::PathBuf;
use structopt::StructOpt;

fn parse_key(s: &str) -> Result<AccountKeyring, String> {
	match s {
		"alice" | "Alice" => Ok(AccountKeyring::Alice),
		"bob" | "Bob" => Ok(AccountKeyring::Bob),
		"charlie" | "Charlie" => Ok(AccountKeyring::Charlie),
		_ => Err(format!("Unknown key {:}", s)),
	}
}

#[derive(Debug, StructOpt)]
#[structopt(
	name = "lfs-demo-rpc-client",
	about = "Let's submit some user data to our chain"
)]
struct Opt {
	/// RPC Server to use
	#[structopt(long, default_value = "http://localhost:9933")]
	server: String,

	/// Use the this Key
	#[structopt(short = "k", long = "key", parse(try_from_str = parse_key), default_value = "Alice")]
	key: AccountKeyring,

	/// Set as root, not for the given account
	#[structopt(long)]
	root: bool,

	/// Store under `name` rather than the name of the file
	#[structopt(long)]
	name: Option<String>,

	/// Input file or folder
	#[structopt(name = "FILE")]
	inputs: PathBuf,
}

fn main() {
	env_logger::init();
	let m = Opt::from_args();

	let uri = m.server;
	let key = m.key;
	let root = m.root;

	let files = {
		let item = m.inputs;
		let name = m
			.name
			.map(|s| s.as_str().to_owned())
			.or_else(|| {
				if let Some(Some(s)) = item.as_path().file_name().map(|s| s.to_str()) {
					return Some(s.to_owned());
				}
				return None;
			})
			.expect("Not a proper file.");
		vec![(name, item)]
	};

	rt::run(rt::lazy(move || {
		http::connect(&uri)
			.and_then(move |channel: RpcChannel| {
				// let's upload the image via RPC
				let client = LfsClient::<LfsId>::new(channel.clone());
				join_all(files.into_iter().map(move |(name, path)| {
					client
						.upload(std::fs::read(path.clone()).expect("Could not read file "))
						.map(move |r| {
							println!("File {:?} uploaded via RPC: {:}", path, r);
							(name, r)
						})
				}))
				.map(|v: Vec<(String, LfsId)>| (channel, v))
			})
			.map(move |(channel, to_set)| {
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

				(channel, to_set, genesis_hash, nonce)
			})
			.map(move |(channel, to_set, genesis_hash, nonce)| {
				// submit the reference ID as our avatar entry
				let mut running_nonce = nonce;
				let mut calls = vec![];
				for (name, remote_id) in to_set {
					let tip = 0;
					let extra: SignedExtra = (
						frame_system::CheckVersion::<Runtime>::new(),
						frame_system::CheckGenesis::<Runtime>::new(),
						frame_system::CheckEra::<Runtime>::from(Era::Immortal),
						frame_system::CheckNonce::<Runtime>::from(running_nonce),
						frame_system::CheckWeight::<Runtime>::new(),
						pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
					);
					let raw_payload = SignedPayload::from_raw(
						if root {
							sudo::Call::<Runtime>::sudo(Box::new(
								user_data::Call::<Runtime>::root_update(
									name.as_bytes().to_vec(),
									remote_id.clone().into(),
								)
								.into(),
							))
							.into()
						} else {
							user_data::Call::<Runtime>::update(
								name.as_bytes().to_vec(),
								remote_id.clone().into(),
							)
							.into()
						},
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

					let extrinsic = UncheckedExtrinsic::new_signed(
						call,
						account.into(),
						signature.into(),
						extra,
					);

					let client = AuthorClient::<Hash, Hash>::new(channel.clone());
					let sub = client
						.submit_extrinsic(extrinsic.encode().into())
						.wait()
						.map_err(|e| {
							println!("Error: {:?}", e);
						})
						.map(move |hash| {
							if root {
								println!(
									"Submitted {:?} to root (as {}): {:} in {:}",
									name, key, remote_id, hash
								);
							} else {
								println!(
									"Submitted {:?} for {:}: {:} in {:}",
									name, key, remote_id, hash
								);
							}
						});
					calls.push(sub);
					running_nonce += 1;
				}
				join_all(calls)
			})
			.map(|_| {
				println!("------ All submitted");
			})
			.map_err(|e| {
				println!("Error: {:?}", e);
			})
	}))
}
