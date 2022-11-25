#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod gui_error;

use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    sync::Mutex,
    vec,
};

use serde::Serialize;
use tauri::{Manager, State};

use gui_error::Result;
use libscsi::{command::TestResult, Scsi};
use wd_vsc::{
    device_configuration_page, operations_page, password_utility, power_condition_mode_page,
    security_block::read_security_block, WdVsc,
};

struct Storage {
    device: Mutex<Option<Scsi>>,
}

#[derive(Serialize)]
struct PathAndName {
    pub path: PathBuf,
    pub name: String,
}

fn get_root_with_file(path: &Path, filename: &str) -> Result<PathBuf> {
    let mut path = path.canonicalize()?;
    loop {
        if path.join(filename).exists() {
            break;
        } else {
            path = match path.parent() {
                Some(parent) => parent.to_owned(),
                None => Err(io::Error::new(io::ErrorKind::NotFound, ""))?,
            };
        }
    }

    Ok(path)
}

#[tauri::command]
fn list_drives() -> Result<Vec<PathAndName>> {
    let mut result = HashMap::new();
    let paths = fs::read_dir("/sys/dev/block/")?;
    for path in paths {
        // a naive way to determine if it's a partition or a drive.
        let path = &path?.path();
        let block_path = match get_root_with_file(path, "removable") {
            Ok(p) => p,
            Err(_) => continue,
        };
        let product_path = get_root_with_file(path, "model");
        let filename = block_path.file_name().unwrap();
        if filename.to_string_lossy().starts_with("sd") {
            let product_name = match product_path {
                Ok(path) => fs::read_to_string(path.join("model"))?,
                Err(_) => "".to_owned(),
            };
            result.insert(PathBuf::from("/dev/").join(filename), product_name);
        }
    }

    let mut result: Vec<_> = result
        .iter()
        .map(|r| PathAndName {
            path: r.0.to_owned(),
            name: r.1.to_owned(),
        })
        .collect();

    result.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(result)
}

#[tauri::command]
fn open_device(path: String, storage: State<Storage>) -> Result<()> {
    let device = Scsi::new(&path, None)?;
    let name = device.inquiry_product_identification()?;
    if !name.to_lowercase().contains("my passport") {
        Err(wd_vsc::Error::Other(
            "This device doesn't seem like a my passport device.".to_owned(),
        ))?
    }

    let _ = storage.device.lock().unwrap().insert(device);

    Ok(())
}

#[tauri::command]
fn current_device(storage: State<Storage>) -> Result<PathBuf> {
    let guard = storage.device.lock().unwrap();
    match guard.as_ref() {
        Some(device) => Ok(device.path().to_owned()),
        None => Ok(PathBuf::new()),
    }
}

#[tauri::command]
fn get_security_status(storage: State<Storage>) -> Result<String> {
    let device = storage.device.lock().unwrap();
    let device = device.as_ref().unwrap();

    Ok(format!("{:?}", device.encryption_status()?.security_status))
}

#[tauri::command]
fn get_hint(storage: State<Storage>) -> Result<String> {
    let device = storage.device.lock().unwrap();
    let device = device.as_ref().unwrap();
    let security_block = read_security_block(device)?;

    Ok(security_block.hint)
}

#[tauri::command]
fn unlock_device(password: String, storage: State<Storage>) -> Result<()> {
    let device = storage.device.lock().unwrap();
    let device = device.as_ref().unwrap();
    let security_block = read_security_block(device)?;
    let status = device.encryption_status()?;

    let password_blob = password_utility::create_password_blob(
        status.current_cipher,
        &security_block.salt,
        security_block.iteration_count,
        &password,
    )?;

    device.unlock_encryption(password_blob)?;

    Ok(())
}

#[tauri::command]
fn set_password(password: String, hint: String, storage: State<Storage>) -> Result<()> {
    let device = storage.device.lock().unwrap();
    let device = device.as_ref().unwrap();
    let status = device.encryption_status()?;
    password_utility::change_password(
        device,
        &status,
        Some(password),
        None,
        Some(hint),
        None,
        None,
        None,
        None,
    )?;

    Ok(())
}

