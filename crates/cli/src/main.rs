mod commands;

use clap::{Parser, Subcommand};
use commands::common::{SfdlArgs, load_password_file};

#[derive(Parser)]
#[command(name = "rsfdl", version, about = "SFDL file downloader")]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand)]
enum Commands {
	/// Display SFDL container info
	Info {
		#[command(flatten)]
		args: SfdlArgs,
		/// Output as JSON
		#[arg(long)]
		json: bool,
	},
	/// List files in SFDL container
	List {
		#[command(flatten)]
		args: SfdlArgs,
		/// Resolve bulk folders via FTP connection
		#[arg(short, long)]
		resolve: bool,
		/// Output as JSON
		#[arg(long)]
		json: bool,
		/// Exclude files matching glob pattern (can be repeated)
		#[arg(long)]
		exclude: Vec<String>,
		/// Disable all exclusion patterns
		#[arg(long)]
		no_exclude: bool,
		/// Show excluded files (marked with [excluded])
		#[arg(long)]
		show_excluded: bool,
	},
	/// Download files from SFDL container
	Download {
		#[command(flatten)]
		args: SfdlArgs,
		/// Download destination directory
		#[arg(short, long)]
		dest: Option<String>,
		/// Max concurrent downloads
		#[arg(short, long)]
		threads: Option<u32>,
		/// Max download speed in KB/s (0 = unlimited)
		#[arg(long)]
		max_speed: Option<u32>,
		/// Max retry attempts per file
		#[arg(long)]
		retries: Option<u32>,
		/// Delay between retries in seconds
		#[arg(long)]
		retry_delay: Option<u32>,
		/// Abort if insufficient disk space
		#[arg(long)]
		strict_disk_check: bool,
		/// Exclude files matching glob pattern (can be repeated)
		#[arg(long)]
		exclude: Vec<String>,
		/// Disable all exclusion patterns
		#[arg(long)]
		no_exclude: bool,
		/// Suppress progress display
		#[arg(short, long)]
		quiet: bool,
	},
	/// Manage settings
	Config {
		#[command(subcommand)]
		action: ConfigAction,
	},
}

#[derive(Subcommand)]
enum ConfigAction {
	/// Show current settings
	Show,
	/// Edit settings in $EDITOR
	Edit,
	/// Print path to settings file
	Path,
}

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt()
		.with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
		.with_writer(std::io::stderr)
		.init();

	let cli = Cli::parse();

	match cli.command {
		Commands::Info { args, json } => {
			let passwords = load_password_file(args.password_file.as_deref());
			commands::info::run(&args, &passwords, json);
		}
		Commands::List {
			args,
			resolve,
			json,
			exclude,
			no_exclude,
			show_excluded,
		} => {
			let passwords = load_password_file(args.password_file.as_deref());
			commands::list::run(&args, &passwords, resolve, json, &exclude, no_exclude, show_excluded).await;
		}
		Commands::Download {
			args,
			dest,
			threads,
			max_speed,
			retries,
			retry_delay,
			strict_disk_check,
			exclude,
			no_exclude,
			quiet,
		} => {
			let passwords = load_password_file(args.password_file.as_deref());
			commands::download::run(
				&args, &passwords, dest.as_deref(), threads, max_speed, retries, retry_delay, strict_disk_check, &exclude, no_exclude, quiet,
			)
			.await;
		}
		Commands::Config { action } => match action {
			ConfigAction::Show => {
				commands::config::run_show();
			}
			ConfigAction::Edit => {
				if let Err(e) = commands::config::run_edit() {
					eprintln!("Error: {}", e);
					std::process::exit(1);
				}
			}
			ConfigAction::Path => {
				commands::config::run_path();
			}
		},
	}
}
