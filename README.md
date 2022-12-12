# My My Passport Utility
An unofficial WD My Passport utility for both Linux and Windows

# How to Build
## CLI
Just run `cargo build --package mmpu`.
## GUI
Check [Tauri guid](https://tauri.app/v1/guides/getting-started/prerequisites#setting-up-linux) for prerequisites.
After that, run `cargo tauri build`.

# GUI Basic Usage
![GUI](https://user-images.githubusercontent.com/15065470/206982060-9a943ba6-1be5-4b4d-878c-bfcb47cc4d1a.gif)

# CLI Basic Usage
Unlock a drive

`
sudo mmpu --device /dev/sdx --unlock pass
`

Set a password for a drive

`
sudo mmpu --device /dev/sdx --set-password pass
`

For Windows users, use `\\.\physicaldrive0(1, 2, 3 etc.)` or `\\.\X:` as device path

# Credit
[WD-Decrypte](https://github.com/SofianeHamlaoui/WD-Decrypte)
