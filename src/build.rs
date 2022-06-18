use std::env;
use std::fs::{File, OpenOptions};
use std::path::{PathBuf, Path};
use std::process::{Command, Stdio};
use std::io::BufReader;
use clap::ArgMatches;

use cargo_metadata::{Artifact, Message, Package, MetadataCommand};
use linkle::format::{nacp::NacpFile, nxo::NxoFile, romfs::RomFs, pfs0::Pfs0, npdm::NpdmInput, npdm::AcidBehavior};

#[derive(Debug, Serialize, Deserialize, Default)]
struct NspMetadata {
    npdm: Option<NpdmInput>,
    npdm_json: Option<String>
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct NroMetadata {
    romfs: Option<String>,
    icon: Option<String>,
    nacp: Option<NacpFile>
}

// Note: when the tier 3 target gets added (https://github.com/rust-lang/rust/pull/88991), we will no longer need to manually include/write these for building

const DEFAULT_TARGET_TRIPLE_64: &str = "aarch64-nintendo-switch";
const DEFAULT_TARGET_JSON_64: &str = include_str!("../default/specs/aarch64-nintendo-switch.json");
const DEFAULT_TARGET_LD_64: &str = include_str!("../default/specs/aarch64-nintendo-switch.ld");

const DEFAULT_TARGET_TRIPLE_32: &str = "armv7-nintendo-switch";
const DEFAULT_TARGET_JSON_32: &str = include_str!("../default/specs/armv7-nintendo-switch.json");
const DEFAULT_TARGET_LD_32: &str = include_str!("../default/specs/armv7-nintendo-switch.ld");

#[inline]
const fn get_default_target_triple(is_32bit: bool) -> &'static str {
    if is_32bit {
        DEFAULT_TARGET_TRIPLE_32
    }
    else {
        DEFAULT_TARGET_TRIPLE_64
    }
}

#[inline]
const fn get_default_target_json(is_32bit: bool) -> &'static str {
    if is_32bit {
        DEFAULT_TARGET_JSON_32
    }
    else {
        DEFAULT_TARGET_JSON_64
    }
}

#[inline]
const fn get_default_target_ld(is_32bit: bool) -> &'static str {
    if is_32bit {
        DEFAULT_TARGET_LD_32
    }
    else {
        DEFAULT_TARGET_LD_64
    }
}

fn prepare_default_target(root: &str, is_32bit: bool) -> String {
    let target_path = format!("{}/target", root);
    std::fs::create_dir_all(target_path.clone()).unwrap();

    let json = format!("{}/{}.json", target_path, get_default_target_triple(is_32bit));
    let ld = format!("{}/{}.ld", target_path, get_default_target_triple(is_32bit));

    std::fs::write(json, get_default_target_json(is_32bit).replace("<ld_path>", ld.as_str())).unwrap();
    std::fs::write(ld, get_default_target_ld(is_32bit).to_string()).unwrap();
    
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

    let npdm = if let Some(npdm_json) = metadata.npdm_json {
        let npdm_json_path = root.join(npdm_json.clone());
        NpdmInput::from_json(&npdm_json_path).unwrap()
    }
    else if let Some(npdm) = metadata.npdm {
        npdm
    }
    else {
        panic!("No npdm specified")
    };

    let mut option = OpenOptions::new();
    let output_option = option.write(true).create(true).truncate(true);
    let mut out_file = output_option.open(main_npdm.clone()).map_err(|err| (err, main_npdm.clone())).unwrap();
    npdm.into_npdm(&mut out_file, AcidBehavior::Empty).unwrap();

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

pub fn handle_build(build_cmd: &ArgMatches) {
    let is_verbose = build_cmd.is_present("verbose");

    let is_release = build_cmd.is_present("release");
    if is_verbose {
        println!("Profile: {}", match is_release {
            true => "release",
            false => "dev"
        });
    }

    let is_32bit = build_cmd.is_present("arm");

    let path = match build_cmd.value_of("path") {
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
        println!("Building and generating NSP...");
    }
    else if is_nro {
        println!("Building and generating NRO...");
    }
    else {
        println!("Building...");
    }

    let rust_target_path = match env::var("RUST_TARGET_PATH") {
        Ok(s) => PathBuf::from(s),
        Err(_) => metadata.workspace_root.clone()
    };

    let triple = match build_cmd.value_of("triple") {
        Some(triple_str) => triple_str,
        None => get_default_target_triple(is_32bit)
    };
    if is_verbose {
        println!("Triple: {}", triple);
    }

    let use_default_target = build_cmd.value_of("use-custom-target").is_none();
    if is_verbose {
        println!("Use default target: {}", use_default_target);
    }

    let build_target_path = match use_default_target {
        true => prepare_default_target(rust_target_path.to_str().unwrap(), is_32bit),
        false => rust_target_path.to_str().unwrap().into(),
    };
    if is_verbose {
        println!("Build target path: {}", build_target_path);
    }

    let mut build_args: Vec<String> = vec![
        String::from("build"),
        format!("--target={}", triple),
        String::from("--message-format=json-diagnostic-rendered-ansi")
    ];
    if is_release {
        build_args.push(String::from("--release"));
    }

    let mut command = Command::new("cargo")
        .args(&build_args)
        .stdout(Stdio::piped())
        .env("RUST_TARGET_PATH", build_target_path)
        .current_dir(path)
        .spawn()
        .unwrap();
    
    let reader = BufReader::new(command.stdout.take().unwrap());
    for message in Message::parse_stream(reader) {
        match message {
            Ok(Message::CompilerArtifact(ref artifact)) => {
                if artifact.target.kind.contains(&"bin".into()) || artifact.target.kind.contains(&"cdylib".into()) {
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
}
