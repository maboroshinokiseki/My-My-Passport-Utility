use std::{
    fs::OpenOptions,
    io::{stdin, stdout, Read, Write},
    path::PathBuf,
};

use clap::{Parser, ValueEnum};

use libscsi::command::TestResult;
use wd_vsc::{
    device_configuration_page, operations_page, password_utility::*, power_condition_mode_page,
    security_block::*, Cipher, Error, SecurityStatus, WdVsc, DEFAULT_ITERATION_COUNT, DEFAULT_SALT,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// My passport device
    #[arg(short, long)]
    device: Option<PathBuf>,

    /// Show device info
    #[arg(short, long, requires = "device")]
    info: bool,

    /// Set or get password hint
    #[arg(long, requires = "device")]
    hint: Option<Option<String>>,

    /// Unlock the device with the given password
    #[arg(
        short,
        long,
        group = "crypto",
        group = "urg",
        value_name = "PASSWORD",
        requires = "device"
    )]
    unlock: Option<String>,

    /// Iteration count for unlocking or removing password (optional)
    #[arg(long, requires = "urg")]
    iteration_count: Option<u32>,

    /// Salt for unlocking or removing password, the max is 4 character (8 bytes) long (optional)
    #[arg(long, requires = "urg")]
    salt: Option<String>,

    /// Set a new password
    #[arg(
        short,
        long,
        group = "crypto",
        value_name = "CURRENT PASSWORD",
        requires = "device"
    )]
    set_password: Option<String>,

    /// Provide old password while setting new passwords if needed
    #[arg(short, long, requires = "set_password", value_name = "OLD PASSWORD")]
    old_password: Option<String>,

    /// New iteration count (optional).
    /// WD software will either ignore or mess with this value.
    /// Don't use WD software if you have set a custom value.
    #[arg(long, requires = "set_password", requires = "i_know_what_i_am_doing")]
    new_iteration_count: Option<u32>,

    /// Old iteration count (optional)
    #[arg(long, requires = "set_password")]
    old_iteration_count: Option<u32>,

    /// New salt, the max is 4 character (8 bytes) long (optional).
    /// WD software will either ignore or mess with this value.
    /// Don't use WD software if you have set a custom value.
    #[arg(long, requires = "set_password", requires = "i_know_what_i_am_doing")]
    new_salt: Option<String>,

    /// Old salt, the max is 4 character (8 bytes) long (optional)
    #[arg(long, requires = "set_password")]
    old_salt: Option<String>,

    /// Remove current password
    #[arg(
        short,
        long,
        group = "crypto",
        group = "urg",
        value_name = "PASSWORD",
        requires = "device"
    )]
    remove_password: Option<String>,

    /// Erase both password and content, not recoverable
    #[arg(
        short,
        long,
        group = "crypto",
        group = "eg",
        requires = "i_know_what_i_am_doing",
        requires = "device"
    )]
    erase: bool,

    /// The device may erase data by itself, that's not controllable.
    /// This option is meaningless, since data is still been encrypted.
    /// You'll not be able to get the original data without a userkey and a disk internal key.
    #[arg(short, long, requires = "erase")]
    preserve_data: bool,

    /// Set new encryption cipher
    #[arg(short, long, requires = "eg")]
    cipher: Option<Cipher>,

    /// Force it to do some dangerous things
    #[arg(long)]
    i_know_what_i_am_doing: bool,

    /// Generate password blob. Specify output argument to output to a file. If output argument is not specified, it'll output to stdout.
    #[arg(
        short,
        long,
        group = "crypto",
        group = "urg",
        group = "eg",
        value_name = "PASSWORD"
    )]
    generate_password_blob: Option<String>,

    /// Output password blob
    #[arg(long, requires = "generate_password_blob", value_name = "PATH")]
    output: Option<PathBuf>,

    /// Unlock the device with a password from the given path. If no path was specified, then it'll try to read from stdin
    #[arg(
        short = 'U',
        long,
        group = "crypto",
        group = "urg",
        value_name = "PATH",
        requires = "device"
    )]
    unlock_with_password_blob: Option<Option<PathBuf>>,

    /// Get or set virtual cdrom on or off
    #[arg(long, requires = "device")]
    virtual_cd: Option<Option<Switch>>,

    /// Get or set led brightness, 0 means off, 255 means on, some model may support something middle
    #[arg(long, requires = "device")]
    led_brightness: Option<Option<u8>>,

    /// Get or set sleep timer (in seconds), 0 means don't go sleep, value may be rounded
    #[arg(long, requires = "device")]
    sleep_timer: Option<Option<u32>>,

    /// Very minimum self diagnostic
    #[arg(long, requires = "device")]
    self_test: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Switch {
    On,
    Off,
}

