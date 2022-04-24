extern crate linkle;
extern crate serde;
extern crate serde_json;
extern crate cargo_metadata;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate clap;

use std::env;
use clap::{Arg, AppSettings, App, SubCommand};

mod new;

mod build;

fn main() {
    let matches =
        App::new(crate_name!())
        .version(concat!("v", crate_version!()))
        .bin_name("cargo")
        .setting(AppSettings::SubcommandRequired)
        .subcommand(SubCommand::with_name("nx")
            .author(crate_authors!(""))
            .about(crate_description!())
            .setting(AppSettings::SubcommandRequired)
            .subcommand(SubCommand::with_name("new")
                .arg(
                    Arg::with_name("path")
                    .required(true)
                    .help("New package path")
                )
                .arg(
                    Arg::with_name("name")
                    .long("name")
                    .required(false)
                    .help("Set the new package name (path directory name used by default)")
                    .takes_value(true)
                    .value_name("NAME")
                )
                .arg(
                    Arg::with_name("edition")
                    .long("edition")
                    .required(false)
                    .help(format!("Set the Rust edition to use (values: {:?}, default: {})", new::SUPPORTED_EDITIONS, new::DEFAULT_EDITION).as_str())
                    .takes_value(true)
                    .value_name("EDITION")
                )
                .arg(
                    Arg::with_name("lib")
                    .long("lib")
                    .required(false)
                    .help("Create a library package")
                )
                .arg(
                    Arg::with_name("nro")
                    .long("nro")
                    .required(false)
                    .help("Create a NRO package (default behavior)")
                )
                .arg(
                    Arg::with_name("nsp")
                    .long("nsp")
                    .required(false)
                    .help("Create a NSP package")
                )
                // TODO: author, program ID support (maybe other NPDM/NACP fields?)
            )
            .subcommand(SubCommand::with_name("build")
                .arg(
                    Arg::with_name("release")
                    .short("r")
                    .long("release")
                    .help("Builds on release profile")
                    .required(false)
                )
                .arg(
                    Arg::with_name("path")
                    .short("p")
                    .long("path")
                    .takes_value(true)
                    .value_name("DIR")
                    .help("Sets a custom path")
                    .required(false)
                )
                .arg(
                    Arg::with_name("triple")
                    .short("tp")
                    .long("triple")
                    .takes_value(true)
                    .value_name("TRIPLE")
                    .help("Sets a custom target triple")
                    .required(false)
                )
                .arg(
                    Arg::with_name("use-custom-target")
                    .short("ctg")
                    .long("use-custom-target")
                    .help("Avoids using the default target files")
                    .required(false)
                )
                .arg(
                    Arg::with_name("verbose")
                    .short("v")
                    .long("verbose")
                    .help("Displays extra information during the build process")
                    .required(false)
                )
                .arg( // TODO: better way to do this?
                    Arg::with_name("arm")
                    .long("arm")
                    .required(false)
                    .help("Compiles as 32-bit default target (64-bit target is used by default)")
                )
            )
        )
        .get_matches();

    let nx_cmd = matches.subcommand_matches("nx").unwrap();
    if let Some(new_cmd) = nx_cmd.subcommand_matches("new") {
        new::handle_new(new_cmd);
    }
    else if let Some(build_cmd) = nx_cmd.subcommand_matches("build") {
        build::handle_build(build_cmd);
    }
}
