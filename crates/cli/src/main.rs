mod commands;

use clap::{Parser, Subcommand};

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
        /// Path to .sfdl file
        file: String,
        /// Decryption password
        #[arg(short, long)]
        password: Option<String>,
        /// File with passwords to try (one per line)
        #[arg(long)]
        password_file: Option<String>,
    },
    /// List files in SFDL container
    List {
        /// Path to .sfdl file
        file: String,
        /// Decryption password
        #[arg(short, long)]
        password: Option<String>,
        /// File with passwords to try (one per line)
        #[arg(long)]
        password_file: Option<String>,
        /// Resolve bulk folders via FTP connection
        #[arg(short, long)]
        resolve: bool,
    },
    /// Download files from SFDL container
    Download {
        /// Path to .sfdl file
        file: String,
        /// Decryption password
        #[arg(short, long)]
        password: Option<String>,
        /// File with passwords to try (one per line)
        #[arg(long)]
        password_file: Option<String>,
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
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Info { file, password, password_file } => {
            let passwords = load_password_file(password_file.as_deref());
            commands::info::run(&file, password.as_deref(), &passwords);
        }
        Commands::List { file, password, password_file, resolve } => {
            let passwords = load_password_file(password_file.as_deref());
            commands::list::run(&file, password.as_deref(), &passwords, resolve).await;
        }
        Commands::Download { file, password, password_file, dest, threads, exclude } => {
            let passwords = load_password_file(password_file.as_deref());
            commands::download::run(&file, password.as_deref(), &passwords, dest.as_deref(), threads, &exclude).await;
        }
    }
}

fn load_password_file(path: Option<&str>) -> Vec<String> {
    let Some(path) = path else { return Vec::new() };
    match std::fs::read_to_string(path) {
        Ok(content) => content
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect(),
        Err(e) => {
            eprintln!("Warning: Cannot read password file '{}': {}", path, e);
            Vec::new()
        }
    }
}
