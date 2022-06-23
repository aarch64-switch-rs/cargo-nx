use std::fmt;
use std::path::PathBuf;
use std::fs;
use clap::ArgMatches;

pub const SUPPORTED_EDITIONS: &[u32] = &[2015, 2018, 2021];
pub const DEFAULT_EDITION: u32 = 2021;

const INITIAL_VERSION: &str = "0.1.0";

const DEFAULT_AUTHOR: &str = "aarch64-switch-rs authors";

const DEFAULT_PROGRAM_ID: u64 = 0x0100AAAABBBBCCCC;

const DEFAULT_LIB_CARGO_TOML: &str = include_str!("../default/lib/Cargo.toml");
const DEFAULT_LIB_CARGO_CONFIG_TOML: &str = include_str!("../default/lib/.cargo/config.toml");
const DEFAULT_LIB_SRC_LIB_RS: &str = include_str!("../default/lib/src/lib.rs");

const DEFAULT_NRO_CARGO_TOML: &str = include_str!("../default/nro/Cargo.toml");
const DEFAULT_NRO_CARGO_CONFIG_TOML: &str = include_str!("../default/nro/.cargo/config.toml");
const DEFAULT_NRO_SRC_MAIN_RS: &str = include_str!("../default/nro/src/main.rs");

const DEFAULT_NSP_CARGO_TOML: &str = include_str!("../default/nsp/Cargo.toml");
const DEFAULT_NSP_CARGO_CONFIG_TOML: &str = include_str!("../default/nsp/.cargo/config.toml");
const DEFAULT_NSP_SRC_MAIN_RS: &str = include_str!("../default/nsp/src/main.rs");

#[derive(Debug)]
enum PackageKind {
    Lib,
    Nro,
    Nsp
}

impl fmt::Display for PackageKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fmt_str = match self {
            PackageKind::Lib => "lib",
            PackageKind::Nro => "nro",
            PackageKind::Nsp => "nsp"
        };

        write!(f, "{}", fmt_str)
    }
}

#[derive(Debug, Default)]
struct PackageInfo<'a> {
    name: &'a str,
    author: &'a str,
    version: &'a str,
    edition: u32,
    program_id: u64
}

fn process_default_file<'a>(file: &str, replace_info: &PackageInfo<'a>) -> String {
    file.replace("<name>", replace_info.name)
        .replace("<author>", replace_info.author)
        .replace("<version>", replace_info.version)
        .replace("<edition>", format!("{}", replace_info.edition).as_str())
        .replace("<program_id>", format!("0x{:016X}", replace_info.program_id).as_str())
}

pub fn handle_new(new_cmd: &ArgMatches) {
    let path = new_cmd.value_of("path").unwrap();
    let path_b = PathBuf::from(path);

    if path_b.is_dir() {
        panic!("Specified path already exists...");
    }

    let mut info: PackageInfo = Default::default();

    info.name = new_cmd.value_of("name").unwrap_or(path_b.file_name().unwrap().to_str().unwrap());

    let mut kind = PackageKind::Nro;
    let mut kind_arg_count = 0;
    if new_cmd.is_present("lib") {
        kind_arg_count += 1;
        kind = PackageKind::Lib;
    }
    if new_cmd.is_present("nro") {
        kind_arg_count += 1;
        kind = PackageKind::Nro;
    }
    if new_cmd.is_present("nsp") {
        kind_arg_count += 1;
        kind = PackageKind::Nsp;
    }
    if kind_arg_count > 1 {
        panic!("Too many package kinds specified...");
    }

    info.edition = DEFAULT_EDITION;
    if let Some(edition_v) = new_cmd.value_of("edition") {
        if let Ok(edition_val) = edition_v.parse::<u32>() {
            if SUPPORTED_EDITIONS.contains(&edition_val) {
                info.edition = edition_val;
            }
            else {
                panic!("Unsupported edition -- supported editions: {:?}", SUPPORTED_EDITIONS);
            }
        }
        else {
            panic!("The specified edition is not a valid number...");
        }
    }

    info.version = INITIAL_VERSION;

    info.author = DEFAULT_AUTHOR;

    info.program_id = DEFAULT_PROGRAM_ID;

    fs::create_dir_all(path_b.clone()).unwrap();

    match kind {
        PackageKind::Lib => {
            let cargo_toml = process_default_file(DEFAULT_LIB_CARGO_TOML, &info);
            let cargo_toml_path = format!("{}/Cargo.toml", path);
            fs::write(cargo_toml_path, cargo_toml).unwrap();

            let cargo_path = format!("{}/.cargo", path);
            fs::create_dir(cargo_path.clone()).unwrap();

            let cargo_config_toml = process_default_file(DEFAULT_LIB_CARGO_CONFIG_TOML, &info);
            let cargo_config_toml_path = format!("{}/config.toml", cargo_path);
            fs::write(cargo_config_toml_path, cargo_config_toml).unwrap();
            
            let src_path = format!("{}/src", path);
            fs::create_dir(src_path.clone()).unwrap();

            let src_lib_rs = process_default_file(DEFAULT_LIB_SRC_LIB_RS, &info);
            let src_lib_rs_path = format!("{}/lib.rs", src_path);
            fs::write(src_lib_rs_path, src_lib_rs).unwrap();
        },
        PackageKind::Nro => {
            let cargo_toml = process_default_file(DEFAULT_NRO_CARGO_TOML, &info);
            let cargo_toml_path = format!("{}/Cargo.toml", path);
            fs::write(cargo_toml_path, cargo_toml).unwrap();

            let cargo_path = format!("{}/.cargo", path);
            fs::create_dir(cargo_path.clone()).unwrap();

            let cargo_config_toml = process_default_file(DEFAULT_NRO_CARGO_CONFIG_TOML, &info);
            let cargo_config_toml_path = format!("{}/config.toml", cargo_path);
            fs::write(cargo_config_toml_path, cargo_config_toml).unwrap();
            
            let src_path = format!("{}/src", path);
            fs::create_dir(src_path.clone()).unwrap();

            let src_main_rs = process_default_file(DEFAULT_NRO_SRC_MAIN_RS, &info);
            let src_main_rs_path = format!("{}/main.rs", src_path);
            fs::write(src_main_rs_path, src_main_rs).unwrap();
        },
        PackageKind::Nsp => {
            let cargo_toml = process_default_file(DEFAULT_NSP_CARGO_TOML, &info);
            let cargo_toml_path = format!("{}/Cargo.toml", path);
            fs::write(cargo_toml_path, cargo_toml).unwrap();

            let cargo_path = format!("{}/.cargo", path);
            fs::create_dir(cargo_path.clone()).unwrap();

            let cargo_config_toml = process_default_file(DEFAULT_NSP_CARGO_CONFIG_TOML, &info);
            let cargo_config_toml_path = format!("{}/config.toml", cargo_path);
            fs::write(cargo_config_toml_path, cargo_config_toml).unwrap();
            
            let src_path = format!("{}/src", path);
            fs::create_dir(src_path.clone()).unwrap();

            let src_main_rs = process_default_file(DEFAULT_NSP_SRC_MAIN_RS, &info);
            let src_main_rs_path = format!("{}/main.rs", src_path);
            fs::write(src_main_rs_path, src_main_rs).unwrap();
        }
    }

    println!("Created `{}` package ({})", info.name, kind);
}