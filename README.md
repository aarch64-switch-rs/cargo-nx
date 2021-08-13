# cargo-nx

## Introduction

`cargo-nx` is a simple build command for projects based on this organization's libraries and tools, to avoid having to mess with makefiles, scripts or having several copies of the same linker/target files for every single project, while also including support for generating various formats after projects are compiled.

## Installation

Assuming you have `cargo` installed, it's just a matter two steps.

1) Install `xargo`:

    ```bash
    cargo install xargo
    ```

2) Install `cargo-nx`:

    ```bash
    cargo install cargo-nx --git https://github.com/aarch64-switch-rs/cargo-nx
    ```

## Usage

First of all, the program can be executed as `cargo-nx` or simply as a cargo subcommand, `cargo nx`.

The only mandatory input is the build profile, which can be either `dev` or `release`.

Other optional parameters/flags:

- `-p <path>`, `--path=<path>`: Specifies a path with a crate to build (containing `Cargo.toml`, etc.), since the current directory is used by default otherwise.

- `-tp <triple>`, `--triple=<triple>`: Specifies the target triple, which is "aarch64-none-elf" by default.

- `-ctg`, `--use-custom-target`: Notifies the program to not use the default target JSON/linker script, which can be used to use custom ones.

- `-v`, `--verbose`: Displays extra information during the build process.

### Build formats

Build format fields used for building must be placed placed inside `[package.metadata.nx.<format>]` in `Cargo.toml`. These fields vary depending on the project's format.

The program itself detects the target format when parsing `Cargo.toml`. Note that multiple formats at the same time are not currently supported.

#### NRO

Projects which generate homebrew NRO binaries don't have any mandatory fields, but only optional ones.

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

> Note: the `romfs` and `icon` fields must point to items located relative to the project's directory

The fields present on the `nacp` object, all of them optional, are the following:

| Field             | Description                                       | Default value              |
|:----------------- |:-------------------------------------------------:| --------------------------:|
| name              | The application name                              | Unknown Application        |
| author            | The application author                            | Unknown Author             |
| version           | The application version                           | 1.0.0                      |
| title_id          | The application ID                                | 0000000000000000           |
| dlc_base_title_id | The base ID of all the application's DLC          | title_id + 0x1000          |
| lang (object)     | Different names/authors depending of the language | values above for all langs |

| Language codes      | Corresponding names    |
|:-------------------:|:----------------------:|
| en-US               | American English       |
| en-GB               | British English        |
| ja                  | Japanese               |
| fr                  | French                 |
| de                  | German                 |
| es-419              | Latin-American Spanish |
| es                  | Spanish                |
| it                  | Italian                |
| nl                  | Dutch                  |
| fr-CA               | Canadian French        |
| pt                  | Portuguese             |
| ru                  | Russian                |
| ko                  | Korean                 |
| zh-TW               | Chinese (Traditional)  |
| zh-CN               | Chinese (Simplified)   |

- Example with specific languages:

```toml
[package]
name = "Multi-language"
version = "0.2.0"
authors = ["XorTroll"]
edition = "2018"

[package.metadata.nx.nro]
nacp = { name = "A", author = "B", version = "0.2 beta", lang = { ja = { name = "J" }, es = { author = "X" }, it = { name = "I", author = "T" } } }
```

```bash
Names/authors produced above:

- Japanese: "J", "B"
- Spanish: "A", "X"
- Italian: "I", "T"
- Other languages: "A", "B"
```

> Note: therefore, only `name` and `author` fields can be language-specific

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

> Note: the NPDM JSON file follows the same format used in most other homebrews (check projects like [Atmosphere](https://github.com/Atmosphere-NX/Atmosphere/blob/master/stratosphere/sm/sm.json), [emuiibo](https://github.com/XorTroll/emuiibo/blob/master/emuiibo/npdm.json), [ldn_mitm](https://github.com/spacemeowx2/ldn_mitm/blob/master/ldn_mitm/res/app.json)...) and, like with the paths in the NRO format, it must be relative to the project's directory

## Credits

- [linkle](https://github.com/MegatonHammer/linkle) libraries as the core element of this project
- [cargo-count](https://github.com/kbknapp/cargo-count) as the example followed to make a cargo subcommand project