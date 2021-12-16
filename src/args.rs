use argh::{FromArgs, TopLevelCommand};
use std::{env, path::Path};

/// Get vault secrets from path expressions, define environment variables, then execute into args and command
#[derive(FromArgs)]
pub struct Args {
	/// bitcoin host and port (http://localhost:8332)
	#[argh(option, short = 'h', default = "\"http://localhost:8332\".to_owned()")]
	pub host: String,

	/// rpc user
	#[argh(option, short = 'u')]
	pub user: String,

	/// rpc password
	#[argh(option, short = 'p')]
	pub password: String,
}

/// copy of argh::from_env to insert command name and version
pub fn from_env<T: TopLevelCommand>() -> T {
	let args: Vec<String> = std::env::args().collect();
	// get the file name of path or the full path
	let cmd = Path::new(&args[0])
		.file_name()
		.and_then(|s| s.to_str())
		.unwrap_or(&args[0]);
	let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
	T::from_args(&[cmd], &args_str[1..]).unwrap_or_else(|early_exit| {
		println!("{} {}\n", env!("CARGO_BIN_NAME"), env!("CARGO_PKG_VERSION"));
		println!("{}", early_exit.output);
		std::process::exit(match early_exit.status {
			Ok(()) => 0,
			Err(()) => 1,
		})
	})
}
