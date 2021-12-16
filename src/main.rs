mod args;
mod config;

use bitcoincore_rpc::{Auth, Client, RpcApi};
use bitcoincore_rpc_json::HashOrHeight;
use hyper::{
	header::CONTENT_TYPE,
	service::{make_service_fn, service_fn},
	Body, Request, Response, Server,
};
use lazy_static::lazy_static;
use prometheus::{
	register_counter, register_counter_vec, register_gauge, register_gauge_vec, Counter,
	CounterVec, Encoder, Gauge, GaugeVec, TextEncoder,
};
use std::sync::Arc;

use crate::{args::Args, config::Config};

// converted from https://github.com/jvstein/bitcoin-prometheus-exporter/blob/master/bitcoind-monitor.py

lazy_static! {
	static ref BITCOIN_UPTIME: GaugeVec = register_gauge_vec!(
		"bitcoin_uptime",
		"Number of seconds the Bitcoin daemon has been running",
		&["version", "protocol"]
	)
	.unwrap();
	static ref BITCOIN_BLOCKS: Gauge = register_gauge!("bitcoin_blocks", "Block height").unwrap();
	static ref BITCOIN_DIFFICULTY: Gauge =
		register_gauge!("bitcoin_difficulty", "Difficulty").unwrap();
	static ref BITCOIN_PEERS: Gauge = register_gauge!("bitcoin_peers", "Number of peers").unwrap();
	static ref BITCOIN_CONN_IN: Gauge =
		register_gauge!("bitcoin_conn_in", "Number of connections in").unwrap();
	static ref BITCOIN_CONN_OUT: Gauge =
		register_gauge!("bitcoin_conn_out", "Number of connections out").unwrap();
	static ref BITCOIN_WARNINGS: Counter = register_counter!(
		"bitcoin_warnings",
		"Number of network or blockchain warnings detected"
	)
	.unwrap();
	static ref BITCOIN_MEMINFO_USED: Gauge =
		register_gauge!("bitcoin_meminfo_used", "Number of bytes used").unwrap();
	static ref BITCOIN_MEMINFO_FREE: Gauge =
		register_gauge!("bitcoin_meminfo_free", "Number of bytes available").unwrap();
	static ref BITCOIN_MEMINFO_TOTAL: Gauge =
		register_gauge!("bitcoin_meminfo_total", "Number of bytes managed").unwrap();
	static ref BITCOIN_MEMINFO_LOCKED: Gauge =
		register_gauge!("bitcoin_meminfo_locked", "Number of bytes locked").unwrap();
	static ref BITCOIN_MEMINFO_CHUNKS_USED: Gauge =
		register_gauge!("bitcoin_meminfo_chunks_used", "Number of allocated chunks").unwrap();
	static ref BITCOIN_MEMINFO_CHUNKS_FREE: Gauge =
		register_gauge!("bitcoin_meminfo_chunks_free", "Number of unused chunks").unwrap();
	static ref BITCOIN_MEMPOOL_BYTES: Gauge =
		register_gauge!("bitcoin_mempool_bytes", "Size of mempool in bytes").unwrap();
	static ref BITCOIN_MEMPOOL_SIZE: Gauge = register_gauge!(
		"bitcoin_mempool_size",
		"Number of unconfirmed transactions in mempool"
	)
	.unwrap();
	static ref BITCOIN_MEMPOOL_USAGE: Gauge = register_gauge!(
		"bitcoin_mempool_usage",
		"Total memory usage for the mempool"
	)
	.unwrap();
	static ref BITCOIN_MEMPOOL_UNBROADCAST: Gauge = register_gauge!(
		"bitcoin_mempool_unbroadcast",
		"Number of transactions waiting for acknowledgment"
	)
	.unwrap();
	static ref BITCOIN_LATEST_BLOCK_HEIGHT: Gauge = register_gauge!(
		"bitcoin_latest_block_height",
		"Height or index of latest block"
	)
	.unwrap();
	static ref BITCOIN_LATEST_BLOCK_WEIGHT: Gauge = register_gauge!(
		"bitcoin_latest_block_weight",
		"Weight of latest block according to BIP 141"
	)
	.unwrap();
	static ref BITCOIN_LATEST_BLOCK_SIZE: Gauge =
		register_gauge!("bitcoin_latest_block_size", "Size of latest block in bytes").unwrap();
	static ref BITCOIN_LATEST_BLOCK_TXS: Gauge = register_gauge!(
		"bitcoin_latest_block_txs",
		"Number of transactions in latest block"
	)
	.unwrap();
	static ref BITCOIN_TXCOUNT: Gauge =
		register_gauge!("bitcoin_txcount", "Number of TX since the genesis block").unwrap();
	static ref BITCOIN_NUM_CHAINTIPS: Gauge = register_gauge!(
		"bitcoin_num_chaintips",
		"Number of known blockchain branches"
	)
	.unwrap();
	static ref BITCOIN_TOTAL_BYTES_RECV: Gauge =
		register_gauge!("bitcoin_total_bytes_recv", "Total bytes received").unwrap();
	static ref BITCOIN_TOTAL_BYTES_SENT: Gauge =
		register_gauge!("bitcoin_total_bytes_sent", "Total bytes sent").unwrap();
	static ref BITCOIN_LATEST_BLOCK_INPUTS: Gauge = register_gauge!(
		"bitcoin_latest_block_inputs",
		"Number of inputs in transactions of latest block"
	)
	.unwrap();
	static ref BITCOIN_LATEST_BLOCK_OUTPUTS: Gauge = register_gauge!(
		"bitcoin_latest_block_outputs",
		"Number of outputs in transactions of latest block"
	)
	.unwrap();
	static ref BITCOIN_LATEST_BLOCK_VALUE: Gauge = register_gauge!(
		"bitcoin_latest_block_value",
		"Bitcoin value of all transactions in the latest block"
	)
	.unwrap();
	static ref BITCOIN_LATEST_BLOCK_FEE: Gauge = register_gauge!(
		"bitcoin_latest_block_fee",
		"Total fee to process the latest block"
	)
	.unwrap();
	static ref BITCOIN_BAN_CREATED: GaugeVec = register_gauge_vec!(
		"bitcoin_ban_created",
		"Time the ban was created",
		&["address", "reason"]
	)
	.unwrap();
	static ref BITCOIN_BANNED_UNTIL: GaugeVec = register_gauge_vec!(
		"bitcoin_banned_until",
		"Time the ban expires",
		&["address", "reason"]
	)
	.unwrap();
	static ref BITCOIN_SIZE_ON_DISK: Gauge = register_gauge!(
		"bitcoin_size_on_disk",
		"Estimated size of the block and undo files"
	)
	.unwrap();
	static ref BITCOIN_VERIFICATION_PROGRESS: Gauge = register_gauge!(
		"bitcoin_verification_progress",
		"Estimate of verification progress [0..1]"
	)
	.unwrap();
	static ref BITCOIN_RPC_ACTIVE: Gauge =
		register_gauge!("bitcoin_rpc_active", "Number of RPC calls being processed").unwrap();
	static ref BITCOIN_HASHPS_NEG1: Gauge = register_gauge!(
		"bitcoin_hashps_neg1",
		"Estimated network hash rate per second since the last difficulty change"
	)
	.unwrap();
	static ref BITCOIN_HASHPS_1: Gauge = register_gauge!(
		"bitcoin_hashps_1",
		"Estimated network hash rate per second for the last block"
	)
	.unwrap();
	static ref BITCOIN_HASHPS: Gauge = register_gauge!(
		"bitcoin_hashps",
		"Estimated network hash rate per second for the last 120 blocks"
	)
	.unwrap();
	static ref EXPORTER_ERRORS: CounterVec = register_counter_vec!(
		"bitcoin_exporter_errors",
		"Number of errors encountered by the exporter",
		&["type"]
	)
	.unwrap();
	static ref PROCESS_TIME: Counter = register_counter!(
		"bitcoin_exporter_process_time",
		"Time spent processing metrics from bitcoin node"
	)
	.unwrap();
}

