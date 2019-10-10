//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

use std::sync::Arc;
use std::time::Duration;
use futures::prelude::*;
use substrate_client::LongestChain;
use substrate_consensus_babe::{import_queue, start_babe, Config};
use substrate_finality_grandpa::{self, FinalityProofProvider as GrandpaFinalityProofProvider};
use substrate_lfs_demo_runtime::{self, GenesisConfig, opaque::Block, RuntimeApi, lfs_crypto};
use substrate_service::{error::{Error as ServiceError}, AbstractService, Configuration, ServiceBuilder};
use substrate_transaction_pool::{self, txpool::{Pool as TransactionPool}};
use substrate_inherents::InherentDataProviders;
use substrate_network::construct_simple_protocol;
use substrate_executor::native_executor_instance;
pub use substrate_executor::NativeExecutor;

// Our native executor instance.
native_executor_instance!(
	pub Executor,
	substrate_lfs_demo_runtime::api::dispatch,
	substrate_lfs_demo_runtime::native_version,
);

construct_simple_protocol! {
	/// Demo protocol attachment for substrate.
	pub struct NodeProtocol where Block = Block { }
}

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
macro_rules! new_full_start {
	($config:expr) => {{
		let mut import_setup = None;
		let inherent_data_providers = substrate_inherents::InherentDataProviders::new();
		let mut tasks_to_spawn = None;

		let builder = substrate_service::ServiceBuilder::new_full::<
			substrate_lfs_demo_runtime::opaque::Block, substrate_lfs_demo_runtime::RuntimeApi, crate::service::Executor
		>($config)?
			.with_select_chain(|_config, backend| {
				Ok(substrate_client::LongestChain::new(backend.clone()))
			})?
			.with_transaction_pool(|config, client|
				Ok(substrate_transaction_pool::txpool::Pool::new(config, substrate_transaction_pool::ChainApi::new(client)))
			)?
			.with_import_queue(|_config, client, mut select_chain, transaction_pool| {
				let select_chain = select_chain.take()
					.ok_or_else(|| substrate_service::Error::SelectChainRequired)?;
				let (block_import, link_half) =
					substrate_finality_grandpa::block_import::<_, _, _, substrate_lfs_demo_runtime::RuntimeApi, _, _>(
						client.clone(), client.clone(), select_chain
					)?;
				let justification_import = block_import.clone();

				let (import_queue, babe_link, babe_block_import, pruning_task) = substrate_consensus_babe::import_queue(
					substrate_consensus_babe::Config::get_or_compute(&*client)?,
					block_import,
					Some(Box::new(justification_import)),
					None,
					client.clone(),
					client,
					inherent_data_providers.clone(),
					Some(transaction_pool)
				)?;

				import_setup = Some((babe_block_import.clone(), link_half, babe_link));
				tasks_to_spawn = Some(vec![Box::new(pruning_task)]);

				Ok(import_queue)
			})?;

		(builder, import_setup, inherent_data_providers, tasks_to_spawn)
	}}
}

