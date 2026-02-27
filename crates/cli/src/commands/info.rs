use rsfdl_core::sfdl::models::{FtpDataConnectionType, SfdlContainer, SslProtocol};

use rsfdl_core::format_bytes;

use super::common::{DecryptOutcome, SfdlArgs, load_and_decrypt};

pub fn run(args: &SfdlArgs, password_list: &[String]) {
    let (container, _settings, outcome) = match load_and_decrypt(args, password_list) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    let encrypted_label = match outcome {
        DecryptOutcome::WasPlaintext => "no",
        DecryptOutcome::Decrypted => "yes (decrypted)",
        DecryptOutcome::AutoDecrypted => "yes (auto-decrypted)",
    };

    print_info(&container, encrypted_label);
}

fn print_info(c: &SfdlContainer, encrypted_label: &str) {
    let ssl_label = match c.connection.ssl_protocol {
        SslProtocol::None => "Plain FTP",
        SslProtocol::Tls => "TLS 1.0",
        SslProtocol::Tls11 => "TLS 1.1",
        SslProtocol::Tls12 => "TLS 1.2",
        SslProtocol::Ssl2 => "SSL 2",
        SslProtocol::Ssl3 => "SSL 3",
    };

    let conn_label = match c.connection.data_connection_type {
        FtpDataConnectionType::Passive | FtpDataConnectionType::AutoPassive => "Passive",
        FtpDataConnectionType::Active => "Active",
        FtpDataConnectionType::ExtendedPassive => "ExtPassive",
    };

    let file_count: usize = c.packages.iter().map(|p| p.file_list.len()).sum();
    let bulk_count: usize = c.packages.iter().map(|p| p.bulk_folder_list.len()).sum();
    let total_bytes: u64 = c
        .packages
        .iter()
        .flat_map(|p| p.file_list.iter())
        .map(|f| f.file_size)
        .sum();

    println!("Container: {}", c.description);
    println!("Uploader:  {}", c.uploader);
    println!("Version:   {}", c.container_version);
    println!("Encrypted: {}", encrypted_label);
    println!(
        "Server:    {}:{} ({}, {})",
        c.connection.host, c.connection.port, ssl_label, conn_label
    );
    println!("Packages:  {}", c.packages.len());

    if file_count > 0 {
        println!("Files:     {}", file_count);
        println!("Total:     {}", format_bytes(total_bytes));
    }
    if bulk_count > 0 {
        println!("Bulk dirs: {}", bulk_count);
    }
}