// # Create Prometheus metrics to track bitcoind stats.

// BITCOIN_ESTIMATED_SMART_FEE_GAUGES = {}  # type: Dict[int, Gauge]

async fn serve_req(_req: Request<Body>, rpc: Arc<Client>) -> Result<Response<Body>, hyper::Error> {
	let encoder = TextEncoder::new();

	if let Ok(blockchaininfo) = rpc.get_blockchain_info() {
		BITCOIN_BLOCKS.set(blockchaininfo.blocks as f64);
		BITCOIN_DIFFICULTY.set(blockchaininfo.difficulty as f64);
		BITCOIN_SIZE_ON_DISK.set(blockchaininfo.size_on_disk as f64);
		BITCOIN_VERIFICATION_PROGRESS.set(blockchaininfo.verification_progress as f64);

		if let Ok(latest_blockstats) = rpc.get_block_stats2(
			HashOrHeight::Hash(blockchaininfo.best_block_hash),
			Some(vec![
				"total_size",
				"total_weight",
				"total_fee",
				"txs",
				"height",
				"ins",
				"outs",
				"total_out",
			]),
		) {
			BITCOIN_LATEST_BLOCK_SIZE.set(latest_blockstats.total_size as f64);
			BITCOIN_LATEST_BLOCK_TXS.set(latest_blockstats.txs as f64);
			BITCOIN_LATEST_BLOCK_HEIGHT.set(latest_blockstats.height as f64);
			BITCOIN_LATEST_BLOCK_WEIGHT.set(latest_blockstats.total_weight as f64);
			BITCOIN_LATEST_BLOCK_INPUTS.set(latest_blockstats.ins as f64);
			BITCOIN_LATEST_BLOCK_OUTPUTS.set(latest_blockstats.outs as f64);
			BITCOIN_LATEST_BLOCK_VALUE.set(latest_blockstats.total_out.as_sat() as f64);
			BITCOIN_LATEST_BLOCK_FEE.set(latest_blockstats.total_fee.as_sat() as f64);
		}
	}

	if let Ok(networkinfo) = rpc.get_network_info() {
		if let Ok(uptime) = rpc.uptime() {
			BITCOIN_UPTIME
				.with_label_values(&[
					&networkinfo.version.to_string(),
					&networkinfo.protocol_version.to_string(),
				])
				.set(uptime as f64);
		}
		BITCOIN_PEERS.set(networkinfo.connections as f64);
		if let Some(connections_in) = networkinfo.connections_in {
			BITCOIN_CONN_IN.set(connections_in as f64);
		}
		if let Some(connections_out) = networkinfo.connections_out {
			BITCOIN_CONN_OUT.set(connections_out as f64);
		}
		if !networkinfo.warnings.is_empty() {
			BITCOIN_WARNINGS.inc()
		}
	}

	if let Ok(hashps) = rpc.get_network_hash_ps(Some(120), None) {
		BITCOIN_HASHPS.set(hashps);
	}
	if let Ok(hashps) = rpc.get_network_hash_ps(Some(0), None) {
		BITCOIN_HASHPS_NEG1.set(hashps);
	}
	if let Ok(hashps) = rpc.get_network_hash_ps(Some(1), None) {
		BITCOIN_HASHPS_1.set(hashps);
	}

	if let Ok(banned) = rpc.list_banned() {
		for ban in banned.iter() {
			BITCOIN_BAN_CREATED
				.with_label_values(&[&ban.address, "manually added"])
				.set(ban.ban_created as f64);
			BITCOIN_BANNED_UNTIL
				.with_label_values(&[&ban.address, "manually added"])
				.set(ban.banned_until as f64);
		}
	}

	if let Ok(txstats) = rpc.get_chain_tx_stats(None, None) {
		BITCOIN_TXCOUNT.set(txstats.txcount as f64);
	}

	if let Ok(chaintips) = rpc.get_chain_tips() {
		BITCOIN_NUM_CHAINTIPS.set(chaintips.len() as f64);
	}

	if let Ok(meminfo) = rpc.get_memory_info() {
		BITCOIN_MEMINFO_USED.set(meminfo.locked.used as f64);
		BITCOIN_MEMINFO_FREE.set(meminfo.locked.free as f64);
		BITCOIN_MEMINFO_TOTAL.set(meminfo.locked.total as f64);
		BITCOIN_MEMINFO_LOCKED.set(meminfo.locked.locked as f64);
		BITCOIN_MEMINFO_CHUNKS_USED.set(meminfo.locked.chunks_used as f64);
		BITCOIN_MEMINFO_CHUNKS_FREE.set(meminfo.locked.chunks_free as f64);
	}

	if let Ok(mempool) = rpc.get_mempool_info() {
		BITCOIN_MEMPOOL_BYTES.set(mempool.bytes as f64);
		BITCOIN_MEMPOOL_SIZE.set(mempool.size as f64);
		BITCOIN_MEMPOOL_USAGE.set(mempool.usage as f64);
		BITCOIN_MEMPOOL_UNBROADCAST.set(mempool.unbroadcastcount as f64);
	}

	if let Ok(netotals) = rpc.get_net_totals() {
		BITCOIN_TOTAL_BYTES_RECV.set(netotals.total_bytes_recv as f64);
		BITCOIN_TOTAL_BYTES_SENT.set(netotals.total_bytes_sent as f64);
	}

	let metric_families = prometheus::gather();

	let mut buffer = vec![];
	encoder.encode(&metric_families, &mut buffer).unwrap();

	let response = Response::builder()
		.status(200)
		.header(CONTENT_TYPE, encoder.format_type())
		.body(Body::from(buffer))
		.unwrap();

	Ok(response)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	// setup logging
	env_logger::init_from_env(
		env_logger::Env::new()
			.default_filter_or("bitcoin_exporter=info")
			.default_write_style_or("auto"),
	);
	log::info!("{} v{}", env!("CARGO_BIN_NAME"), env!("CARGO_PKG_VERSION"));

	// parse args
	let args: Args = args::from_env();
	// parse yaml config
	let config = Config::read(&args.config)?;

	let rpc = Client::new(&config.host, Auth::UserPass(config.user, config.password)).unwrap();
	let rpc = Arc::new(rpc);

	let addr = &config.bind.parse()?;
	log::info!("listening on http://{}", addr);

	let serve_future = make_service_fn(move |_| {
		let rpc = rpc.clone();
		async move {
			Ok::<_, hyper::Error>(service_fn(move |req| {
				let rpc = rpc.clone();
				serve_req(req, rpc)
			}))
		}
	});

	let server = Server::bind(&addr).serve(serve_future);
	if let Err(err) = server.await {
		log::error!("server error: {}", err);
	}
	Ok(())
}
