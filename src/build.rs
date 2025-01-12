use std::{
    fs::{File, OpenOptions},
    io::BufReader,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use cargo_metadata::{Artifact, Message, MetadataCommand, Package};
use linkle::format::{
    nacp::Nacp,
    npdm::{AcidBehavior, Npdm},
    nxo::Nxo,
    pfs0::Pfs0,
    romfs::RomFs,
};

/// The default target triple to use when building.
const DEFAULT_TARGET_TRIPLE: &str = "aarch64-nintendo-switch-freestanding";

/// The default icon to use when building an NRO.
const DEFAULT_NRO_ICON: &[u8] = include_bytes!("../default/nro/default_icon.jpg");

/// The `build` subcommand CLI arguments.
#[derive(clap::Args)]
pub struct Args {
    /// Builds using the release profile.
    #[arg(short, long)]
    pub release: bool,
    /// The path to the project to build.
    #[arg(short, long, default_value = ".", value_name = "DIR", value_parser)]
    pub path: PathBuf,
    /// The custom target triple to use, if any.
    #[arg(short, long)]
    pub target: Option<String>,
    /// Displays extra information during the build process.
    #[arg(short, long)]
    pub verbose: bool,
}

/// Handle the `build` subcommand.
pub fn handle_subcommand(args: Args) {
    let metadata = MetadataCommand::new()
        .manifest_path(args.path.join("Cargo.toml"))
        .no_deps()
        .exec()
        .unwrap();

    let metadata_v = &metadata.packages[0].metadata;

    let is_nsp = metadata_v.pointer("/nx/nsp").is_some();
    let is_nro = metadata_v.pointer("/nx/nro").is_some();
    if is_nsp && is_nro {
        panic!("Error: multiple target formats are not yet supported...");
    } else if is_nsp {
        println!("Building and generating NSP...");
    } else if is_nro {
        println!("Building and generating NRO...");
    } else {
        println!("Building...");
    }

    let rust_target_path = match std::env::var("RUST_TARGET_PATH") {
        Ok(s) => PathBuf::from(s),
        Err(_) => metadata.workspace_root.clone(),
    };

    let target = args.target.as_deref().unwrap_or(DEFAULT_TARGET_TRIPLE);
    if args.verbose {
        println!("Target triple: {}", target);
    }

    let build_target_path = rust_target_path.to_str().unwrap();
    if args.verbose {
        println!("Build target path: {}", build_target_path);
    }

    let mut build_args: Vec<String> = vec![
        String::from("build"),
        format!("--target={}", target),
        String::from("--message-format=json-diagnostic-rendered-ansi"),
    ];
    if args.release {
        build_args.push(String::from("--release"));
    }

    #[allow(clippy::zombie_processes)] // TODO: Fix `spawned process is never waited` clippy warning
    let mut command = Command::new("cargo")
        .args(&build_args)
        .stdout(Stdio::piped())
        .env("RUST_TARGET_PATH", build_target_path)
        .current_dir(&args.path)
        .spawn()
        .unwrap();

    let reader = BufReader::new(command.stdout.take().unwrap());
    for message in Message::parse_stream(reader) {
        match message {
            Ok(Message::CompilerArtifact(ref artifact)) => {
                if artifact.target.kind.contains(&"bin".into())
                    || artifact.target.kind.contains(&"cdylib".into())
                {
                    let package: &Package = match metadata
                        .packages
                        .iter()
                        .find(|v| v.id == artifact.package_id)
                    {
                        Some(v) => v,
                        None => continue,
                    };

                    let root = package.manifest_path.parent().unwrap();

                    if is_nsp {
                        let nsp_metadata: NspMetadata =
                            serde_json::from_value(metadata_v.pointer("/nx/nsp").cloned().unwrap())
                                .unwrap_or_default();
                        handle_nsp_format(root, artifact, nsp_metadata);
                    } else if is_nro {
                        let nro_metadata: NroMetadata =
                            serde_json::from_value(metadata_v.pointer("/nx/nro").cloned().unwrap())
                                .unwrap_or_default();
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

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct NspMetadata {
    npdm: Option<Npdm>,
    npdm_json: Option<String>,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct NroMetadata {
    romfs: Option<String>,
    icon: Option<String>,
    nacp: Option<Nacp>,
}

fn get_output_elf_path_as(artifact: &Artifact, extension: &str) -> PathBuf {
    let mut elf = artifact.filenames[0].clone();
    assert!(elf.set_extension(extension));
    elf
}

fn handle_nro_format(root: &Path, artifact: &Artifact, metadata: NroMetadata) {
    let elf = artifact.filenames[0].clone();
    let nro = get_output_elf_path_as(artifact, "nro");

    let romfs = metadata
        .romfs
        .as_ref()
        .map(|romfs_dir| RomFs::from_directory(&root.join(romfs_dir)).unwrap());
    let provided_icon = metadata
        .icon
        .as_ref()
        .map(|icon_file| root.join(icon_file.clone()))
        .map(|icon_path| icon_path.to_string_lossy().into_owned());

    let icon: Option<String> = match provided_icon {
        Some(icon) => Some(icon),
        _ => {
            let temp_icon = get_output_elf_path_as(artifact, "jpg");
            std::fs::write(temp_icon.clone(), DEFAULT_NRO_ICON)
                .expect("Failed to save temporary default icon file");

            Some(temp_icon.to_string_lossy().into_owned())
        }
    };

    Nxo::from_elf(elf.to_str().unwrap())
        .unwrap()
        .write_nro(
            &mut File::create(nro.clone()).unwrap(),
            romfs,
            icon.as_deref(),
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
        let npdm_json_path = root.join(npdm_json);
        Npdm::from_json(&npdm_json_path).unwrap()
    } else if let Some(npdm) = metadata.npdm {
        npdm
    } else {
        panic!("No npdm specified")
    };

    let mut option = OpenOptions::new();
    let output_option = option.write(true).create(true).truncate(true);
    let mut out_file = output_option
        .open(main_npdm.clone())
        .map_err(|err| (err, main_npdm.clone()))
        .unwrap();
    npdm.into_npdm(&mut out_file, AcidBehavior::Empty).unwrap();

    Nxo::from_elf(elf.to_str().unwrap())
        .unwrap()
        .write_nso(&mut File::create(main_exe).unwrap())
        .unwrap();

    let mut nsp = Pfs0::from_directory(exefs_dir.to_str().unwrap()).unwrap();
    let mut option = OpenOptions::new();
    let output_option = option.write(true).create(true).truncate(true);
    nsp.write_pfs0(
        &mut output_option
            .open(exefs_nsp.clone())
            .map_err(|err| (err, exefs_nsp.clone()))
            .unwrap(),
    )
    .map_err(|err| (err, exefs_nsp.clone()))
    .unwrap();

    println!("Built {}", exefs_nsp.to_string_lossy());
}
