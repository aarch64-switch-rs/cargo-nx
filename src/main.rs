extern crate linkle;
extern crate serde;
extern crate serde_json;
extern crate cargo_metadata;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate clap;

use std::env;
use std::fs::{File, OpenOptions};
use std::path::{PathBuf, Path};
use std::process::{Command, Stdio};
use std::io::BufReader;
use clap::{Arg, AppSettings, App, SubCommand};

use cargo_metadata::{Artifact, Message, Package, MetadataCommand};
use linkle::format::{nacp::NacpFile, nxo::NxoFile, romfs::RomFs, pfs0::Pfs0, npdm::NpdmJson, npdm::ACIDBehavior};

#[derive(Debug, Serialize, Deserialize, Default)]
struct NspMetadata {
    npdm: String
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct NroMetadata {
    romfs: Option<String>,
    icon: Option<String>,
    nacp: Option<NacpFile>
}

const DEFAULT_TARGET_TRIPLE: &str = "aarch64-none-elf";
const DEFAULT_TARGET_JSON: &str = include_str!("../default_specs/aarch64-none-elf.json");
const DEFAULT_TARGET_LD: &str = include_str!("../default_specs/aarch64-none-elf.ld");

fn prepare_default_target(root: &str) -> String {
    let target_path = format!("{}/target", root);
    std::fs::create_dir_all(target_path.clone()).unwrap();

    let json = format!("{}/{}.json", target_path, DEFAULT_TARGET_TRIPLE);
    let ld = format!("{}/{}.ld", target_path, DEFAULT_TARGET_TRIPLE);

    std::fs::write(json, DEFAULT_TARGET_JSON.replace("$LD_PATH", ld.as_str())).unwrap();
    std::fs::write(ld, DEFAULT_TARGET_LD.to_string()).unwrap();
    
    target_path
}

fn get_output_elf_path_as(artifact: &Artifact, extension: &str) -> PathBuf {
    let mut elf = artifact.filenames[0].clone();
    assert!(elf.set_extension(extension));
    elf
}

fn handle_nro_format(root: &Path, artifact: &Artifact, metadata: NroMetadata) {
    let elf = artifact.filenames[0].clone();
    let nro = get_output_elf_path_as(artifact, "nro");

    let romfs = metadata.romfs.as_ref().map(|romfs_dir| RomFs::from_directory(&root.join(romfs_dir)).unwrap());
    let icon = metadata.icon.as_ref().map(|icon_file| root.join(icon_file.clone())).map(|icon_path| icon_path.to_string_lossy().into_owned());

    NxoFile::from_elf(elf.to_str().unwrap())
    .unwrap()
    .write_nro(
        &mut File::create(nro.clone()).unwrap(),
        romfs,
        icon.as_ref().map(|icon_path| icon_path.as_str()),
        metadata.nacp,
    )
    .unwrap();

    println!("Built {}", nro.to_string_lossy());
}

fn handle_nsp_format(root: &Path, artifact: &Artifact, metadata: NspMetadata) {
    let elf = artifact.filenames[0].clone();
    
    let output_path = elf.parent().unwrap();
    let exefs_dir = output_path.join("exefs");
    let _ = std::fs::remove_dir_all(exefs_dir.clone());
    std::fs::create_dir(exefs_dir.clone()).unwrap();

    let main_npdm = exefs_dir.join("main.npdm");
    let main_exe = exefs_dir.join("main");

    let exefs_nsp = get_output_elf_path_as(artifact, "nsp");

    let npdm_json = root.join(metadata.npdm.clone());
    let npdm = NpdmJson::from_file(&npdm_json).unwrap();
    let mut option = OpenOptions::new();
    let output_option = option.write(true).create(true).truncate(true);
    let mut out_file = output_option.open(main_npdm.clone()).map_err(|err| (err, main_npdm.clone())).unwrap();
    npdm.into_npdm(&mut out_file, ACIDBehavior::Empty).unwrap();

    NxoFile::from_elf(elf.to_str().unwrap()).unwrap().write_nso(&mut File::create(main_exe.clone()).unwrap()).unwrap();

    let mut nsp = Pfs0::from_directory(exefs_dir.to_str().unwrap()).unwrap();
    let mut option = OpenOptions::new();
    let output_option = option.write(true).create(true).truncate(true);
    nsp.write_pfs0(
        &mut output_option
            .open(exefs_nsp.clone())
            .map_err(|err| (err, exefs_nsp.clone())).unwrap(),
    )
    .map_err(|err| (err, exefs_nsp.clone())).unwrap();

    println!("Built {}", exefs_nsp.to_string_lossy());
}

fn main() {
    let matches = App::new(crate_name!())
        .version(concat!("v", crate_version!()))
        .bin_name("cargo")
        .settings(&[AppSettings::GlobalVersion,
            AppSettings::SubcommandRequired])
        .subcommand(SubCommand::with_name("nx")
            .author(crate_authors!(""))
            .about(crate_description!())
            .arg(Arg::with_name("profile")
                .help("Build profile")
                .possible_values(&["dev", "release"])
                .required(true))
            .arg(Arg::with_name("path")
                .short("p")
                .long("path")
                .value_name("DIR")
                .help("Sets a custom path")
                .required(false)
                .takes_value(true))
            .arg(Arg::with_name("triple")
                .short("tp")
                .long("triple")
                .value_name("TRIPLE")
                .help("Sets a custom target triple")
                .required(false)
                .takes_value(true))
            .arg(Arg::with_name("use-custom-target")
                .short("ctg")
                .long("use-custom-target")
                .help("Avoids using the default target files")
                .required(false))
            .arg(Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Show extra information")
                .required(false)))
        .get_matches();

    let nx_matches = matches.subcommand_matches("nx").unwrap();

    let is_verbose = nx_matches.value_of("verbose").is_some();

    let profile = nx_matches.value_of("profile").unwrap();
    if is_verbose {
        println!("Profile: {}", profile);
    }

    let path = match nx_matches.value_of("path") {
        Some(path_str) => path_str,
        None => "."
    };

    let metadata = MetadataCommand::new()
        .manifest_path(Path::new(path).join("Cargo.toml"))
        .no_deps()
        .exec()
        .unwrap();

    let metadata_v = &metadata.packages[0].metadata;

    let is_nsp = metadata_v.pointer("/nx/nsp").is_some();
    let is_nro = metadata_v.pointer("/nx/nro").is_some();

    if is_nsp && is_nro {
        panic!("Error: multiple target formats are not yet supported...");
    }
    else if is_nsp {
        println!("Compiling and generating NSP...");
    }
    else if is_nro {
        println!("Compiling and generating NRO...");
    }
    else {
        panic!("Error: no target formats in Cargo.toml...");
    }

    let rust_target_path = match env::var("RUST_TARGET_PATH") {
        Ok(s) => PathBuf::from(s),
        Err(_) => metadata.workspace_root.clone()
    };

    let triple = match nx_matches.value_of("triple") {
        Some(triple_str) => triple_str,
        None => DEFAULT_TARGET_TRIPLE
    };
    if is_verbose {
        println!("Triple: {}", triple);
    }

    let use_default_target = nx_matches.value_of("use-custom-target").is_none();
    if is_verbose {
        println!("Use default target: {}", use_default_target);
    }

    let build_target_path = match use_default_target {
        true => prepare_default_target(rust_target_path.to_str().unwrap()),
        false => rust_target_path.to_str().unwrap().into(),
    };
    if is_verbose {
        println!("Build target path: {}", build_target_path);
    }

    let xargo_args: Vec<String> = vec![
        String::from("build"),
        format!("--target={}", triple),
        String::from("--message-format=json-diagnostic-rendered-ansi"),
        format!("--{}", profile)
    ];

    let mut command = Command::new("xargo")
        .args(&xargo_args)
        .stdout(Stdio::piped())
        .env("RUST_TARGET_PATH", build_target_path)
        .current_dir(path)
        .spawn()
        .unwrap();
    
    let reader = BufReader::new(command.stdout.take().unwrap());
    for message in Message::parse_stream(reader) {
        match message {
            Ok(Message::CompilerArtifact(ref artifact))
                if artifact.target.kind.contains(&"bin".into()) || artifact.target.kind.contains(&"cdylib".into()) => {
                let package: &Package = match metadata.packages.iter().find(|v| v.id == artifact.package_id) {
                    Some(v) => v,
                    None => continue,
                };

                let root = package.manifest_path.parent().unwrap();

                if is_nsp {
                    let nsp_metadata: NspMetadata = serde_json::from_value(metadata_v.pointer("/nx/nsp").cloned().unwrap()).unwrap_or_default();
                    handle_nsp_format(root, artifact, nsp_metadata);
                }
                else if is_nro {
                    let nro_metadata: NroMetadata = serde_json::from_value(metadata_v.pointer("/nx/nro").cloned().unwrap()).unwrap_or_default();
                    handle_nro_format(root, artifact, nro_metadata);
                }
            }
            Ok(Message::CompilerMessage(msg)) => {
                if let Some(msg) = msg.message.rendered {
                    println!("{}", msg);
                } else {
                    println!("{:?}", msg);
                }
            }
            Ok(_) => (),
            Err(err) => {
                panic!("{:?}", err);
            }
        }
    }

    println!("Done!");
}
