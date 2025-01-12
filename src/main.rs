use clap::Parser as _;
use tracing_subscriber::EnvFilter;
mod build;
mod link;
mod new;

fn main() {
    // Set up the logger
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Parse the command-line arguments and handle the subcommand
    let Cargo::Nx(args) = Cargo::parse();
    match args.subcommand {
        CargoNxSubcommand::New(args) => new::handle_subcommand(args),
        CargoNxSubcommand::Build(args) => build::handle_subcommand(args),
        CargoNxSubcommand::Link(args) => link::handle_subcommand(args),
    }
}

#[derive(clap::Parser)]
#[clap(name = "cargo", bin_name = "cargo")]
enum Cargo {
    Nx(CargoNxArgs),
}

#[derive(clap::Args)]
#[clap(author, version, about)]
struct CargoNxArgs {
    #[command(subcommand)]
    pub subcommand: CargoNxSubcommand,
}

#[derive(clap::Subcommand)]
enum CargoNxSubcommand {
    #[command(about = "Create a new Rust project for the Nintendo Switch")]
    New(new::Args),
    #[command(about = "Build a Rust project for the Nintendo Switch")]
    Build(build::Args),
    #[command(about = "Send a file to the Nintendo Switch")]
    Link(link::Args),
}
