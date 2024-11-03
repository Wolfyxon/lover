# Lover
Lover is a open source command line build system and runner for Love2D projects inspired by Cargo.

[Wiki](https://github.com/Wolfyxon/lover/wiki) | [CLI usage](https://github.com/Wolfyxon/lover/wiki/Using-Lover) | [Constants](https://github.com/Wolfyxon/lover/wiki/Constants)

## Features
### Easy cross-platform building
You can easily build your game for all supported platforms with a single command.

### Automatic dependency management
Love binaries required for building are downloaded automatically and can easily be managed by using Lover commands.

### Finally a good `run` command
When using `lover run` you can pass arguments to your game and even --flags.

Most tools like **Makefile** and **Cargo** will treat all flags as their own and not allow such things.

```
lover run someArgument --someFlag --gameMode=survival
```

### Simple command line interface
Lover has a simple and easy to use command syntax (at least I hope).

## Supported platforms
- ✅ **Full support**: The platform is fully supported and should work
- 🟡 **Partial support**: The platform mostly works but you may encounter issues
- 📁 **Planned**: Support will be implemented in future
- ⭕ **Not yet needed**: The platform is not widely used. If you want support for it [you can open an issue](https://github.com/Wolfyxon/lover/issues/new) and it will be implemented.
- ❗ **Testers/maintainers needed**: someone is needed to test and/or maintain the platform
- ❌ **Impossible**: The platform is currently impossible to implement

### Build targets
| Name                | Arch   | Alias   | Status |
|---------------------|--------|---------|--------|
| Universal LOVE file |        | `love`  | ✅     |
| Linux AppImage      | x86_64 | `linux` | ✅     |
| Linux AppImage      | x86_32 |         | ❌     |
| Windows             | x86_64 | `win64` | ✅     |
| Windows             | x86_32 | `win32` | ✅     |
| MacOS               |        |         | ❗     |
| Web                 |        |         | 📁     |
| Android             |        |         | 📁     |
| Nintendo 3DS `3DSX` |        |         | 📁     |
| Nintendo 3DS `CIA`  |        |         | 📁     |
| Nintendo Wii U      |        |         | ❗     |
| Nintendo Switch     |        |         | ❗     |

### Lover tool
| Platform | Arch   | Status |
|----------|--------|--------|
| Linux    | x86_64 | ✅     |
| Linux    | x86_32 | ⭕     |
| Windows  | x86_64 | ✅     |
| Windows  | x86_32 | ⭕     |
| MacOS    |        | ❗     |

## Compiling
Lover is written in **Rust** and managed by **Cargo**. 

Install Cargo on your system then open the terminal in the Lover's source directory and run:
```
cargo build
```
or
```
cargo run
```
to just run it.

Read [the documentation](https://doc.rust-lang.org/cargo/) for more info.

## Why?
I wanted to create a simple expandable and universal system for building, running and managing Love2D projects.

This is a replacement for my previous project [Love2D Universal](https://github.com/Wolfyxon/love2d-universal) which utilized a single Makefile, however a global system-wide tool is a way better approach.
A single script setup for a large project is not a good idea, as organization is not great for such big scripts and implementing a lot of advanced features is not easy. 
Also this tool does not require installing as much software as Love2D Universal and has a nice error handling and warnings.

This tool is also very similar to [Cargo](https://github.com/rust-lang/cargo/) which manages Rust projects.

## Used crates
- `reqwest`: Sending HTTP requests and downloading files
- `serde`: Serializing structs
- `serde_json`: JSON parsing and serde support
- `toml`: TOML parsing and serde support
- `zip`: Managing ZIP archives
- `image`: Handling image files
- `dirs`: Finding system directories on various platforms
- `regex`: Using regular expressions on strings
- `ansi_term`: Styling terminal output