#[tauri::command]
fn remove_password(password: String, storage: State<Storage>) -> Result<()> {
    let device = storage.device.lock().unwrap();
    let device = device.as_ref().unwrap();
    let status = device.encryption_status()?;
    password_utility::change_password(
        device,
        &status,
        None,
        Some(password),
        None,
        None,
        None,
        None,
        None,
    )?;

    Ok(())
}

#[tauri::command]
fn change_password(
    current_password: String,
    new_password: String,
    hint: String,
    storage: State<Storage>,
) -> Result<()> {
    let device = storage.device.lock().unwrap();
    let device = device.as_ref().unwrap();
    let status = device.encryption_status()?;
    password_utility::change_password(
        device,
        &status,
        Some(new_password),
        Some(current_password),
        Some(hint),
        None,
        None,
        None,
        None,
    )?;

    Ok(())
}

#[tauri::command]
fn basic_diagnose(storage: State<Storage>) -> Result<String> {
    let device = storage.device.lock().unwrap();
    let device = device.as_ref().unwrap();
    match device.send_diagnostic() {
        TestResult::Ok => Ok("Everything is okay.".to_owned()),
        TestResult::HardwareError => Ok("Hardware error!".to_owned()),
        TestResult::Other(e) => Err(e)?,
    }
}

#[tauri::command]
fn get_sleep_timer(storage: State<Storage>) -> Result<u32> {
    let device = storage.device.lock().unwrap();
    let device = device.as_ref().unwrap();

    Ok(power_condition_mode_page::get_sleep_timer(device)?)
}

#[tauri::command]
fn set_sleep_timer(timer: u32, storage: State<Storage>) -> Result<()> {
    let device = storage.device.lock().unwrap();
    let device = device.as_ref().unwrap();
    power_condition_mode_page::set_sleep_timer(device, timer)?;

    Ok(())
}

#[tauri::command]
fn get_led_state(storage: State<Storage>) -> Result<bool> {
    let device = storage.device.lock().unwrap();
    let device = device.as_ref().unwrap();

    Ok(operations_page::get_led_brightness(device)? != 0)
}

#[tauri::command]
fn set_led_state(on: bool, storage: State<Storage>) -> Result<()> {
    let device = storage.device.lock().unwrap();
    let device = device.as_ref().unwrap();
    match on {
        true => operations_page::set_led_brightness(device, 255)?,
        false => operations_page::set_led_brightness(device, 0)?,
    }

    Ok(())
}

#[tauri::command]
fn get_vcd_state(storage: State<Storage>) -> Result<bool> {
    let device = storage.device.lock().unwrap();
    let device = device.as_ref().unwrap();

    Ok(device_configuration_page::get_virtual_cd_status(device)?)
}

#[tauri::command]
fn set_vcd_state(on: bool, storage: State<Storage>) -> Result<()> {
    let device = storage.device.lock().unwrap();
    let device = device.as_ref().unwrap();
    device_configuration_page::set_virtual_cd_status(device, on)?;

    Ok(())
}

#[tauri::command]
fn erase_device(storage: State<Storage>) -> Result<()> {
    let device = storage.device.lock().unwrap();
    let device = device.as_ref().unwrap();
    let status = device.encryption_status()?;
    device.reset_data_encryption_key(status.current_cipher, status.key_reset_enabler)?;

    let support_unmap = device.inquiry_unmap_support()?;
    let max_unmap_block = device.inquiry_unmap_block_limit()?;
    if support_unmap && max_unmap_block > 0 {
        let cap = device.read_capacity16()?;
        device.unmap(0, cap.logical_block_count, max_unmap_block)?;
    }

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(Storage {
            device: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            list_drives,
            open_device,
            current_device,
            get_security_status,
            get_hint,
            unlock_device,
            set_password,
            remove_password,
            change_password,
            basic_diagnose,
            get_sleep_timer,
            set_sleep_timer,
            get_led_state,
            set_led_state,
            get_vcd_state,
            set_vcd_state,
            erase_device
        ])
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
