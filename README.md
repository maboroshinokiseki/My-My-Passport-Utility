# My My Passport Utility
An unofficial WD My Passport utility

# How to Build
## for CLI
just run `cargo build --package mmpu`.
## for GUI
Check [Tauri guid](https://tauri.app/v1/guides/getting-started/prerequisites#setting-up-linux) for prerequisites.
After that, run `cargo tauri build`.

# GUI Basic Usage
![Password](https://user-images.githubusercontent.com/15065470/204031153-be78a688-7917-45e0-a79a-2ee899c92648.png)
![Diagnose](https://user-images.githubusercontent.com/15065470/204031232-b5720eea-8130-44d0-9bc5-30a0025a57ca.png)
![Settings](https://user-images.githubusercontent.com/15065470/204031259-218fc686-bf7e-40c2-af2f-bb463443de62.png)
![Erase](https://user-images.githubusercontent.com/15065470/204031267-98f3663d-7a58-40f7-861d-96cc0a9c89ff.png)

# CLI Basic Usage
Unlock a drive

`
sudo mmpu --device /dev/sda --unlock pass
`

Set a password for a drive

`
sudo mmpu --device /dev/sda --set-password pass
`

# Credit
[WD-Decrypte](https://github.com/SofianeHamlaoui/WD-Decrypte)
