const { invoke } = window.__TAURI__.tauri;
const { message } = window.__TAURI__.dialog;
const { appWindow } = window.__TAURI__.window;

const SecurityStatus = {
  NoUserPassword: 'NoUserPassword',
  Locked: 'Locked',
  Unlocked: 'Unlocked',
  UnlockAttemptExceeded: 'UnlockAttemptExceeded',
  NoEncryption: 'NoEncryption'
};

async function list_drives() {
  return await invoke("list_drives");
}

async function open_device(path) {
  await invoke("open_device", { path: path });
}

async function current_device() {
  return await invoke("current_device");
}

async function get_security_status() {
  return await invoke("get_security_status");
}

async function get_hint() {
  return await invoke("get_hint");
}

async function unlock_device(password) {
  await invoke("unlock_device", { password: password });
}

async function set_password(password, hint) {
  await invoke("set_password", { password: password, hint: hint });
}

async function remove_password(password) {
  await invoke("remove_password", { password: password });
}

async function change_password(current_password, new_password, hint) {
  await invoke("change_password", { currentPassword: current_password, newPassword: new_password, hint: hint });
}

async function basic_diagnose() {
  let m = await invoke("basic_diagnose");
  await message(m);
}

async function get_sleep_timer() {
  return parseInt(await invoke("get_sleep_timer"), 10);
}

async function set_sleep_timer(timer) {
  await invoke("set_sleep_timer", { timer: parseInt(timer, 10) });
  await message("Success!");
}

async function get_led_state() {
  return await invoke("get_led_state");
}

async function set_led_state(state) {
  await invoke("set_led_state", { on: state });
  await message("Success!");
}

async function get_vcd_state() {
  return await invoke("get_vcd_state");
}

async function set_vcd_state(state) {
  await invoke("set_vcd_state", { on: state });
  await message("Success!");
}

async function erase_device() {
  await invoke("erase_device");
}

function toggle_block(visible, block) {
  if (visible) {
    block.classList.remove("d-none");
  } else {
    block.classList.add("d-none");
  }
}

