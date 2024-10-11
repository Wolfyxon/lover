# Lover
[Love2D](https://love2d.org/) runner and cross platform builder inspired by Cargo.

> [!NOTE]
> This tool is in early development, most of the described features may not work yet.

## Usage
### Getting started
Your project must contain a `lover.toml` file with the configuration, here's an example:
```toml
[package]
name = "Epic game"
description = "My cool game"
author = "me"
version = "1.0"
```
You can also initialize a new project with everything already set up using:
```
lover new myCoolProjectName
```

### Running
```
lover run
```

### Building
#### Your platform
```
lover build
```
#### Specific platform
```
lover build <platformName>
```
example:
```
lover build linux
```
#### All platforms from the config
```
lover build all
```
