//! The `cargo nx link` subcommand.
//!
//! This is a Rust implementation of the `nxlink` command-line tool.
//!
//! It sends a file to the Nintendo Switch using the _nx-hbmenu netloader_.
//!
//! See: https://github.com/switchbrew/switch-tools/blob/22756068dd0ed6ff9734c59cb4f99ebd3f62555b/src/nxlink.c

use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
    time::Duration,
};

use netloader::loader::send::send_nro_file;

/// The `link` subcommand CLI arguments.
#[derive(clap::Args)]
pub struct Args {
    /// The IP address of the netloader server.
    #[arg(short, long, value_parser)]
    pub address: Option<IpAddr>,
    /// The number of times to retry server discovery.
    #[arg(short, long, default_value_t = 10)]
    pub retries: u32,
    /// Set upload path for the file.
    #[arg(short, long, value_parser)]
    pub path: Option<PathBuf>,
    /// Extra arguments to pass to the NRO file.
    #[arg(long = "args", value_name = "ARGS")]
    pub extra_args: Option<String>,
    /// Start the nxLink stdio server after a successful file transfer.
    #[arg(short, long, action)]
    pub server: bool,
    /// NRO file to send to the netloader server.
    #[arg(value_name = "FILE", value_parser)]
    pub nro_file: PathBuf,
    /// Args to send to NRO
    #[arg(value_name = "ARGS", value_parser)]
    pub nro_args: Vec<String>,
}

/// Handle the `link` subcommand.
#[tokio::main(flavor = "current_thread")]
pub async fn handle_subcommand(
    Args {
        address,
        retries,
        path,
        extra_args,
        server,
        nro_file,
        mut nro_args,
    }: Args,
) {
    tracing::debug!("File path: {}", nro_file.display());

    // Check if the file exists
    if !nro_file.exists() {
        eprintln!("The file does not exist: {}", nro_file.display());
        return;
    }

    if !nro_file.is_file() {
        eprintln!("The path is not a file: {}", nro_file.display());
        return;
    }

    // Check if the file extension is valid
    if !nro_file.extension().map_or(false, |ext| ext == "nro") {
        eprintln!(
            "The file must have a `.nro` extension: {}",
            nro_file.display()
        );
        return;
    }

    // Get the file name
    let nro_file_name = match nro_file.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => {
            eprintln!("Failed to get the file name");
            return;
        }
    };

    tracing::debug!("NRO file name: {}", nro_file_name);

    // If the path has a `.nro` extension, use it as the destination path
    // Otherwise, if the path ends with a `/`, join the file name to the path
    let dest_path = match path {
        Some(path) => {
            if path.extension().map_or(false, |ext| ext == "nro") {
                path.to_str()
                    .expect("Failed to convert path to string")
                    .to_string()
            } else if path.to_str().map_or(false, |path| path.ends_with("/")) {
                path.join(nro_file_name)
                    .to_str()
                    .expect("Failed to convert path to string")
                    .to_string()
            } else {
                eprintln!("Invalid path: {}", path.display());
                return;
            }
        }
        // Otherwise, use the NRO file name
        None => nro_file_name,
    };

    tracing::debug!("Destination path: {}", dest_path);

    // Open the file for reading
    let mut file = match std::fs::File::open(nro_file) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to read the file: {}", e);
            return;
        }
    };

    // Get the file length
    let file_length = match file.metadata() {
        Ok(metadata) => metadata.len() as usize,
        Err(e) => {
            eprintln!("Failed to get the file size: {}", e);
            return;
        }
    };

    tracing::debug!("File length: {}", file_length);

    // Parse the extra arguments, and add them to the NRO arguments
    if let Some(extra_args) = extra_args {
        let extra_args = parse_extra_args(extra_args);
        if !extra_args.is_empty() {
            nro_args.extend(extra_args);
        }
    }

    // Determine the server IP address
    let remote_addr = match address {
        Some(ip_addr) => (ip_addr, netloader::SERVER_PORT),
        None => {
            match netloader::loader::discovery::discover(Duration::from_millis(250), retries).await
            {
                Ok(Some(ip_addr)) => (ip_addr, netloader::SERVER_PORT),
                Ok(None) => {
                    eprintln!("No server found in the network");
                    return;
                }
                Err(err) => {
                    eprintln!("Server discovery failed: {}", err);
                    return;
                }
            }
        }
    };

    println!("Sending file to: {}", remote_addr.0);

    // Send the file to the remote server
    tokio::select! {biased;
        res = send_nro_file(remote_addr, &dest_path, &mut file, file_length, nro_args) => {
            match res {
                Ok(_) => {
                    println!("File sent successfully");
                }
                Err(err) => {
                    eprintln!("Failed to send the file: {err}");
                }
            }
        }
        _ = tokio::signal::ctrl_c() => {
            eprintln!("Aborted by the user");
        }
    }

    // Start the nxlink stdio server if requested
    if server {
        println!("Starting the nxlink stdio server. Press Ctrl+C to exit.");

        let stdio_server_addr = (Ipv4Addr::UNSPECIFIED, netloader::CLIENT_PORT);
        tokio::select! {biased;
            _ = netloader::stdio::start_server(stdio_server_addr) => {}
            _ = tokio::signal::ctrl_c() => {}
        }
    }
}

/// Parse the extra arguments CLI string into a vector of arguments.
fn parse_extra_args(args: String) -> Vec<String> {
    let mut args_chars = args.trim().chars();
    let mut result = Vec::new();

    let mut current_arg = String::new();
    while let Some(current_char) = args_chars.next() {
        if current_char == ' ' {
            continue;
        }

        // If the argument is quoted, parse until the closing quote,
        // otherwise parse until the next space
        if current_char == '"' || current_char == '\'' {
            let quote = current_char;
            for c in args_chars.by_ref() {
                if c == quote {
                    break;
                }
                current_arg.push(c);
            }
        } else {
            // Add the current character to the current argument
            current_arg.push(current_char);

            // Parse until the next space
            for c in args_chars.by_ref() {
                if c == ' ' {
                    break;
                }
                current_arg.push(c);
            }
        }

        // Add the current argument to the result
        if !current_arg.is_empty() {
            result.push(std::mem::take(&mut current_arg));
        }
    }

    result
}
