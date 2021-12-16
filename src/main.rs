mod args;

use bitcoincore_rpc::{Auth, Client, RpcApi};
use hyper::{
	header::CONTENT_TYPE,
	service::{make_service_fn, service_fn},
	Body, Request, Response, Server,
};
use lazy_static::lazy_static;
use prometheus::{
	labels, opts, register_counter, register_gauge, Counter, Encoder, Gauge, TextEncoder,
};
use std::sync::Arc;

use crate::args::Args;

// taken from https://github.com/jvstein/bitcoin-prometheus-exporter/blob/master/bitcoind-monitor.py

lazy_static! {
	static ref BITCOIN_UPTIME: Gauge = register_gauge!(
		"bitcoin_uptime",
		"Number of seconds the Bitcoin daemon has been running"
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
	static ref BITCOIN_BAN_CREATED: Gauge = register_gauge!(opts!(
		"bitcoin_ban_created",
		"Time the ban was created",
		labels! {"address" => "", "reason" => "",}
	))
	.unwrap();
	static ref BITCOIN_BANNED_UNTIL: Gauge = register_gauge!(opts!(
		"bitcoin_banned_until",
		"Time the ban expires",
		labels! {"address" => "", "reason" => ""}
	))
	.unwrap();
	static ref BITCOIN_SERVER_VERSION: Gauge =
		register_gauge!("bitcoin_server_version", "The server version").unwrap();
	static ref BITCOIN_PROTOCOL_VERSION: Gauge = register_gauge!(
		"bitcoin_protocol_version",
		"The protocol version of the server"
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
	static ref EXPORTER_ERRORS: Counter = register_counter!(opts!(
		"bitcoin_exporter_errors",
		"Number of errors encountered by the exporter",
		labels! {"type" => "", }
	))
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

	let uptime = rpc.uptime().unwrap();
	BITCOIN_UPTIME.set(uptime as f64);
	let blockchaininfo = rpc.get_blockchain_info().unwrap();
	BITCOIN_BLOCKS.set(blockchaininfo.blocks as f64);
	let networkinfo = rpc.get_network_info().unwrap();
	BITCOIN_PEERS.set(networkinfo.connections as f64);
	if let Some(connections_in) = networkinfo.connections_in {
		BITCOIN_CONN_IN.set(connections_in as f64);
	}
	if let Some(connections_out) = networkinfo.connections_out {
		BITCOIN_CONN_OUT.set(connections_out as f64);
	}
	BITCOIN_DIFFICULTY.set(blockchaininfo.difficulty as f64);
	let hashps = rpc.get_network_hash_ps(Some(120), None).unwrap();
	BITCOIN_HASHPS.set(hashps);
	let hashps = rpc.get_network_hash_ps(Some(0), None).unwrap();
	BITCOIN_HASHPS_NEG1.set(hashps);
	let hashps = rpc.get_network_hash_ps(Some(1), None).unwrap();
	BITCOIN_HASHPS_1.set(hashps);
	BITCOIN_SERVER_VERSION.set(networkinfo.version as f64);
	BITCOIN_PROTOCOL_VERSION.set(networkinfo.protocol_version as f64);
	BITCOIN_SIZE_ON_DISK.set(blockchaininfo.size_on_disk as f64);
	BITCOIN_VERIFICATION_PROGRESS.set(blockchaininfo.verification_progress as f64);
	if !networkinfo.warnings.is_empty() {
		BITCOIN_WARNINGS.inc()
	}
	let chaintips = rpc.get_chain_tips().unwrap();
	BITCOIN_NUM_CHAINTIPS.set(chaintips.len() as f64);
	let netotals = rpc.get_net_totals().unwrap();
	BITCOIN_TOTAL_BYTES_RECV.set(netotals.total_bytes_recv as f64);
	BITCOIN_TOTAL_BYTES_SENT.set(netotals.total_bytes_sent as f64);

	let metric_families = prometheus::gather();

	let mut buffer = vec![];
	encoder.encode(&metric_families, &mut buffer).unwrap();
	// HTTP_BODY_GAUGE.set(buffer.len() as f64);

	let response = Response::builder()
		.status(200)
		.header(CONTENT_TYPE, encoder.format_type())
		.body(Body::from(buffer))
		.unwrap();

	Ok(response)
}

#[tokio::main]
async fn main() {
	// setup logging
	env_logger::init_from_env(
		env_logger::Env::new()
			.default_filter_or("bitcoin_exporter=info")
			.default_write_style_or("auto"),
	);
	log::info!("{} v{}", env!("CARGO_BIN_NAME"), env!("CARGO_PKG_VERSION"));

	let args: Args = args::from_env();
	let rpc = Client::new(&args.host, Auth::UserPass(args.user, args.password)).unwrap();
	let rpc = Arc::new(rpc);

	let addr = ([127, 0, 0, 1], 9898).into();
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
		eprintln!("server error: {}", err);
	}
}
