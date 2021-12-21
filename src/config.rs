use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs::File;

fn default_host() -> String {
	"http://127.0.0.1:8332".to_owned()
}

fn default_bind() -> String {
    "127.0.0.1:9898".to_owned()
}

/// Config file
#[derive(Deserialize, Clone)]
pub struct Config {
	/// bitcoin rpc host
	#[serde(default = "default_host")]
	pub host: String,
	/// rpc user
	pub user: String,
	/// rpc password
	pub password: String,
	/// bind to addr:port
	#[serde(default = "default_bind")]
	pub bind: String
}

impl Config {
	pub fn read(config: &str) -> Result<Config> {
		// open configuration file
		let file = File::open(&config).with_context(|| format!("Can't open {}", &config))?;
		// deserialize configuration
		let config: Config =
			serde_yaml::from_reader(file).with_context(|| format!("Can't read {}", &config))?;
		Ok(config)
	}
}
