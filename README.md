# Lover
[Love2D](https://love2d.org/) is a open source command line build system and runner for Love2D projects inspired by Cargo.

[See the wiki](https://github.com/Wolfyxon/lover/wiki).

> [!NOTE]
> This tool is in early development, some of the described features may not work yet.
> 
> [See the project's progress](https://github.com/Wolfyxon/lover/issues/1).

## Features
### Easy cross-platform building
You can easily build your game for all supported platforms with a single command.

### Automatic dependency management
Love binaries required for building are downloaded automatically and can easily be managed by using Lover commands.

### Finally a good `run` command
When using `lover run` you can pass arguments to your game and even --flags.

Most tools like **Makefile** and **Cargo** will complain when the user specifies an unknown flag.

### Simple command line interface
Lover has a simple and easy to use command syntax (at least I hope).

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
Also a single script setup for a large project is not a good idea (ignore my mess). Also this tool does not require installing as much software as Love2D Universal and has a nice error handling and warnings.

This tool is also very similar to [Cargo](https://github.com/rust-lang/cargo/) which manages Rust projects.
