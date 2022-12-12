use std::{
    fs::OpenOptions,
    io::{stdin, stdout, Read, Write},
    path::PathBuf,
};

use clap::Parser;
use libscsi::command::TestResult;
use wd_vsc::{
    device_configuration_page, operations_page, password_utility::*, power_condition_mode_page,
    security_block::*, Error, SecurityStatus, WdVsc, DEFAULT_ITERATION_COUNT, DEFAULT_SALT,
};
mod args;
use args::*;

fn main() -> wd_vsc::Result<()> {
    let cli = Cli::parse();

    let salt = create_salt_blob(cli.salt)?;
    let new_salt = create_salt_blob(cli.new_salt)?;
    let old_salt = create_salt_blob(cli.old_salt)?;

    let (device, status) = match cli.device {
        Some(path) => {
            let path = if cfg!(target_os = "windows") {
                // A small QOL for Windows users
                let path_str = path.to_string_lossy().as_ref().to_owned();
                let chars: Vec<char> = path_str.chars().collect();

                let is_drive_root = (chars.len() == 1 && chars[0].is_ascii_alphabetic())
                    || (chars.len() == 2 && chars[0].is_ascii_alphabetic() && chars[1] == ':');

                if is_drive_root {
                    PathBuf::from(format!("\\\\.\\{}:", chars[0]))
                } else {
                    path
                }
            } else {
                path
            };

            let device = libscsi::Scsi::new(&path)?;

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
        match device.send_diagnostic()? {
            TestResult::Ok => println!("Ok"),
            TestResult::HardwareError => println!("Hardware Error!"),
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
