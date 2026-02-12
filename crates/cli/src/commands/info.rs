use rsfdl_core::sfdl::crypto::{decrypt_container, try_passwords, validate_password};
use rsfdl_core::sfdl::models::{FtpDataConnectionType, SfdlContainer, SslProtocol};
use rsfdl_core::sfdl::parser::parse_sfdl;
use std::process;

use rsfdl_core::format_bytes;

pub fn run(file: &str, password: Option<&str>, password_list: &[String]) {
    let xml = match std::fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Cannot read file '{}': {}", file, e);
            process::exit(1);
        }
    };

    let mut container = match parse_sfdl(&xml) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    let encrypted_label = if container.encrypted {
        if let Some(pw) = password {
            if !validate_password(&container, pw) {
                eprintln!("Error: Invalid password");
                process::exit(1);
            }
            if let Err(e) = decrypt_container(&mut container, pw) {
                eprintln!("Error: Decryption failed: {}", e);
                process::exit(1);
            }
            "yes (decrypted)"
        } else if let Some(pw) = try_passwords(&container, password_list) {
            if let Err(e) = decrypt_container(&mut container, &pw) {
                eprintln!("Error: Auto-decrypt failed: {}", e);
                process::exit(1);
            }
            eprintln!("Auto-decrypted with password from list");
            "yes (auto-decrypted)"
        } else {
            eprintln!("Error: File is encrypted. Provide a password with -p <password>");
            process::exit(1);
        }
    } else {
        "no"
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