fn main() -> wd_vsc::Result<()> {
    let cli = Cli::parse();

    let salt = create_salt_blob(cli.salt)?;
    let new_salt = create_salt_blob(cli.new_salt)?;
    let old_salt = create_salt_blob(cli.old_salt)?;

    let (device, status) = match cli.device {
        Some(path) => {
            let device = libscsi::Scsi::new(&path, None)?;

            let product_name = device.inquiry_product_identification()?;
            if !product_name.to_lowercase().contains("my passport") && !cli.i_know_what_i_am_doing {
                return Err(wd_vsc::Error::Other(
                    "This device doesn't seem like a my passport device. \
        Use --i-know-what-i-am-doing flag if you wish to continue."
                        .to_owned(),
                ));
            }

            let status = device.encryption_status()?;

            (Some(device), Some(status))
        }
        None => (None, None),
    };

    if let Some(password) = cli.generate_password_blob {
        let (salt, iteration_count) = unwrap_salt_and_iteration_count(
            device.as_ref(),
            salt,
            cli.iteration_count,
            Some("Warring: Failed to read salt from disk, will use the default value."),
            Some("Warring: Failed to read iteration count from disk, will use the default value."),
        );

        let cipher = cli
            .cipher
            .or_else(|| status.as_ref().map(|s| s.current_cipher))
            .ok_or_else(|| {
                wd_vsc::Error::Other("You'll need to provide a cipher or a device path".to_owned())
            })?;

        let blob = create_password_blob(cipher, &salt, iteration_count, &password)?;
        if let Some(output_path) = cli.output {
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(output_path)
                .unwrap();

            file.write_all(&blob)?;
        } else {
            stdout().write_all(&blob)?;
        }

        // if device is none, that means nothing else depends on device path.
        if device.is_none() {
            return Ok(());
        }
    }

    let (device, status) = (device.unwrap(), status.unwrap());

    if cli.info {
        println!("Device status: {:?}", status.security_status);
        println!("Current cipher: {:?}", status.current_cipher);
        println!("Supported ciphers: {:?}", status.supported_ciphers);

        let security_block = read_security_block(&device);
        match security_block {
            Ok(s) => {
                println!("Iteration count: {}", s.iteration_count);
                let (h, b, t) = unsafe { s.salt.align_to::<u16>() };
                let text = if h.is_empty() && t.is_empty() {
                    let mut text = "(".to_owned();
                    text.push_str(&String::from_utf16_lossy(b));
                    text.push(')');

                    text
                } else {
                    String::new()
                };
                println!("Salt: {:?} {}", s.salt, text);
                println!("Hint: {}", s.hint);
            }
            Err(_) => {
                println!("Iteration count (default): {}", DEFAULT_ITERATION_COUNT);
                let (h, b, t) = unsafe { DEFAULT_SALT.align_to::<u16>() };
                let text = if h.is_empty() && t.is_empty() {
                    let mut text = "(".to_owned();
                    text.push_str(&String::from_utf16_lossy(b));
                    text.push(')');

                    text
                } else {
                    String::new()
                };
                println!("Salt (default): {:?} {}", DEFAULT_SALT, text);
            }
        }
    }

    if let Some(hint) = &cli.hint {
        match hint {
            Some(_) => {
                if cli.set_password.is_none() {
                    Err(wd_vsc::Error::Other(
                        "Use --set-password if you want to set hint info".to_owned(),
                    ))?
                }
            }
            None => {
                let (block, _) = read_security_block_or_default(&device);
                println!("{}", block.hint);
            }
        }
    }

    if let Some(password) = cli.unlock {
        let (salt, iteration_count) = unwrap_salt_and_iteration_count(
            Some(&device),
            salt,
            cli.iteration_count,
            Some("Warring: Failed to read salt from disk, will use the default value."),
            Some("Warring: Failed to read iteration count from disk, will use the default value."),
        );

        let password_blob =
            create_password_blob(status.current_cipher, &salt, iteration_count, &password)?;
        device.unlock_encryption(password_blob)?;
    }

    if cli.set_password.is_some() {
        change_password(
            &device,
            &status,
            cli.set_password,
            cli.old_password,
            cli.hint.clone().unwrap_or_default(),
            cli.new_iteration_count,
            new_salt,
            cli.old_iteration_count,
            old_salt,
        )?;
    }

    if cli.remove_password.is_some() {
        change_password(
            &device,
            &status,
            None,
            cli.remove_password,
            cli.hint.unwrap_or_default(),
            None,
            None,
            cli.iteration_count,
            salt,
        )?;
    }

    if cli.erase {
        if let Some(cipher) = cli.cipher {
            if !status.supported_ciphers.contains(&cipher) {
                Err(wd_vsc::Error::UnsupportedCipher)?
            }
        }

        // For getting a most recent key_reset_enabler
        let status = device.encryption_status()?;

        device.reset_data_encryption_key(
            cli.cipher.unwrap_or(status.current_cipher),
            status.key_reset_enabler,
        )?;

        if !cli.preserve_data {
            let support_unmap = device.inquiry_unmap_support()?;
            let max_unmap_block = device.inquiry_unmap_block_limit()?;
            if support_unmap && max_unmap_block > 0 {
                let cap = device.read_capacity16()?;
                device.unmap(0, cap.logical_block_count, max_unmap_block)?;
            }
        }
    }

    if let Some(path) = cli.unlock_with_password_blob {
        match path {
            Some(path) => {
                let mut file = OpenOptions::new().read(true).open(path).unwrap();
                let mut blob = vec![];
                file.read_to_end(&mut blob)?;
                device.unlock_encryption(blob)?;
            }
            None => {
                let mut blob = vec![];
                stdin().read_to_end(&mut blob)?;
                device.unlock_encryption(blob)?;
            }
        }
    }

    if let Some(virtual_cd) = cli.virtual_cd {
        match virtual_cd {
            Some(switch) => {
                check_device_unlocked(
                    &status.security_status,
                    "Device need to be unlocked in order to change virtual cd status",
                )?;

                let enable_cd = match switch {
                    Switch::On => true,
                    Switch::Off => false,
                };

                device_configuration_page::set_virtual_cd_status(&device, enable_cd)?;
            }
            None => {
                let status = device_configuration_page::get_virtual_cd_status(&device)?;
                print!("virtual cd status: ");
                match status {
                    true => println!("On"),
                    false => println!("Off"),
                }
            }
        }
    }

    if let Some(led_brightness) = cli.led_brightness {
        match led_brightness {
            Some(led_brightness) => {
                check_device_unlocked(
                    &status.security_status,
                    "Device need to be unlocked in order to change led brightness",
                )?;

                operations_page::set_led_brightness(&device, led_brightness)?;
            }
            None => {
                println!(
                    "led brightness: {}",
                    operations_page::get_led_brightness(&device)?
                );
            }
        }
    }

    if let Some(sleep_timer) = cli.sleep_timer {
        match sleep_timer {
            Some(sleep_timer) => {
                check_device_unlocked(
                    &status.security_status,
                    "Device need to be unlocked in order to change sleep timer",
                )?;
                power_condition_mode_page::set_sleep_timer(&device, sleep_timer)?;
            }
            None => {
                let sleep_timer = power_condition_mode_page::get_sleep_timer(&device)?;
                print!("sleep timer: ");
                match sleep_timer {
                    0 => println!("0 (disabled)"),
                    1 => println!("1 second"),
                    timer => println!("{} seconds", timer),
                }
            }
        }
    }

    if cli.self_test {
        match device.send_diagnostic() {
            TestResult::Ok => println!("Ok"),
            TestResult::HardwareError => println!("Hardware Error!"),
            TestResult::Other(e) => Err(e)?,
        }
    }

    Ok(())
}

fn check_device_unlocked(status: &SecurityStatus, error_message: &str) -> wd_vsc::Result<()> {
    match status {
        SecurityStatus::Locked | SecurityStatus::UnlockAttemptExceeded => {
            Err(Error::NotUnlocked(error_message.to_owned()))?
        }
        _ => Ok(()),
    }
}
