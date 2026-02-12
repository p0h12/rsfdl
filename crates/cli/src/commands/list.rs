use rsfdl_core::sfdl::crypto::{decrypt_container, try_passwords, validate_password};
use rsfdl_core::sfdl::models::HashType;
use rsfdl_core::sfdl::parser::parse_sfdl;
use std::process;

use rsfdl_core::format_bytes;

pub async fn run(file: &str, password: Option<&str>, password_list: &[String], resolve: bool) {
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

    if container.encrypted {
        if let Some(pw) = password {
            if !validate_password(&container, pw) {
                eprintln!("Error: Invalid password");
                process::exit(1);
            }
            if let Err(e) = decrypt_container(&mut container, pw) {
                eprintln!("Error: Decryption failed: {}", e);
                process::exit(1);
            }
        } else if let Some(pw) = try_passwords(&container, password_list) {
            if let Err(e) = decrypt_container(&mut container, &pw) {
                eprintln!("Error: Auto-decrypt failed: {}", e);
                process::exit(1);
            }
            eprintln!("Auto-decrypted with password from list");
        } else {
            eprintln!("Error: File is encrypted. Provide a password with -p <password>");
            process::exit(1);
        }
    }

    // Resolve bulk folders if requested
    if resolve {
        let has_bulk = container
            .packages
            .iter()
            .any(|p| p.bulk_folder_mode && !p.bulk_folder_list.is_empty());

        if has_bulk {
            eprintln!("Resolving bulk folders via FTP...");
            match rsfdl_core::ftp::listing::resolve_container_bulk_folders(
                &container.connection,
                &container.packages,
                30, // default FTP timeout in seconds
            )
            .await
            {
                Ok(resolved_files) => {
                    // Group by package name
                    let mut files_by_package: std::collections::HashMap<
                        String,
                        Vec<rsfdl_core::sfdl::models::FileItem>,
                    > = std::collections::HashMap::new();
                    for file in resolved_files {
                        files_by_package
                            .entry(file.package_name.clone())
                            .or_default()
                            .push(file);
                    }
                    for pkg in &mut container.packages {
                        if let Some(new_files) = files_by_package.remove(&pkg.name) {
                            pkg.file_list.extend(new_files);
                        }
                        pkg.bulk_folder_list.clear();
                        pkg.bulk_folder_mode = false;
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to resolve bulk folders: {}", e);
                }
            }
        }
    }

    let mut total_files = 0usize;
    let mut total_bytes = 0u64;
    let mut total_bulk = 0usize;

    for pkg in &container.packages {
        if !pkg.name.is_empty() {
            if pkg.bulk_folder_mode {
                println!("Package: {} (Bulk Folder Mode)", pkg.name);
            } else {
                println!("Package: {}", pkg.name);
            }
        }
        println!();

        for file_item in &pkg.file_list {
            let hash_label = match file_item.hash_type {
                HashType::MD5 => "  [MD5]",
                HashType::CRC => "  [CRC]",
                HashType::SHA1 => "  [SHA1]",
                HashType::None => "",
            };

            println!(
                "  {:<60} {:>10}{}",
                file_item.full_path,
                format_bytes(file_item.file_size),
                hash_label
            );
            total_files += 1;
            total_bytes += file_item.file_size;
        }

        for bulk in &pkg.bulk_folder_list {
            println!("  [DIR] {}", bulk.bulk_folder_path);
            total_bulk += 1;
        }
    }

    println!();
    if total_files > 0 {
        println!("{} files, {} total", total_files, format_bytes(total_bytes));
    }
    if total_bulk > 0 {
        println!(
            "{} bulk folder(s) (use --resolve to list contents via FTP)",
            total_bulk
        );
    }
}
