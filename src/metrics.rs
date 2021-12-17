use lazy_static::lazy_static;
use prometheus::{
	register_counter, register_counter_vec, register_gauge, register_gauge_vec, Counter,
	CounterVec, Gauge, GaugeVec,
};

lazy_static! {
	pub(crate) static ref BITCOIN_UPTIME: GaugeVec = register_gauge_vec!(
		"bitcoin_uptime",
		"Number of seconds the Bitcoin daemon has been running",
		&["version", "protocol", "chain"]
	)
	.unwrap();
	pub(crate) static ref BITCOIN_BLOCKS: Gauge = register_gauge!("bitcoin_blocks", "Block height").unwrap();
	pub(crate) static ref BITCOIN_DIFFICULTY: Gauge =
		register_gauge!("bitcoin_difficulty", "Difficulty").unwrap();
	pub(crate) static ref BITCOIN_PEERS: Gauge = register_gauge!("bitcoin_peers", "Number of peers").unwrap();
	pub(crate) static ref BITCOIN_CONN_IN: Gauge =
		register_gauge!("bitcoin_conn_in", "Number of connections in").unwrap();
	pub(crate) static ref BITCOIN_CONN_OUT: Gauge =
		register_gauge!("bitcoin_conn_out", "Number of connections out").unwrap();
	pub(crate) static ref BITCOIN_WARNINGS: Counter = register_counter!(
		"bitcoin_warnings",
		"Number of network or blockchain warnings detected"
	)
	.unwrap();
	pub(crate) static ref BITCOIN_MEMINFO_USED: Gauge =
		register_gauge!("bitcoin_meminfo_used", "Number of bytes used").unwrap();
	pub(crate) static ref BITCOIN_MEMINFO_FREE: Gauge =
		register_gauge!("bitcoin_meminfo_free", "Number of bytes available").unwrap();
	pub(crate) static ref BITCOIN_MEMINFO_TOTAL: Gauge =
		register_gauge!("bitcoin_meminfo_total", "Number of bytes managed").unwrap();
	pub(crate) static ref BITCOIN_MEMINFO_LOCKED: Gauge =
		register_gauge!("bitcoin_meminfo_locked", "Number of bytes locked").unwrap();
	pub(crate) static ref BITCOIN_MEMINFO_CHUNKS_USED: Gauge =
		register_gauge!("bitcoin_meminfo_chunks_used", "Number of allocated chunks").unwrap();
	pub(crate) static ref BITCOIN_MEMINFO_CHUNKS_FREE: Gauge =
		register_gauge!("bitcoin_meminfo_chunks_free", "Number of unused chunks").unwrap();
	pub(crate) static ref BITCOIN_MEMPOOL_BYTES: Gauge =
		register_gauge!("bitcoin_mempool_bytes", "Size of mempool in bytes").unwrap();
	pub(crate) static ref BITCOIN_MEMPOOL_SIZE: Gauge = register_gauge!(
		"bitcoin_mempool_size",
		"Number of unconfirmed transactions in mempool"
	)
	.unwrap();
	pub(crate) static ref BITCOIN_MEMPOOL_USAGE: Gauge = register_gauge!(
		"bitcoin_mempool_usage",
		"Total memory usage for the mempool"
	)
	.unwrap();
	pub(crate) static ref BITCOIN_MEMPOOL_UNBROADCAST: Gauge = register_gauge!(
		"bitcoin_mempool_unbroadcast",
		"Number of transactions waiting for acknowledgment"
	)
	.unwrap();
	pub(crate) static ref BITCOIN_LATEST_BLOCK_HEIGHT: Gauge = register_gauge!(
		"bitcoin_latest_block_height",
		"Height or index of latest block"
	)
	.unwrap();
	pub(crate) static ref BITCOIN_LATEST_BLOCK_WEIGHT: Gauge = register_gauge!(
		"bitcoin_latest_block_weight",
		"Weight of latest block according to BIP 141"
	)
	.unwrap();
	pub(crate) static ref BITCOIN_LATEST_BLOCK_SIZE: Gauge =
		register_gauge!("bitcoin_latest_block_size", "Size of latest block in bytes").unwrap();
	pub(crate) static ref BITCOIN_LATEST_BLOCK_TXS: Gauge = register_gauge!(
		"bitcoin_latest_block_txs",
		"Number of transactions in latest block"
	)
	.unwrap();
	pub(crate) static ref BITCOIN_TXCOUNT: Gauge =
		register_gauge!("bitcoin_txcount", "Number of TX since the genesis block").unwrap();
	pub(crate) static ref BITCOIN_NUM_CHAINTIPS: Gauge = register_gauge!(
		"bitcoin_num_chaintips",
		"Number of known blockchain branches"
	)
	.unwrap();
	pub(crate) static ref BITCOIN_TOTAL_BYTES_RECV: Gauge =
		register_gauge!("bitcoin_total_bytes_recv", "Total bytes received").unwrap();
	pub(crate) static ref BITCOIN_TOTAL_BYTES_SENT: Gauge =
		register_gauge!("bitcoin_total_bytes_sent", "Total bytes sent").unwrap();
	pub(crate) static ref BITCOIN_LATEST_BLOCK_INPUTS: Gauge = register_gauge!(
		"bitcoin_latest_block_inputs",
		"Number of inputs in transactions of latest block"
	)
	.unwrap();
	pub(crate) static ref BITCOIN_LATEST_BLOCK_OUTPUTS: Gauge = register_gauge!(
		"bitcoin_latest_block_outputs",
		"Number of outputs in transactions of latest block"
	)
	.unwrap();
	pub(crate) static ref BITCOIN_LATEST_BLOCK_VALUE: Gauge = register_gauge!(
		"bitcoin_latest_block_value",
		"Bitcoin value of all transactions in the latest block"
	)
	.unwrap();
	pub(crate) static ref BITCOIN_LATEST_BLOCK_FEE: Gauge = register_gauge!(
		"bitcoin_latest_block_fee",
		"Total fee to process the latest block"
	)
	.unwrap();
	pub(crate) static ref BITCOIN_BAN_CREATED: GaugeVec = register_gauge_vec!(
		"bitcoin_ban_created",
		"Time the ban was created",
		&["address", "reason"]
	)
	.unwrap();
	pub(crate) static ref BITCOIN_BANNED_UNTIL: GaugeVec = register_gauge_vec!(
		"bitcoin_banned_until",
		"Time the ban expires",
		&["address", "reason"]
	)
	.unwrap();
	pub(crate) static ref BITCOIN_SIZE_ON_DISK: Gauge = register_gauge!(
		"bitcoin_size_on_disk",
		"Estimated size of the block and undo files"
	)
	.unwrap();
	pub(crate) static ref BITCOIN_VERIFICATION_PROGRESS: Gauge = register_gauge!(
		"bitcoin_verification_progress",
		"Estimate of verification progress [0..1]"
	)
	.unwrap();
	pub(crate) static ref BITCOIN_RPC_ACTIVE: Gauge =
		register_gauge!("bitcoin_rpc_active", "Number of RPC calls being processed").unwrap();
	pub(crate) static ref BITCOIN_HASHPS_NEG1: Gauge = register_gauge!(
		"bitcoin_hashps_neg1",
		"Estimated network hash rate per second since the last difficulty change"
	)
	.unwrap();
	pub(crate) static ref BITCOIN_HASHPS_1: Gauge = register_gauge!(
		"bitcoin_hashps_1",
		"Estimated network hash rate per second for the last block"
	)
	.unwrap();
	pub(crate) static ref BITCOIN_HASHPS: Gauge = register_gauge!(
		"bitcoin_hashps",
		"Estimated network hash rate per second for the last 120 blocks"
	)
	.unwrap();
	pub(crate) static ref EXPORTER_ERRORS: CounterVec = register_counter_vec!(
		"bitcoin_exporter_errors",
		"Number of errors encountered by the exporter",
		&["type"]
	)
	.unwrap();
	pub(crate) static ref PROCESS_TIME: Counter = register_counter!(
		"bitcoin_exporter_process_time",
		"Time spent processing metrics from bitcoin node"
	)
	.unwrap();
	// static definition for now (TODO: use const expression)
	pub(crate) static ref SMART_FEE_2: Gauge = register_gauge!(
		"bitcoin_est_smart_fee_2",
		"Estimated smart fee per kilobyte for confirmation in 2 blocks"
	).unwrap();
	pub(crate) static ref SMART_FEE_3: Gauge = register_gauge!(
		"bitcoin_est_smart_fee_3",
		"Estimated smart fee per kilobyte for confirmation in 3 blocks"
	).unwrap();
	pub(crate) static ref SMART_FEE_5: Gauge = register_gauge!(
		"bitcoin_est_smart_fee_5",
		"Estimated smart fee per kilobyte for confirmation in 5 blocks"
	).unwrap();
	pub(crate) static ref SMART_FEE_20: Gauge = register_gauge!(
		"bitcoin_est_smart_fee_20",
		"Estimated smart fee per kilobyte for confirmation in 20 blocks"
	).unwrap();
}
