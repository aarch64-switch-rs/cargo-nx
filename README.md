# cargo nx

## Introduction

*cargo nx* is a simple build command for projects based on this organization's libraries and tools, to avoid having to mess with makefiles, scripts or having several copies of the same linker/target files for every single project, while also including support for generating various formats after projects are compiled.

## Installation

Assuming you have `cargo` installed, it's just a matter of running `cargo install cargo-nx --git https://github.com/aarch64-switch-rs/cargo-nx`
will install the `cargo-nx` executable, which can also be simply ran as a cargo command, `cargo nx`.

## Project formats

Extra fields used for building are placed inside `[package.metadata.sprinkle.<format>]` in Cargo.toml. These fields vary depending on the project's format:

### NRO

Projects which generate homebrew NRO binaries don't need any mandatory fields/files, but can set optional ones.

- Example:

```toml
[package]
name = "Project"
version = "0.1.0"
authors = ["XorTroll"]
edition = "2018"

[package.metadata.nx.nro]
romfs = "romfs_dir"
icon = "icon.jpg"
nacp = { name = "Sample project", author = "XorTroll", version = "0.1 beta" }
```

> Note: the `romfs` and `icon` fields must point to items located in the same directory as Cargo.toml!

#### NACP fields

> Note: every fields are optional!

| Field             | Description                                      | Default value       |
| ----------------- |:------------------------------------------------:| -------------------:|
| name              | The application name.                            | Unknown Application |
| author            | The application author.                          | Unknown Author      |
| version           | The application version.                         | 1.0.0               |
| title_id          | The application title id.                        | 0000000000000000    |
| dlc_base_title_id | The base id of all the title DLC.                | title_id + 0x1000   |
| lang (object)     | Different name/author depending of the language  | use name and author |

| Supported Languages|
|:------------------:|
| en-US              |
| en-GB              |
| ja                 |
| fr                 |
| de                 |
| es-419             |
| es                 |
| it                 |
| nl                 |
| fr-CA              |
| pt                 |
| ru                 |
| ko                 |
| zh-TW              |
| zh-CN              |

- Example with specific languages:

```toml
[package]
name = "Multi-language"
version = "0.2.0"
authors = ["XorTroll"]
edition = "2018"

[package.metadata.nx.nro]
nacp = { name = "A", author = "B", version = "0.2 beta", lang = { ja = { name = "J" }, es = { author = "X" }, it = { name = "I", author = "T" } } }

# Result:
# - Japanese: "J", "B"
# - Spanish: "A", "X"
# - Italian: "I", "T"
# - Other languages: "A", "B"
```

> Note: only `name` and `author` can be language-specific, other parameters such as `titleid` or `version` are not!

### NSP

Projects which generate sysmodule NSP exefs packages need a single, mandatory field for the NPDM data:

- Example:

```toml
[package]
name = "Project"
version = "0.2.10"
authors = ["XorTroll"]
edition = "2018"

[package.metadata.nx.nsp]
npdm = "npdm.json"
```

> Note: the NPDM JSON file follows the same format used in most homebrews (check projects like Atmosphere, emuiibo, ldn_mitm...) and must be located in the same directory as Cargo.toml!

## Building

> Command: `sprinkle <format> [<optional-extra-cargo-arguments>]`

> Available formats (listed above): `nro`, `nsp`

Running this command will (among other minor details) run `xargo build` and, after building the project, will generate the specific files depending on the project build format.

A default target is used, whose JSON specs are included within sprinkle itself (check [here](/specs)). Support for custom targets is planned but not supported yet.

## Credits

- *linkle* project and its developers for the base of this fork