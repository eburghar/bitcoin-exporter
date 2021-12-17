# bitcoin-exporter

Serve bitcoind core metrics under `/metrics` path.

Rewrite of [bitcoin-prometheus-exporter](https://github.com/jvstein/bitcoin-prometheus-exporter) in rust.

Use a forked [rust-bitcoincore-rpc](https://git.itsufficient.me/rust/bitcoincore-rpc)
with applied [pr157](https://github.com/rust-bitcoin/rust-bitcoincore-rpc/pull/157) and
[pr171](https://github.com/rust-bitcoin/rust-bitcoincore-rpc/pull/171) and missing rpc calls implemented.

```
bitcoin-exporter 0.4.0

Usage: bitcoin-exporter [-c <config>] [-v]

Export bitcoin core metrics to prometheus format

Options:
  -c, --config      configuration file
  -v, --verbose     more detailed output
  --help            display usage information
```

The configuration accepts the following keys. `host` and `bind` are optional. `user`, `password` and `host` represent
the bitcoind server rpc parameters.

```yaml
user: user
password: changeme
host: 'http://localhost:3222'
bind: '127.0.0.1:9898'
```
