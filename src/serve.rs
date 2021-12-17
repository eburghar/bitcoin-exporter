use bitcoincore_rpc::{Client, RpcApi};
use bitcoincore_rpc_json::HashOrHeight;
use hyper::{header::CONTENT_TYPE, Body, Method, Request, Response};
use prometheus::{Encoder, TextEncoder};
use std::{net::SocketAddr, sync::Arc};

use crate::metrics::{
	BITCOIN_BANNED_UNTIL, BITCOIN_BAN_CREATED, BITCOIN_BLOCKS, BITCOIN_CONN_IN, BITCOIN_CONN_OUT,
	BITCOIN_DIFFICULTY, BITCOIN_HASHPS, BITCOIN_HASHPS_1, BITCOIN_HASHPS_NEG1,
	BITCOIN_LATEST_BLOCK_FEE, BITCOIN_LATEST_BLOCK_HEIGHT, BITCOIN_LATEST_BLOCK_INPUTS,
	BITCOIN_LATEST_BLOCK_OUTPUTS, BITCOIN_LATEST_BLOCK_SIZE, BITCOIN_LATEST_BLOCK_TXS,
	BITCOIN_LATEST_BLOCK_VALUE, BITCOIN_LATEST_BLOCK_WEIGHT, BITCOIN_MEMINFO_CHUNKS_FREE,
	BITCOIN_MEMINFO_CHUNKS_USED, BITCOIN_MEMINFO_FREE, BITCOIN_MEMINFO_LOCKED,
	BITCOIN_MEMINFO_TOTAL, BITCOIN_MEMINFO_USED, BITCOIN_MEMPOOL_BYTES, BITCOIN_MEMPOOL_SIZE,
	BITCOIN_MEMPOOL_UNBROADCAST, BITCOIN_MEMPOOL_USAGE, BITCOIN_NUM_CHAINTIPS, BITCOIN_PEERS,
	BITCOIN_SIZE_ON_DISK, BITCOIN_TOTAL_BYTES_RECV, BITCOIN_TOTAL_BYTES_SENT, BITCOIN_TXCOUNT,
	BITCOIN_UPTIME, BITCOIN_VERIFICATION_PROGRESS, BITCOIN_WARNINGS, SMART_FEE_2, SMART_FEE_20,
	SMART_FEE_3, SMART_FEE_5,
};

/// Create Prometheus metrics to track bitcoind stats.
pub(crate) async fn serve_req(
	req: Request<Body>,
	addr: SocketAddr,
	rpc: Arc<Client>,
) -> Result<Response<Body>, hyper::Error> {
	if req.method() != Method::GET || req.uri().path() != "/metrics" {
		log::warn!("  [{}] {} {}", addr, req.method(), req.uri().path());
		return Ok(Response::default());
	}

	let encoder = TextEncoder::new();

	// TODO: use async tasks to do rpc calls in //
	if let Ok(networkinfo) = rpc.get_network_info() {
		if let Ok(blockchaininfo) = rpc.get_blockchain_info() {
			// uptime came with version, protocol and chain label
			if let Ok(uptime) = rpc.uptime() {
				BITCOIN_UPTIME
					.with_label_values(&[
						&networkinfo.version.to_string(),
						&networkinfo.protocol_version.to_string(),
						&blockchaininfo.chain,
					])
					.set(uptime as f64);
			}

			BITCOIN_BLOCKS.set(blockchaininfo.blocks as f64);
			BITCOIN_DIFFICULTY.set(blockchaininfo.difficulty as f64);
			BITCOIN_SIZE_ON_DISK.set(blockchaininfo.size_on_disk as f64);
			BITCOIN_VERIFICATION_PROGRESS.set(blockchaininfo.verification_progress as f64);

			if let Ok(latest_blockstats) =
				rpc.get_block_stats2(HashOrHeight::Hash(blockchaininfo.best_block_hash), None)
			{
				BITCOIN_LATEST_BLOCK_SIZE.set(latest_blockstats.total_size as f64);
				BITCOIN_LATEST_BLOCK_TXS.set(latest_blockstats.txs as f64);
				BITCOIN_LATEST_BLOCK_HEIGHT.set(latest_blockstats.height as f64);
				BITCOIN_LATEST_BLOCK_WEIGHT.set(latest_blockstats.total_weight as f64);
				BITCOIN_LATEST_BLOCK_INPUTS.set(latest_blockstats.ins as f64);
				BITCOIN_LATEST_BLOCK_OUTPUTS.set(latest_blockstats.outs as f64);
				BITCOIN_LATEST_BLOCK_VALUE.set(latest_blockstats.total_out.as_btc() as f64);
				BITCOIN_LATEST_BLOCK_FEE.set(latest_blockstats.total_fee.as_btc() as f64);
			}
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

	if let Ok(smartfee) = rpc.estimate_smart_fee(2, None) {
		if let Some(fee_rate) = smartfee.fee_rate {
			SMART_FEE_2.set(fee_rate.as_sat() as f64)
		}
	}
	if let Ok(smartfee) = rpc.estimate_smart_fee(3, None) {
		if let Some(fee_rate) = smartfee.fee_rate {
			SMART_FEE_3.set(fee_rate.as_sat() as f64)
		}
	}
	if let Ok(smartfee) = rpc.estimate_smart_fee(5, None) {
		if let Some(fee_rate) = smartfee.fee_rate {
			SMART_FEE_5.set(fee_rate.as_sat() as f64)
		}
	}
	if let Ok(smartfee) = rpc.estimate_smart_fee(20, None) {
		if let Some(fee_rate) = smartfee.fee_rate {
			SMART_FEE_20.set(fee_rate.as_sat() as f64)
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
