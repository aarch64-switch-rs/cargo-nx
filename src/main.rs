#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate clap;

mod args;
mod build;
mod new;

use self::args::{Cargo, CargoNxSubcommand};
use clap::Parser;

fn main() {
    let Cargo::Nx(args) = Cargo::parse();
    match args.subcommand {
        CargoNxSubcommand::New(new_args) => new::handle_new(new_args),
        CargoNxSubcommand::Build(build_args) => build::handle_build(build_args),
    }
}
