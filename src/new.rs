use crate::args::{CargoNxNew, PackageKind};
use std::fs;

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

#[derive(Debug, Default)]
struct PackageInfo<'a> {
    name: &'a str,
    author: &'a str,
    version: &'a str,
    edition: u16,
    program_id: u64,
}

fn process_default_file<'a>(file: &str, replace_info: &PackageInfo<'a>) -> String {
    file.replace("<name>", replace_info.name)
        .replace("<author>", replace_info.author)
        .replace("<version>", replace_info.version)
        .replace("<edition>", format!("{}", replace_info.edition).as_str())
        .replace(
            "<program_id>",
            format!("0x{:016X}", replace_info.program_id).as_str(),
        )
}

pub fn handle_new(args: CargoNxNew) {
    if args.path.is_dir() {
        panic!("Specified path already exists...");
    }

    let name = args.name.as_deref().unwrap_or_else(|| {
        args.path
            .file_name()
            .expect("path has invalid file name")
            .to_str()
            .expect("path file name is not valid UTF-8")
    });
    let edition = args
        .edition
        .parse::<u16>()
        .expect("invalid edition. how did this even happen??");
    let version = INITIAL_VERSION;
    let author = DEFAULT_AUTHOR;
    let program_id = DEFAULT_PROGRAM_ID;
    let info = PackageInfo {
        name,
        edition,
        version,
        author,
        program_id,
    };

    fs::create_dir_all(&args.path).expect("failed to create project directory");

    let cargo_toml = match args.kind {
        PackageKind::Lib => DEFAULT_LIB_CARGO_TOML,
        PackageKind::Nro => DEFAULT_NRO_CARGO_TOML,
        PackageKind::Nsp => DEFAULT_NSP_CARGO_TOML,
    };
    let cargo_config_toml = match args.kind {
        PackageKind::Lib => DEFAULT_LIB_CARGO_CONFIG_TOML,
        PackageKind::Nro => DEFAULT_NRO_CARGO_CONFIG_TOML,
        PackageKind::Nsp => DEFAULT_NSP_CARGO_CONFIG_TOML,
    };
    let src_main_file = match args.kind {
        PackageKind::Lib => DEFAULT_LIB_SRC_LIB_RS,
        PackageKind::Nro => DEFAULT_NRO_SRC_MAIN_RS,
        PackageKind::Nsp => DEFAULT_NSP_SRC_MAIN_RS,
    };

    let cargo_toml = process_default_file(cargo_toml, &info);
    fs::write(args.path.join("Cargo.toml"), cargo_toml)
        .expect("failed to create project Cargo.toml");

    let dot_cargo_path = args.path.join(".cargo");
    fs::create_dir(dot_cargo_path.clone()).expect("failed to create project .cargo directory");

    let cargo_config_toml = process_default_file(cargo_config_toml, &info);
    fs::write(dot_cargo_path.join("config.toml"), cargo_config_toml)
        .expect("failed to write to project .cargo/config.toml");

    let src_path = args.path.join("src");
    fs::create_dir(&src_path).expect("failed to create project src directory");

    let main_file_path = match args.kind {
        PackageKind::Lib => src_path.join("lib.rs"),
        PackageKind::Nro | PackageKind::Nsp => src_path.join("main.rs"),
    };

    let src_lib_rs = process_default_file(src_main_file, &info);
    fs::write(&main_file_path, src_lib_rs).expect("failed to create project lib/main file");

    println!("Created `{}` package ({})", info.name, args.kind);
}
