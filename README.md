# Lover
Lover is a open source command line build system and runner for [Love2D](https://love2d.org) projects inspired by Cargo.

[Wiki](https://github.com/Wolfyxon/lover/wiki) |
[Example project](https://github.com/Wolfyxon/lover-example) |
[CLI usage](https://github.com/Wolfyxon/lover/wiki/Using-Lover) |
[Constants](https://github.com/Wolfyxon/lover/wiki/Constants) |
[Downloads](https://github.com/Wolfyxon/lover/releases/latest) |

## Features
### Default environment variables
You can access certain constants like the game's version by the use of `os.getenv()`.
```lua
local version = os.getenv("LOVER_PKG_VERSION")
```
[learn more](https://github.com/Wolfyxon/lover/wiki/Constants)

### Easy cross-platform building
You can easily build your game for all supported platforms with a single command.

`lover build <platform>...`

Example:
```
lover build linux
```
```
lover build win32 win64 linux
```

### Automatic dependency management
Love binaries required for building are downloaded automatically and can easily be managed by using Lover commands.

- `lover install <name>` to install
- `lover uninstall <name>` to remove
- `lover dep <name>` to get details
- `lover dep` to list

Example:
```
lover install love-win64
```

## Supported platforms
- ✅ **Full support**: The platform is fully supported and should work. Treated with the highest priority
- 🟡 **Partial support**: The platform mostly works but you may encounter issues
- 📁 **Planned**: Support will be implemented in future
- ⭕ **Not yet needed**: The platform is not widely used. If you want support for it [you can open an issue](https://github.com/Wolfyxon/lover/issues/new).
- ❗ **Testers/maintainers needed**: someone is needed to test and/or maintain the platform
- ❌ **Impossible**: The platform is currently impossible to implement

### Build targets
| Name                | Arch   | Alias   | Status |
|---------------------|--------|---------|--------|
| Universal LOVE file |        | `love`  | ✅     |
| Linux AppImage      | x86_64 | `linux` | ✅     |
| Linux AppImage      | x86_32 |         | ❌     |
| Windows EXE         | x86_64 | `win64` | ✅     |
| Windows EXE         | x86_32 | `win32` | ✅     |
| MacOS               |        |         | ❗     |
| Web                 |        |         | 📁     |
| Android             |        |         | 📁     |
| Nintendo 3DS `3DSX` |        |         | 📁     |
| Nintendo 3DS `CIA`  |        |         | 📁     |
| Nintendo Wii U      |        |         | ❗     |
| Nintendo Switch     |        |         | ❗     |

Please also see [the compatibility matrix](https://github.com/Wolfyxon/lover/wiki/Building#support).

The `love` target is runnable on all platforms, but require [LÖVE](https://love2d.org/) to be installed.

### Lover tool
| Platform | Arch   | Status |
|----------|--------|--------|
| Linux    | x86_64 | ✅     |
| Linux    | x86_32 | ⭕     |
| Windows  | x86_64 | ✅     |
| Windows  | x86_32 | ⭕     |
| MacOS    |        | ❗     |

## Example outputs
(Note that this may not always be up to date)  
(Also normally this is colored)

`lover help`
```
Usage: lover <command> [<arguments>]... 

Lover is a open source cross-platform build system for Love2D projects.
https://github.com/Wolfyxon/lover

Available commands:

  help:      Shows help.
  version:   Shows the current Lover version.
  new:       Initializes a new Love2D project.
  create:    Runs an interactive project setup
  run:       Runs the game.
  parse:     Checks the validity of Lua scripts.
  build:     Packages the game.
  clean:     Removes compiled build files.
  target:    Lists or shows info of available build targets.
  dep:       Lists or shows info of available dependencies.
  install:   Installs dependencies.
  uninstall: Removes installed dependencies.
  fetch:     Fetches a dependency. Mostly for testing
  env:       Shows a list of available Lover constants and their values.
  module:    Shows the extra code injected into your game when building. Mostly for testing

Use `lover help <command>` to see the usage of a specific command.
For additional help, see the wiki: https://github.com/Wolfyxon/lover/wiki
```

`lover build win64 linux`
```
> Initializing build of: win64, linux
OK: All dependencies are installed.

> Building target: love
Warning: 'luac' not found. Skipping luac parse.
>> Archiving game assets                            [==============================] 1/1 
OK: Successfully built 'love' 

> Building target: win64
>> Extracting Windows Love2D files                  [==============================] 14/14 
>> Embedding game into the LOVE executable          [==============================] 0.763/0.763 KB
OK: The EXE should now be usable, even if something fails.
>> Converting icon to the ICO format
>> Applying info with RCEdit
OK: Successfully built 'win64' 

> Building target: linux
>> Embedding game into the LOVE executable          [==============================] 0.763/0.763 KB
>> Replacing the LOVE binary in the SquashFS
>> Embedding created SquashFS into the AppImage
OK: Successfully built 'linux' 

```

## Why?
I wanted to create a simple expandable and universal system for building, running and managing Love2D projects.

This is a replacement for my previous project [Love2D Universal](https://github.com/Wolfyxon/love2d-universal) which utilized a single Makefile, however a global system-wide tool written in a more advanced language like Rust is a way better approach.
A single script setup for a large project is not a good idea, as organization is not great for such big scripts and implementing a lot of advanced features is not easy. 
Also this tool does not require installing as much software as Love2D Universal and has nice error handling and warnings.

This tool is also very similar to [Cargo](https://github.com/rust-lang/cargo/) which manages Rust projects.

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
- `backhand`: Modifying, creating and parsing SquashFS