/// Builds a new service for a full client.
pub fn new_full<C: Send + Default + 'static>(config: Configuration<C, GenesisConfig>)
	-> Result<impl AbstractService, ServiceError>
{
	let is_authority = config.roles.is_authority();
	let name = config.name.clone();
	let disable_grandpa = config.disable_grandpa;
	let force_authoring = config.force_authoring;
	let dev_seed = config.dev_key_seed.clone();

	let (builder, mut import_setup, inherent_data_providers, mut tasks_to_spawn) = new_full_start!(config);

	let service = builder.with_network_protocol(|_| Ok(NodeProtocol::new()))?
		.with_finality_proof_provider(|client, backend|
			Ok(Arc::new(GrandpaFinalityProofProvider::new(backend, client)) as _)
		)?
		.build()?;

	let (block_import, link_half, babe_link) =
		import_setup.take()
			.expect("Link Half and Block Import are present for Full Services or setup failed before. qed");

	// spawn any futures that were created in the previous setup steps
	if let Some(tasks) = tasks_to_spawn.take() {
		for task in tasks {
			service.spawn_task(
				task.select(service.on_exit())
					.map(|_| ())
					.map_err(|_| ())
			);
		}
	}

	if let Some(seed) = dev_seed {
		service
			.keystore()
			.write()
			.insert_ephemeral_from_seed_by_type::<lfs_crypto::Pair>(
				&seed,
				lfs_crypto::KEY_TYPE,
			)
			.expect("Dev Seed always succeeds");
	}

	if is_authority {
		let proposer = substrate_basic_authorship::ProposerFactory {
			client: service.client(),
			transaction_pool: service.transaction_pool(),
		};

		let client = service.client();
		let select_chain = service.select_chain()
			.ok_or(ServiceError::SelectChainRequired)?;

		let babe_config = substrate_consensus_babe::BabeParams {
			config: Config::get_or_compute(&*client)?,
			keystore: service.keystore(),
			client,
			select_chain,
			block_import,
			env: proposer,
			sync_oracle: service.network(),
			inherent_data_providers: inherent_data_providers.clone(),
			force_authoring: force_authoring,
			time_source: babe_link,
		};

		let babe = start_babe(babe_config)?;
		let select = babe.select(service.on_exit()).then(|_| Ok(()));

		// the BABE authoring task is considered infallible, i.e. if it
		// fails we take down the service with it.
		service.spawn_essential_task(select);
	}

	let grandpa_config = substrate_finality_grandpa::Config {
		// FIXME #1578 make this available through chainspec
		gossip_duration: Duration::from_millis(333),
		justification_period: 4096,
		name: Some(name),
		keystore: Some(service.keystore()),
	};

	match (is_authority, disable_grandpa) {
		(false, false) => {
			// start the lightweight GRANDPA observer
			service.spawn_task(Box::new(substrate_finality_grandpa::run_grandpa_observer(
				grandpa_config,
				link_half,
				service.network(),
				service.on_exit(),
			)?));
		},
		(true, false) => {
			// start the full GRANDPA voter
			let voter_config = substrate_finality_grandpa::GrandpaParams {
				config: grandpa_config,
				link: link_half,
				network: service.network(),
				inherent_data_providers: inherent_data_providers.clone(),
				on_exit: service.on_exit(),
				telemetry_on_connect: Some(service.telemetry_on_connect_stream()),
			};

			// the GRANDPA voter task is considered infallible, i.e.
			// if it fails we take down the service with it.
			service.spawn_essential_task(substrate_finality_grandpa::run_grandpa_voter(voter_config)?);
		},
		(_, true) => {
			substrate_finality_grandpa::setup_disabled_grandpa(
				service.client(),
				&inherent_data_providers,
				service.network(),
			)?;
		},
	}

	Ok(service)
}

/// Builds a new service for a light client.
pub fn new_light<C: Send + Default + 'static>(config: Configuration<C, GenesisConfig>)
	-> Result<impl AbstractService, ServiceError>
{
	let inherent_data_providers = InherentDataProviders::new();

	ServiceBuilder::new_light::<Block, RuntimeApi, Executor>(config)?
		.with_select_chain(|_config, backend| {
			Ok(LongestChain::new(backend.clone()))
		})?
		.with_transaction_pool(|config, client|
			Ok(TransactionPool::new(config, substrate_transaction_pool::ChainApi::new(client)))
		)?
		.with_import_queue_and_fprb(|_config, client, backend, fetcher, _select_chain, transaction_pool| {
			let fetch_checker = fetcher
				.map(|fetcher| fetcher.checker().clone())
				.ok_or_else(|| "Trying to start light import queue without active fetch checker")?;
			let block_import = substrate_finality_grandpa::light_block_import::<_, _, _, RuntimeApi, _>(
				client.clone(), backend, Arc::new(fetch_checker), client.clone()
			)?;

			let finality_proof_import = block_import.clone();
			let finality_proof_request_builder =
				finality_proof_import.create_finality_proof_request_builder();

			// FIXME: pruning task isn't started since light client doesn't do `AuthoritySetup`.
			let (import_queue, ..) = import_queue(
				Config::get_or_compute(&*client)?,
				block_import,
				None,
				Some(Box::new(finality_proof_import)),
				client.clone(),
				client,
				inherent_data_providers.clone(),
				Some(transaction_pool)
			)?;

			Ok((import_queue, finality_proof_request_builder))
		})?
		.with_network_protocol(|_| Ok(NodeProtocol::new()))?
		.with_finality_proof_provider(|client, backend|
			Ok(Arc::new(GrandpaFinalityProofProvider::new(backend, client)) as _)
		)?
		.build()
}