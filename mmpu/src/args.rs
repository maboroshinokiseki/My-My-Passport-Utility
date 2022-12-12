use std::path::PathBuf;

use clap::{Parser, ValueEnum, ValueHint};

use wd_vsc::Cipher;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// My passport device
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    pub device: Option<PathBuf>,

    /// Show device info
    #[arg(short, long, requires = "device")]
    pub info: bool,

    /// Set or get password hint
    #[arg(long, requires = "device")]
    pub hint: Option<Option<String>>,

    /// Unlock the device with the given password
    #[arg(
        short,
        long,
        group = "crypto",
        group = "urg",
        value_name = "PASSWORD",
        requires = "device"
    )]
    pub unlock: Option<String>,

    /// Iteration count for unlocking or removing password (optional)
    #[arg(long, requires = "urg")]
    pub iteration_count: Option<u32>,

    /// Salt for unlocking or removing password, the max is 4 character (8 bytes) long (optional)
    #[arg(long, requires = "urg")]
    pub salt: Option<String>,

    /// Set a new password
    #[arg(
        short,
        long,
        group = "crypto",
        value_name = "CURRENT PASSWORD",
        requires = "device"
    )]
    pub set_password: Option<String>,

    /// Provide old password while setting new passwords if needed
    #[arg(short, long, requires = "set_password", value_name = "OLD PASSWORD")]
    pub old_password: Option<String>,

    /// New iteration count (optional).
    /// WD software will either ignore or mess with this value.
    /// Don't use WD software if you have set a custom value.
    #[arg(long, requires = "set_password", requires = "i_know_what_i_am_doing")]
    pub new_iteration_count: Option<u32>,

    /// Old iteration count (optional)
    #[arg(long, requires = "set_password")]
    pub old_iteration_count: Option<u32>,

    /// New salt, the max is 4 character (8 bytes) long (optional).
    /// WD software will either ignore or mess with this value.
    /// Don't use WD software if you have set a custom value.
    #[arg(long, requires = "set_password", requires = "i_know_what_i_am_doing")]
    pub new_salt: Option<String>,

    /// Old salt, the max is 4 character (8 bytes) long (optional)
    #[arg(long, requires = "set_password")]
    pub old_salt: Option<String>,

    /// Remove current password
    #[arg(
        short,
        long,
        group = "crypto",
        group = "urg",
        value_name = "PASSWORD",
        requires = "device"
    )]
    pub remove_password: Option<String>,

    /// Erase both password and content, not recoverable
    #[arg(
        short,
        long,
        group = "crypto",
        group = "eg",
        requires = "i_know_what_i_am_doing",
        requires = "device"
    )]
    pub erase: bool,

    /// The device may erase data by itself, that's not controllable.
    /// This option is meaningless, since data is still been encrypted.
    /// You'll not be able to get the original data without a userkey and a disk internal key.
    #[arg(short, long, requires = "erase")]
    pub preserve_data: bool,

    /// Set new encryption cipher
    #[arg(short, long, requires = "eg")]
    pub cipher: Option<Cipher>,

    /// Force it to do some dangerous things
    #[arg(long)]
    pub i_know_what_i_am_doing: bool,

    /// Generate password blob. Specify output argument to output to a file. If output argument is not specified, it'll output to stdout.
    #[arg(
        short,
        long,
        group = "crypto",
        group = "urg",
        group = "eg",
        value_name = "PASSWORD"
    )]
    pub generate_password_blob: Option<String>,

    /// Output password blob
    #[arg(long, requires = "generate_password_blob", value_name = "PATH", value_hint = ValueHint::FilePath)]
    pub output: Option<PathBuf>,

    /// Unlock the device with a password from the given path. If no path was specified, then it'll try to read from stdin
    #[arg(
        short = 'U',
        long,
        group = "crypto",
        group = "urg",
        value_name = "PATH",
        requires = "device",
        value_hint = ValueHint::FilePath
    )]
    pub unlock_with_password_blob: Option<Option<PathBuf>>,

    /// Get or set virtual cdrom on or off
    #[arg(long, requires = "device")]
    pub virtual_cd: Option<Option<Switch>>,

    /// Get or set led brightness, 0 means off, 255 means on, some model may support something middle
    #[arg(long, requires = "device")]
    pub led_brightness: Option<Option<u8>>,

    /// Get or set sleep timer (in seconds), 0 means don't go sleep, value may be rounded
    #[arg(long, requires = "device")]
    pub sleep_timer: Option<Option<u32>>,

    /// Very minimum self diagnostic
    #[arg(long, requires = "device")]
    pub self_test: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Switch {
    On,
    Off,
}
