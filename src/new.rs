use std::{fmt, path::PathBuf};

const INITIAL_VERSION: &str = "0.1.0";

const DEFAULT_AUTHOR: &str = "aarch64-switch-rs authors";

const DEFAULT_PROGRAM_ID: u64 = 0x0100AAAABBBBCCCC;

/// The supported Rust editions
const SUPPORTED_EDITIONS: &[&str] = &["2015", "2018", "2021"];

/// Which Rust edition to use by default
const DEFAULT_EDITION: &str = "2021";

/// Which package type to use by default
const DEFAULT_PACKAGE_TYPE: &str = "nro";

const DEFAULT_LIB_CARGO_TOML: &str = include_str!("../default/lib/Cargo.toml");
const DEFAULT_LIB_CARGO_CONFIG_TOML: &str = include_str!("../default/lib/.cargo/config.toml");

const DEFAULT_LIB_SRC_LIB_RS: &str = include_str!("../default/lib/src/lib.rs");
const DEFAULT_NRO_CARGO_TOML: &str = include_str!("../default/nro/Cargo.toml");
const DEFAULT_NRO_CARGO_CONFIG_TOML: &str = include_str!("../default/nro/.cargo/config.toml");

const DEFAULT_NRO_SRC_MAIN_RS: &str = include_str!("../default/nro/src/main.rs");
const DEFAULT_NSP_CARGO_TOML: &str = include_str!("../default/nsp/Cargo.toml");
const DEFAULT_NSP_CARGO_CONFIG_TOML: &str = include_str!("../default/nsp/.cargo/config.toml");

const DEFAULT_NSP_SRC_MAIN_RS: &str = include_str!("../default/nsp/src/main.rs");

/// The `new` subcommand CLI arguments.
#[derive(clap::Args)]
pub struct Args {
    /// Select the package type that will be built by this project.
    #[arg(short = 't', long = "type", value_enum, default_value = DEFAULT_PACKAGE_TYPE)]
    pub kind: PackageKind,
    /// Set the Rust edition to use.
    #[arg(short, long, value_parser = clap::builder::PossibleValuesParser::new(SUPPORTED_EDITIONS), default_value = DEFAULT_EDITION
    )]
    pub edition: String,
    /// Set the name of the newly created package.
    /// The path directory name is used by default.
    #[arg(short, long)]
    pub name: Option<String>,
    /// The path where the new package will be created
    #[arg(value_parser, value_name = "DIR")]
    pub path: PathBuf,
}

/// Handle the `new` subcommand.
pub fn handle_subcommand(args: Args) {
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

    std::fs::create_dir_all(&args.path).expect("failed to create project directory");

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
    std::fs::write(args.path.join("Cargo.toml"), cargo_toml)
        .expect("failed to create project Cargo.toml");

    let dot_cargo_path = args.path.join(".cargo");
    std::fs::create_dir(dot_cargo_path.clone()).expect("failed to create project .cargo directory");

    let cargo_config_toml = process_default_file(cargo_config_toml, &info);
    std::fs::write(dot_cargo_path.join("config.toml"), cargo_config_toml)
        .expect("failed to write to project .cargo/config.toml");

    let src_path = args.path.join("src");
    std::fs::create_dir(&src_path).expect("failed to create project src directory");

    let main_file_path = match args.kind {
        PackageKind::Lib => src_path.join("lib.rs"),
        PackageKind::Nro | PackageKind::Nsp => src_path.join("main.rs"),
    };

    let src_lib_rs = process_default_file(src_main_file, &info);
    std::fs::write(&main_file_path, src_lib_rs).expect("failed to create project lib/main file");

    println!("Created `{}` package ({})", info.name, args.kind);
}

#[derive(Debug, Default)]
struct PackageInfo<'a> {
    name: &'a str,
    author: &'a str,
    version: &'a str,
    edition: u16,
    program_id: u64,
}

fn process_default_file(file: &str, replace_info: &PackageInfo<'_>) -> String {
    file.replace("<name>", replace_info.name)
        .replace("<author>", replace_info.author)
        .replace("<version>", replace_info.version)
        .replace("<edition>", format!("{}", replace_info.edition).as_str())
        .replace(
            "<program_id>",
            format!("0x{:016X}", replace_info.program_id).as_str(),
        )
}

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
#[value(rename_all = "lower")]
pub enum PackageKind {
    Lib,
    Nro,
    Nsp,
}

impl fmt::Display for PackageKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fmt_str = match self {
            PackageKind::Lib => "lib",
            PackageKind::Nro => "nro",
            PackageKind::Nsp => "nsp",
        };

        write!(f, "{}", fmt_str)
    }
}