async function main() {
  let drive_selection = document.querySelector("#drive-selection");
  let drives = await list_drives();
  for (const drive of drives) {
    var opt = document.createElement('option');
    opt.value = drive.path;
    let name = drive.name.trim();
    if (name == "") {
      opt.text = drive.path;
    }
    else {
      opt.text = drive.path + " (" + name + ")";
    }

    drive_selection.appendChild(opt);
  }

  drive_selection.addEventListener("change", async (e) => {
    try {
      await open_device(e.target.value);
      location.reload();
    } catch (error) {
      await message(error);
      location.reload();
    }
  })

  let password_tab = document.querySelector("#nav-password-tab");
  let diagnose_tab = document.querySelector("#nav-diagnose-tab");
  let settings_tab = document.querySelector("#nav-settings-tab");
  let erase_tab = document.querySelector("#nav-erase-tab");

  let device = await current_device();
  if (device == "") {
    return;
  }

  password_tab.disabled = false;
  diagnose_tab.disabled = false;
  settings_tab.disabled = false;
  erase_tab.disabled = false;

  drive_selection.value = device;

  let password_tab_content = document.querySelector("#nav-password");
  let diagnose_tab_content = document.querySelector("#nav-diagnose");
  let settings_tab_content = document.querySelector("#nav-settings");
  let erase_tab_content = document.querySelector("#nav-erase");

  let activate_tab = password_tab;
  let activate_tab_content = password_tab_content;

  let unlock_block = document.querySelector("#unlock-div");
  let set_password_block = document.querySelector("#set-password-div");
  let change_password_block = document.querySelector("#change-password-div");

  let unlock_password_input = document.querySelector("#input-unlock");
  let unlock_hint_input = document.querySelector("#input-hint-show");

  let remove_password_check = document.querySelector("#change-password-remove");
  let change_password_check = document.querySelector("#change-password-change");
  let current_password_input = document.querySelector("#input-current-password");
  let current_hint_input = document.querySelector("#input-current-hint");
  let change_password_trs = document.querySelectorAll(".new-password-class");
  let new_password_input = document.querySelector("#input-new-password");
  let new_verify_password_input = document.querySelector("#input-new-verify-password");
  let new_hint_input = document.querySelector("#input-new-hint-set");
  let remove_password_button = document.querySelector("#remove-password-button");
  let change_password_button = document.querySelector("#change-password-button");

  switch (await get_security_status()) {
    case 'NoUserPassword':
      toggle_block(false, unlock_block);
      toggle_block(true, set_password_block);
      toggle_block(false, change_password_block);
      break;
    case 'Locked':
      toggle_block(true, unlock_block);
      toggle_block(false, set_password_block);
      toggle_block(false, change_password_block);
      unlock_hint_input.value = await get_hint();
      break;
    case 'Unlocked':
      toggle_block(false, unlock_block);
      toggle_block(false, set_password_block);
      toggle_block(true, change_password_block);
      current_hint_input.value = await get_hint();
      break;
    case 'UnlockAttemptExceeded':
      await message("Unlock attempts exceeded. Either unplug and retry or erase the disk.");
      activate_tab = erase_tab;
      activate_tab_content = erase_tab_content;

      password_tab.disabled = true;
      diagnose_tab.disabled = true;
      settings_tab.disabled = true;
      break;
    case 'NoEncryption':
      await message("This device does not support encryption")
      activate_tab = diagnose_tab;
      activate_tab_content = diagnose_tab_content;

      password_tab.disabled = true;
      erase_tab.disabled = true;
      break;
    default:
      break;
  }

  document
    .querySelector("#unlock-button")
    .addEventListener("click", async () => {
      let password = unlock_password_input.value;
      if (password == '') {
        await message("The password is empty.");
        return;
      }

      try {
        await unlock_device(password)
        location.reload();
      } catch (error) {
        await message(error)
      }
    });

  document
    .querySelector("#set-password-button")
    .addEventListener("click", async () => {
      let password = document.querySelector("#input-password").value;
      let password_verify = document.querySelector("#input-verify-password").value;
      if (password != password_verify) {
        await message("Two passwords are not the same.");
        return;
      }

      if (password == '') {
        await message("The password is empty.");
        return;
      }

      try {
        await set_password(password, document.querySelector("#input-hint-set").value);
        location.reload();
      } catch (error) {
        await message(error);
      }
    });

  for (const tr of change_password_trs) {
    toggle_block(!remove_password_check.checked, tr);
  }

  toggle_block(remove_password_check.checked, remove_password_button);
  toggle_block(change_password_button.checked, change_password_button);

  remove_password_check.addEventListener("change", (e) => {
    for (const tr of change_password_trs) {
      toggle_block(e.target.checked, remove_password_button);
      toggle_block(!e.target.checked, change_password_button);
      toggle_block(!e.target.checked, tr);
    }
  });

  change_password_check.addEventListener("change", (e) => {
    for (const tr of change_password_trs) {
      toggle_block(e.target.checked, change_password_button);
      toggle_block(!e.target.checked, remove_password_button);
      toggle_block(e.target.checked, tr);
    }
  });

  remove_password_button.addEventListener("click", async () => {
    let current_password = current_password_input.value;

    if (current_password == '') {
      await message("The password is empty.");
      return;
    }

    try {
      await remove_password(current_password);
      location.reload();
    } catch (error) {
      await message(error);
    }
  });

  change_password_button.addEventListener("click", async () => {
    let current_password = current_password_input.value;
    let new_password = new_password_input.value;
    let new_verify_password = new_verify_password_input.value;
    let new_hint = new_hint_input.value;

    if (current_password == '') {
      await message("The password is empty.");
      return;
    }

    if (new_password == '') {
      await message("The password is empty.");
      return;
    }

    if (new_password != new_verify_password) {
      await message("Two passwords are not the same.");
      return;
    }

    try {
      await change_password(current_password, new_password, new_hint);
      location.reload();
    } catch (error) {
      await message(error);
    }
  });

  document
    .querySelector("#run-self-test")
    .addEventListener("click", () => basic_diagnose());

  let sleep_timer_check = document.querySelector("#sleep-timer-switch");
  let sleep_timer_block = document.querySelector("#sleep-timer-value-block");
  let sleep_timer_input = document.querySelector("#input-timer");
  let sleep_timer_button = document.querySelector("#sleep-timer-button");

  sleep_timer_check.checked = await get_sleep_timer() != 0;
  sleep_timer_input.value = await get_sleep_timer();
  toggle_block(sleep_timer_check.checked, sleep_timer_block)

  let timer_changed = false;

  sleep_timer_check.addEventListener("change", async (e) => {
    if (!e.target.checked && timer_changed) {
      await set_sleep_timer(0);
      sleep_timer_input.value = 0;
      timer_changed = false;
    }

    toggle_block(e.target.checked, sleep_timer_block);
  });
  sleep_timer_button.addEventListener("click", async () => {
    set_sleep_timer(sleep_timer_input.value);
    sleep_timer_input.value = await get_sleep_timer();
    timer_changed = true;
  })

  let led_switch = document.querySelector("#led-switch");
  led_switch.checked = await get_led_state();
  led_switch.addEventListener("change", async (e) => { await set_led_state(e.target.checked) });

  let vcd_switch = document.querySelector("#virtualcd-switch");
  vcd_switch.checked = await get_vcd_state();
  vcd_switch.addEventListener("change", async (e) => { await set_vcd_state(e.target.checked) });

  let erase_button = document.querySelector("#erase-button");
  let erase_check = document.querySelector("#agree-erase");

  erase_check.addEventListener("change", (e) => { (erase_button.disabled = !e.target.checked) });
  erase_button.addEventListener("click", async () => {
    try {
      await erase_device();
      await message("Successfully erased!");
    } catch (error) {
      await message(error)
    }
    erase_check.checked = false;
    erase_button.disabled = true;
  });

  activate_tab.classList.add("active");
  activate_tab_content.classList.add("show");
  activate_tab_content.classList.add("active");
}

window.addEventListener("DOMContentLoaded", async () => {
  try {
    await main();
  } catch (error) {
    await message(error);
    await appWindow.close();
  }
});