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
		#[arg(short, long, default_value = "3")]
		threads: u32,
		/// Exclude files matching glob pattern (can be repeated)
		#[arg(long)]
		exclude: Vec<String>,
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
	Show {
		/// Path to settings file (default: platform config dir)
		#[arg(long)]
		config_file: Option<String>,
	},
	/// Edit settings in $EDITOR
	Edit {
		/// Path to settings file (default: platform config dir)
		#[arg(long)]
		config_file: Option<String>,
	},
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
		Commands::Download { args, dest, threads, exclude } => {
			let passwords = load_password_file(args.password_file.as_deref());
			commands::download::run(&args, &passwords, dest.as_deref(), threads, &exclude).await;
		}
		Commands::Config { action } => match action {
			ConfigAction::Show { config_file } => {
				commands::config::run_show(config_file.as_deref());
			}
			ConfigAction::Edit { config_file } => {
				if let Err(e) = commands::config::run_edit(config_file.as_deref()) {
					eprintln!("Error: {}", e);
					std::process::exit(1);
				}
			}
		},
	}
}
