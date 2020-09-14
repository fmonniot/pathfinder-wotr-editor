# Pathfinder Wrath of the Righteous Editor

An editor for your save games, because sometime you don't want to play by the rules.

## Using the application


### Pre-compiled

Download the latest binary from the [release section](https://github.com/fmonniot/pathfinder-wotr-editor/releases). We currently build only the Windows version, as the game isn't released on other platform as of now.

### From source

If you prefer to compile the application yourself, you should have a working rust environment, clone this repository and then use `cargo build --release` (we do not recommend using the debug version, as it is much slower).

### Usage

Once you have the binary, you can simply execute it to open the application. You'll be asked to select a save game.

If you already know the path to the save game, you can bypass the save selection screen and directly open it by using `pathfinder-wotr-editor /path/to/save.zks`.

If you encounter a bug and want to report it here, please run the executable with the logs enabled.

With bash:

```
RUST_LOG=pathfinder_wotr_editor=debug pathfinder-wotr-editor
```

Or with PowerShell:

```
$env:RUST_LOG="pathfinder_wotr_editor=debug"
pathfinder-wotr-editor
```
