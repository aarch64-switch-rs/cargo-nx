#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate clap;

mod args;
mod build;
mod link;
mod new;

use clap::Parser;

use self::args::{Cargo, CargoNxSubcommand};

fn main() {
    let Cargo::Nx(args) = Cargo::parse();
    match args.subcommand {
        CargoNxSubcommand::New(new_args) => new::handle_new(new_args),
        CargoNxSubcommand::Build(build_args) => build::handle_build(build_args),
        CargoNxSubcommand::Link(link_args) => link::handle_subcommand(link_args),
    }
}
