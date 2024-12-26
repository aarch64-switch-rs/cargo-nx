use std::{fmt, net::IpAddr, path::PathBuf};

use clap::{builder::PossibleValuesParser, Args, Parser, Subcommand};

/// The supported Rust editions
pub static SUPPORTED_EDITIONS: &[&str] = &["2015", "2018", "2021"];
/// Which Rust edition to use by default
pub static DEFAULT_EDITION: &str = "2021";
/// Which package type to use by default
pub static DEFAULT_PACKAGE_TYPE: &str = "nro";

#[derive(Parser)]
#[clap(name = "cargo", bin_name = "cargo")]
pub enum Cargo {
    Nx(CargoNxArgs),
}

#[derive(Args)]
#[clap(author, version, about)]
pub struct CargoNxArgs {
    #[clap(subcommand)]
    pub subcommand: CargoNxSubcommand,
}

#[derive(Subcommand)]
pub enum CargoNxSubcommand {
    New(CargoNxNew),
    Build(CargoNxBuild),
    Link(CargoNxLink),
}

#[derive(Args)]
#[clap(about = "Create a new Rust project for the Nintendo Switch")]
pub struct CargoNxNew {
    /// Select the package type that will be built by this project.
    #[clap(short = 't', long = "type", value_enum, default_value = DEFAULT_PACKAGE_TYPE)]
    pub kind: PackageKind,
    /// Set the Rust edition to use.
    #[clap(short, long, value_parser = PossibleValuesParser::new(SUPPORTED_EDITIONS), default_value = DEFAULT_EDITION)]
    pub edition: String,
    /// Set the name of the newly created package.
    /// The path directory name is used by default.
    #[clap(short, long)]
    pub name: Option<String>,
    /// The path where the new package will be created
    #[clap(value_parser, value_name = "DIR")]
    pub path: PathBuf,
}

#[derive(Args)]
#[clap(about = "Build a Rust project for the Nintendo Switch")]
pub struct CargoNxBuild {
    /// Builds using the release profile.
    #[clap(short, long)]
    pub release: bool,
    /// The path to the project to build.
    #[clap(short, long, default_value = ".", value_name = "DIR", value_parser)]
    pub path: PathBuf,
    /// The custom target triple to use, if any.
    #[clap(short, long)]
    pub target: Option<String>,
    /// Displays extra information during the build process.
    #[clap(short, long)]
    pub verbose: bool,
}

#[derive(Args)]
#[clap(about = "Send a file to the Nintendo Switch")]
pub struct CargoNxLink {
    /// The IP address of the netloader server.
    #[clap(short, long, value_parser)]
    pub address: Option<IpAddr>,
    /// The number of times to retry server discovery.
    #[clap(short, long, default_value_t = 10)]
    pub retries: u32,
    /// Set upload path for the file.
    #[clap(short, long, value_parser)]
    pub path: Option<PathBuf>,
    /// Extra arguments to pass to the NRO file.
    #[clap(long = "args", value_name = "ARGS")]
    pub extra_args: Option<String>,
    /// Start the nxLink stdio server after a successful file transfer.
    #[clap(short, long, action)]
    pub server: bool,
    /// NRO file to send to the netloader server.
    #[clap(value_name = "FILE", value_parser)]
    pub nro_file: PathBuf,
    /// Args to send to NRO
    #[clap(value_name = "ARGS", value_parser)]
    pub nro_args: Vec<String>,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
#[clap(rename_all = "lower")]
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
