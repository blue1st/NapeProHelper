pub mod config;

use config::{AppConfig, ConfigState};
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager, State, WindowEvent};

pub fn build_tray_menu(app: &tauri::AppHandle, cfg: &AppConfig) -> Result<Menu<tauri::Wry>, tauri::Error> {
    let dev = &cfg.device;
    let quit_i = MenuItem::with_id(app, "quit", "終了 (Quit)", true, None::<&str>)?;
    let show_i = MenuItem::with_id(app, "toggle_window", "ウィンドウの表示 / 非表示", true, None::<&str>)?;
    let launcher_i = MenuItem::with_id(app, "open_launcher", "🌐 Keychron Launcher (公式Web設定) を開く", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let sep3 = PredefinedMenuItem::separator(app)?;

    // 8 Layer Items (Layer 0 to 7)
    let mut layer_items = Vec::new();
    for i in 0..8u8 {
        let layer_id = i;
        let is_active = layer_id == dev.active_layer;
        let angle = dev.layer_octashift_angles.get(&layer_id).cloned().unwrap_or((layer_id as u16) * 45);
        let mark = if is_active { "✓ " } else { "   " };

        let custom_name = dev.layer_names.get(&layer_id).cloned();
        let text = match custom_name {
            Some(ref n) if !n.is_empty() && n != &format!("Layer {}", layer_id) && n != &format!("L{}", layer_id) => {
                format!("{}Layer {} ({}°): {}", mark, layer_id, angle, n)
            }
            _ => {
                format!("{}Layer {} ({}°)", mark, layer_id, angle)
            }
        };

        let item = MenuItem::with_id(app, &format!("layer_{}", i), &text, true, None::<&str>)?;
        layer_items.push(item);
    }

    // Auto Switch Mode Item
    let auto_mark = if cfg.auto_switch_enabled { "✓ " } else { "   " };
    let auto_status = if cfg.auto_switch_enabled { "ON" } else { "OFF" };
    let auto_switch_item = MenuItem::with_id(
        app,
        "toggle_auto_switch",
        &format!("{}🔄 自動切り替え ({})", auto_mark, auto_status),
        true,
        None::<&str>,
    )?;

    let mut item_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = Vec::new();
    item_refs.push(&show_i);
    item_refs.push(&launcher_i);
    item_refs.push(&sep1);

    for item in &layer_items {
        item_refs.push(item as &dyn tauri::menu::IsMenuItem<tauri::Wry>);
    }

    item_refs.push(&sep2);
    item_refs.push(&auto_switch_item);

    // Optional Hardware Controls (only shown when show_advanced_hardware_controls is enabled)
    let mut scroll_item_opt = None;
    let mut gesture_item_opt = None;
    let mut dpi_sub_opt = None;
    let mut dpi_items = Vec::new();

    if cfg.show_advanced_hardware_controls {
        let scroll_mark = if dev.trackball_scroll_mode { "✓ " } else { "   " };
        let scroll_status = if dev.trackball_scroll_mode { "ON" } else { "OFF" };
        let s_item = MenuItem::with_id(
            app,
            "toggle_scroll",
            &format!("{}📜 ボールスクロール ({})", scroll_mark, scroll_status),
            true,
            None::<&str>,
        )?;

        let gesture_mark = if dev.trackball_gesture_mode { "✓ " } else { "   " };
        let gesture_status = if dev.trackball_gesture_mode { "ON" } else { "OFF" };
        let g_item = MenuItem::with_id(
            app,
            "toggle_gesture",
            &format!("{}🖐️ ジェスチャー機能 ({})", gesture_mark, gesture_status),
            true,
            None::<&str>,
        )?;

        let dpi_preset_values: Vec<u16> = vec![400, 800, 1800, 3200, 4000];
        for dpi in &dpi_preset_values {
            let mark = if dev.pointer_dpi == *dpi { "✓ " } else { "   " };
            let item = MenuItem::with_id(app, &format!("dpi_{}", dpi), &format!("{}{} DPI", mark, dpi), true, None::<&str>)?;
            dpi_items.push(item);
        }
        let dpi_item_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> =
            dpi_items.iter().map(|item| item as &dyn tauri::menu::IsMenuItem<tauri::Wry>).collect();
        let status_str = if dev.is_connected {
            format!("🎯 DPI 設定 (現在 {} DPI)", dev.pointer_dpi)
        } else {
            "🎯 DPI 設定 (未接続)".to_string()
        };
        let dpi_sub = Submenu::with_items(app, &status_str, true, &dpi_item_refs)?;

        scroll_item_opt = Some(s_item);
        gesture_item_opt = Some(g_item);
        dpi_sub_opt = Some(dpi_sub);
    }

    if let Some(ref s) = scroll_item_opt {
        item_refs.push(s as &dyn tauri::menu::IsMenuItem<tauri::Wry>);
    }
    if let Some(ref g) = gesture_item_opt {
        item_refs.push(g as &dyn tauri::menu::IsMenuItem<tauri::Wry>);
    }
    if let Some(ref d) = dpi_sub_opt {
        item_refs.push(d as &dyn tauri::menu::IsMenuItem<tauri::Wry>);
    }

    item_refs.push(&sep3);
    item_refs.push(&quit_i);

    Menu::with_items(app, &item_refs)
}

pub fn sync_tray_menu(app: &tauri::AppHandle, cfg: &AppConfig) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        if let Ok(menu) = build_tray_menu(app, cfg) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

#[tauri::command]
async fn get_config(app: tauri::AppHandle, state: State<'_, ConfigState>) -> Result<AppConfig, String> {
    let state_arc = state.0.clone();
    let cfg = tauri::async_runtime::spawn_blocking(move || {
        let mut cfg = state_arc.lock().unwrap_or_else(|e| e.into_inner());
        config::scan_hid_devices(&mut cfg);
        config::save_config_to_file(&cfg);
        cfg.clone()
    })
    .await
    .map_err(|e| e.to_string())?;

    sync_tray_menu(&app, &cfg);
    Ok(cfg)
}

#[tauri::command]
async fn check_connection(app: tauri::AppHandle, state: State<'_, ConfigState>) -> Result<AppConfig, String> {
    let state_arc = state.0.clone();
    let cfg = tauri::async_runtime::spawn_blocking(move || {
        let mut cfg = state_arc.lock().unwrap_or_else(|e| e.into_inner());
        config::scan_hid_devices(&mut cfg);
        cfg.clone()
    })
    .await
    .map_err(|e| e.to_string())?;

    sync_tray_menu(&app, &cfg);
    Ok(cfg)
}

#[tauri::command]
async fn set_active_layer(app: tauri::AppHandle, _device_id: Option<String>, layer_id: u8, state: State<'_, ConfigState>) -> Result<AppConfig, String> {
    let state_arc = state.0.clone();
    let cfg = tauri::async_runtime::spawn_blocking(move || {
        let mut cfg = state_arc.lock().unwrap_or_else(|e| e.into_inner());
        cfg.device.active_layer = layer_id;
        if cfg.device.is_connected {
            let mut written = false;
            if let Ok(api) = hidapi::HidApi::new() {
                for dev_info in api.device_list() {
                    if config::is_target_nape_device(dev_info) {
                        if let Ok(hid_dev) = dev_info.open_device(&api) {
                            let mut req = [0u8; 33];
                            req[0] = 0x00;
                            req[1] = 0xA7; // KC_MISC_CMD_GROUP
                            req[2] = 45;   // KC_USER_CMD_NAPE_SET_LAYER (45 / 0x2D)
                            req[3] = layer_id + 1; // 1-based layer index (1..8)
                            if hid_dev.write(&req).is_ok() {
                                written = true;
                            }
                            break;
                        }
                    }
                }
            }
            if !written {
                cfg.device.is_connected = false;
            }
        }
        config::save_config_to_file(&cfg);
        cfg.clone()
    })
    .await
    .map_err(|e| e.to_string())?;

    sync_tray_menu(&app, &cfg);
    let _ = app.emit("config-updated", &cfg);
    Ok(cfg)
}

#[tauri::command]
async fn set_octashift_angle(app: tauri::AppHandle, _device_id: Option<String>, layer_id: Option<u8>, angle: u16, state: State<'_, ConfigState>) -> Result<AppConfig, String> {
    let state_arc = state.0.clone();
    let cfg = tauri::async_runtime::spawn_blocking(move || {
        let mut cfg = state_arc.lock().unwrap_or_else(|e| e.into_inner());
        let target_layer = layer_id.unwrap_or(cfg.device.active_layer);
        cfg.device.layer_octashift_angles.insert(target_layer, angle);
        if target_layer == cfg.device.active_layer {
            cfg.device.octashift_angle = angle;
        }

        if cfg.device.is_connected {
            let mut written = false;
            if let Ok(api) = hidapi::HidApi::new() {
                for dev_info in api.device_list() {
                    if config::is_target_nape_device(dev_info) {
                        if let Ok(hid_dev) = dev_info.open_device(&api) {
                            written = config::set_octashift_angle_official(&hid_dev, target_layer, angle);
                            break;
                        }
                    }
                }
            }
            if !written {
                cfg.device.is_connected = false;
            }
        }
        config::save_config_to_file(&cfg);
        cfg.clone()
    })
    .await
    .map_err(|e| e.to_string())?;

    sync_tray_menu(&app, &cfg);
    let _ = app.emit("config-updated", &cfg);
    Ok(cfg)
}

#[tauri::command]
async fn set_pointer_dpi(app: tauri::AppHandle, _device_id: Option<String>, dpi: u16, state: State<'_, ConfigState>) -> Result<AppConfig, String> {
    let state_arc = state.0.clone();
    let cfg = tauri::async_runtime::spawn_blocking(move || {
        let mut cfg = state_arc.lock().unwrap_or_else(|e| e.into_inner());
        cfg.device.pointer_dpi = dpi;

        if cfg.device.is_connected {
            let mut written = false;
            if let Ok(api) = hidapi::HidApi::new() {
                for dev_info in api.device_list() {
                    if config::is_target_nape_device(dev_info) {
                        if let Ok(hid_dev) = dev_info.open_device(&api) {
                            written = config::set_pointer_dpi_official(&hid_dev, dpi);
                            break;
                        }
                    }
                }
            }
            if !written {
                cfg.device.is_connected = false;
            }
        }
        config::save_config_to_file(&cfg);
        cfg.clone()
    })
    .await
    .map_err(|e| e.to_string())?;

    sync_tray_menu(&app, &cfg);
    let _ = app.emit("config-updated", &cfg);
    Ok(cfg)
}

#[tauri::command]
async fn set_trackball_scroll_mode(app: tauri::AppHandle, _device_id: Option<String>, enabled: bool, state: State<'_, ConfigState>) -> Result<AppConfig, String> {
    let state_arc = state.0.clone();
    let cfg = tauri::async_runtime::spawn_blocking(move || {
        let mut cfg = state_arc.lock().unwrap_or_else(|e| e.into_inner());
        cfg.device.trackball_scroll_mode = enabled;

        if cfg.device.is_connected {
            let mut written = false;
            if let Ok(api) = hidapi::HidApi::new() {
                for dev_info in api.device_list() {
                    if config::is_target_nape_device(dev_info) {
                        if let Ok(hid_dev) = dev_info.open_device(&api) {
                            written = config::set_trackball_force_gesture_scroll_official(&hid_dev, cfg.device.trackball_gesture_mode, cfg.device.trackball_scroll_mode);
                            break;
                        }
                    }
                }
            }
            if !written {
                cfg.device.is_connected = false;
            }
        }
        config::save_config_to_file(&cfg);
        cfg.clone()
    })
    .await
    .map_err(|e| e.to_string())?;

    sync_tray_menu(&app, &cfg);
    let _ = app.emit("config-updated", &cfg);
    Ok(cfg)
}

#[tauri::command]
async fn set_trackball_gesture_mode(app: tauri::AppHandle, _device_id: Option<String>, enabled: bool, state: State<'_, ConfigState>) -> Result<AppConfig, String> {
    let state_arc = state.0.clone();
    let cfg = tauri::async_runtime::spawn_blocking(move || {
        let mut cfg = state_arc.lock().unwrap_or_else(|e| e.into_inner());
        cfg.device.trackball_gesture_mode = enabled;

        if cfg.device.is_connected {
            let mut written = false;
            if let Ok(api) = hidapi::HidApi::new() {
                for dev_info in api.device_list() {
                    if config::is_target_nape_device(dev_info) {
                        if let Ok(hid_dev) = dev_info.open_device(&api) {
                            written = config::set_trackball_force_gesture_scroll_official(&hid_dev, cfg.device.trackball_gesture_mode, cfg.device.trackball_scroll_mode);
                            break;
                        }
                    }
                }
            }
            if !written {
                cfg.device.is_connected = false;
            }
        }
        config::save_config_to_file(&cfg);
        cfg.clone()
    })
    .await
    .map_err(|e| e.to_string())?;

    sync_tray_menu(&app, &cfg);
    let _ = app.emit("config-updated", &cfg);
    Ok(cfg)
}

#[tauri::command]
async fn update_button_mapping(
    app: tauri::AppHandle,
    _device_id: Option<String>,
    layer_id: u8,
    button_id: u8,
    action_type: String,
    key_code: String,
    description: String,
    state: State<'_, ConfigState>,
) -> Result<AppConfig, String> {
    let state_arc = state.0.clone();
    let cfg = tauri::async_runtime::spawn_blocking(move || {
        let mut cfg = state_arc.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(mappings) = cfg.device.button_mappings.get_mut(&layer_id) {
            if let Some(b) = mappings.iter_mut().find(|m| m.button_id == button_id) {
                b.action_type = action_type;
                b.key_code = key_code;
                b.description = description;
            }
        }
        config::save_config_to_file(&cfg);
        cfg.clone()
    })
    .await
    .map_err(|e| e.to_string())?;

    sync_tray_menu(&app, &cfg);
    let _ = app.emit("config-updated", &cfg);
    Ok(cfg)
}

#[tauri::command]
async fn refresh_from_hardware(app: tauri::AppHandle, _device_id: Option<String>, state: State<'_, ConfigState>) -> Result<AppConfig, String> {
    let state_arc = state.0.clone();
    let cfg = tauri::async_runtime::spawn_blocking(move || {
        let mut cfg = state_arc.lock().unwrap_or_else(|e| e.into_inner());
        config::refresh_device_from_hardware(&mut cfg, None);
        cfg.clone()
    })
    .await
    .map_err(|e| e.to_string())?;

    sync_tray_menu(&app, &cfg);
    let _ = app.emit("config-updated", &cfg);
    Ok(cfg)
}

#[tauri::command]
async fn debug_dump_eeprom() -> Result<String, String> {
    let result = tauri::async_runtime::spawn_blocking(move || {
        let api = hidapi::HidApi::new().map_err(|e| e.to_string())?;
        let mut output = String::new();

        for dev_info in api.device_list() {
            if !config::is_target_nape_device(dev_info) {
                continue;
            }

            output.push_str(&format!("=== Device: VID={:04X} PID={:04X} serial={} ===\n",
                dev_info.vendor_id(), dev_info.product_id(), dev_info.serial_number().unwrap_or("?")));

            let device = dev_info.open_device(&api).map_err(|e| e.to_string())?;

            for layer in 0..8u8 {
                let offset = (layer as u16) * 28;
                let mut req = [0u8; 33];
                req[0] = 0x00;
                req[1] = 0x12;
                req[2] = ((offset >> 8) & 0xFF) as u8;
                req[3] = (offset & 0xFF) as u8;
                req[4] = 28;

                if device.write(&req).is_ok() {
                    let mut buf = [0u8; 32];
                    if let Ok(n) = device.read_timeout(&mut buf, 200) {
                        let si = if n >= 4 && buf[0] == 0x12 { 0 }
                                 else if n >= 5 && buf[1] == 0x12 { 1 }
                                 else { 999 };

                        if si != 999 {
                            let di = si + 4;
                            output.push_str(&format!("\n--- Layer {} (offset={}, raw_bytes={}) ---\n", layer, offset, n));

                            let raw_hex: String = buf[di..n].iter().map(|b| format!("{:02X} ", b)).collect();
                            output.push_str(&format!("  RAW: {}\n", raw_hex.trim()));

                            let mut idx = 0;
                            for i in (di..n.min(di + 32)).step_by(2) {
                                if i + 1 < n {
                                    let code = u16::from_be_bytes([buf[i], buf[i + 1]]);
                                    let (act, kc, desc) = config::parse_qmk_keycode(code);
                                    output.push_str(&format!(
                                        "  [{}] 0x{:04X} → {} ({}) [{}]\n",
                                        idx, code, desc, kc, act
                                    ));
                                    idx += 1;
                                }
                            }
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            break;
        }

        if output.is_empty() {
            output = "No Nape Pro device found.".to_string();
        }
        Ok::<String, String>(output)
    })
    .await
    .map_err(|e| e.to_string())??;

    Ok(result)
}

#[tauri::command]
fn open_keychron_launcher() -> Result<(), String> {
    open_url("https://launcher.keychron.com/".to_string())
}

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(&url).spawn();

    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd").args(["/C", "start", "", &url]).spawn();

    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(&url).spawn();

    Ok(())
}

static LAST_EXTERNAL_APP: Mutex<Option<ActiveAppInfo>> = Mutex::new(None);

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ActiveAppInfo {
    pub app_name: String,
    pub title: String,
    pub process_path: String,
}

#[tauri::command]
fn get_active_app_info() -> Result<ActiveAppInfo, String> {
    if let Ok(win) = active_win_pos_rs::get_active_window() {
        let app_name = win.app_name.trim().to_lowercase();
        let proc_path = win.process_path.to_string_lossy().to_lowercase();
        let is_self = app_name.contains("napepro") || proc_path.contains("napepro");

        if !is_self {
            let info = ActiveAppInfo {
                app_name: win.app_name,
                title: win.title,
                process_path: win.process_path.to_string_lossy().to_string(),
            };
            if let Ok(mut guard) = LAST_EXTERNAL_APP.lock() {
                *guard = Some(info.clone());
            }
            return Ok(info);
        }
    }

    if let Ok(guard) = LAST_EXTERNAL_APP.lock() {
        if let Some(ref info) = *guard {
            return Ok(info.clone());
        }
    }

    Err("直前に使用していたアプリケーション情報を取得できませんでした。".into())
}

#[tauri::command]
async fn get_active_app_info_delayed(delay_seconds: u64) -> Result<ActiveAppInfo, String> {
    tokio::time::sleep(std::time::Duration::from_secs(delay_seconds)).await;
    match active_win_pos_rs::get_active_window() {
        Ok(win) => {
            let info = ActiveAppInfo {
                app_name: win.app_name,
                title: win.title,
                process_path: win.process_path.to_string_lossy().to_string(),
            };
            let app_name = info.app_name.trim().to_lowercase();
            let proc_path = info.process_path.to_lowercase();
            if !app_name.contains("napepro") && !proc_path.contains("napepro") {
                if let Ok(mut guard) = LAST_EXTERNAL_APP.lock() {
                    *guard = Some(info.clone());
                }
            }
            Ok(info)
        }
        Err(_) => Err("アプリケーション情報を取得できませんでした。".into()),
    }
}

#[tauri::command]
async fn update_general_config(
    app: tauri::AppHandle,
    show_notifications: Option<bool>,
    show_advanced_hardware_controls: Option<bool>,
    state: State<'_, ConfigState>,
) -> Result<AppConfig, String> {
    let state_arc = state.0.clone();
    let cfg = tauri::async_runtime::spawn_blocking(move || {
        let mut cfg = state_arc.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(v) = show_notifications {
            cfg.show_notifications = v;
        }
        if let Some(v) = show_advanced_hardware_controls {
            cfg.show_advanced_hardware_controls = v;
        }
        config::save_config_to_file(&cfg);
        cfg.clone()
    })
    .await
    .map_err(|e| e.to_string())?;

    sync_tray_menu(&app, &cfg);
    let _ = app.emit("config-updated", &cfg);
    Ok(cfg)
}

#[tauri::command]
async fn update_auto_switch_config(
    app: tauri::AppHandle,
    enabled: bool,
    default_layer: Option<u8>,
    rules: Vec<config::AutoSwitchRule>,
    state: State<'_, ConfigState>,
) -> Result<AppConfig, String> {
    let state_arc = state.0.clone();
    let cfg = tauri::async_runtime::spawn_blocking(move || {
        let mut cfg = state_arc.lock().unwrap_or_else(|e| e.into_inner());
        cfg.auto_switch_enabled = enabled;
        cfg.auto_switch_default_layer = default_layer;
        cfg.auto_switch_rules = rules;
        config::save_config_to_file(&cfg);
        cfg.clone()
    })
    .await
    .map_err(|e| e.to_string())?;

    sync_tray_menu(&app, &cfg);
    let _ = app.emit("config-updated", &cfg);
    Ok(cfg)
}

/// Native Rust Background HID Connection Monitor
/// Independent of WebKit UI timer throttling, checks USB HID status every 2 seconds.
/// Automatically handles device disconnects (e.g. KVM switch away) and re-connects (e.g. KVM switch back).
fn start_hid_connection_monitor(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;

            let app_handle_clone = app_handle.clone();
            let _ = tauri::async_runtime::spawn_blocking(move || {
                let catch_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    if let Some(state) = app_handle_clone.try_state::<ConfigState>() {
                        let mut cfg = state.0.lock().unwrap_or_else(|e| e.into_inner());
                        let prev_connected = cfg.device.is_connected;
                        let now_connected = config::scan_hid_devices(&mut cfg);

                        if prev_connected != now_connected {
                            config::save_config_to_file(&cfg);
                            sync_tray_menu(&app_handle_clone, &cfg);
                            let _ = app_handle_clone.emit("config-updated", cfg.clone());
                        }
                    }
                }));
                if catch_res.is_err() {
                    eprintln!("Panic caught in HID connection monitor background task");
                }
            }).await;
        }
    });
}

fn start_auto_switch_monitor(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut last_app_key = String::new();
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            let monitor_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if let Ok(win) = active_win_pos_rs::get_active_window() {
                    let app_name = win.app_name.trim().to_lowercase();
                    let proc_path = win.process_path.to_string_lossy().to_lowercase();

                    let is_self = app_name.contains("napepro") || proc_path.contains("napepro");

                    if !is_self {
                        if let Ok(mut guard) = LAST_EXTERNAL_APP.lock() {
                            *guard = Some(ActiveAppInfo {
                                app_name: win.app_name.clone(),
                                title: win.title.clone(),
                                process_path: win.process_path.to_string_lossy().to_string(),
                            });
                        }
                    }
                }

                let (enabled, default_layer, rules, active_layer) = {
                    if let Some(state) = app_handle.try_state::<ConfigState>() {
                        let cfg = state.0.lock().unwrap_or_else(|e| e.into_inner());
                        (
                            cfg.auto_switch_enabled,
                            cfg.auto_switch_default_layer,
                            cfg.auto_switch_rules.clone(),
                            cfg.device.active_layer,
                        )
                    } else {
                        return;
                    }
                };

                if !enabled {
                    last_app_key.clear();
                    return;
                }

                if let Ok(win) = active_win_pos_rs::get_active_window() {
                    let app_name = win.app_name.trim().to_lowercase();
                    let title = win.title.trim().to_lowercase();
                    let proc_path = win.process_path.to_string_lossy().to_lowercase();

                    if app_name.contains("napepro") || proc_path.contains("napepro") {
                        return;
                    }

                    // Pause auto-switching when Keychron Launcher (WebHID configuration page) is active
                    let is_keychron_launcher = title.contains("keychron launcher")
                        || title.contains("launcher.keychron")
                        || (title.contains("keychron") && title.contains("launcher"));
                    if is_keychron_launcher {
                        return;
                    }

                    let current_app_key = format!("{}:{}", app_name, title);
                    if current_app_key == last_app_key {
                        return;
                    }

                    let mut matched_layer: Option<u8> = None;

                    for rule in &rules {
                        if !rule.enabled {
                            continue;
                        }
                        let rule_target = rule.app_name.trim().to_lowercase();
                        if rule_target.is_empty() {
                            continue;
                        }

                        if app_name.contains(&rule_target) || title.contains(&rule_target) || proc_path.contains(&rule_target) {
                            matched_layer = Some(rule.target_layer);
                            break;
                        }
                    }

                    let target_layer_to_set = matched_layer.or(default_layer);
                    last_app_key = current_app_key;

                    if let Some(target_layer) = target_layer_to_set {
                        if target_layer != active_layer {
                            let app_handle_clone = app_handle.clone();
                            let _ = tauri::async_runtime::spawn_blocking(move || {
                                if let Some(state) = app_handle_clone.try_state::<ConfigState>() {
                                    let mut cfg = state.0.lock().unwrap_or_else(|e| e.into_inner());
                                    cfg.device.active_layer = target_layer;
                                    if cfg.device.is_connected {
                                        let mut written = false;
                                        if let Ok(api) = hidapi::HidApi::new() {
                                            for dev_info in api.device_list() {
                                                if config::is_target_nape_device(dev_info) {
                                                    if let Ok(hid_dev) = dev_info.open_device(&api) {
                                                        let mut req = [0u8; 33];
                                                        req[0] = 0x00;
                                                        req[1] = 0xA7;
                                                        req[2] = 45;
                                                        req[3] = target_layer + 1;
                                                        if hid_dev.write(&req).is_ok() {
                                                            written = true;
                                                        }
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        if !written {
                                            cfg.device.is_connected = false;
                                        }
                                    }
                                    config::save_config_to_file(&cfg);
                                    sync_tray_menu(&app_handle_clone, &cfg);
                                    let _ = app_handle_clone.emit("config-updated", cfg.clone());
                                }
                            });
                        }
                    }
                }
            }));
            if monitor_res.is_err() {
                eprintln!("Panic caught in auto switch monitor task");
            }
        }
    });
}

pub fn run() {
    let config_state = ConfigState::new();
    let initial_config = {
        let guard = config_state.0.lock().unwrap_or_else(|e| e.into_inner());
        guard.clone()
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .manage(config_state)
        .setup(move |app| {
            let menu = build_tray_menu(app.handle(), &initial_config)?;

            let mut tray_builder = TrayIconBuilder::with_id("main-tray").menu(&menu);
            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }

            let _tray = tray_builder
                .on_menu_event(|app, event| {
                    let id = event.id.as_ref().to_string();
                    let app_handle = app.clone();

                    if id == "quit" {
                        app.exit(0);
                    } else if id == "toggle_window" {
                        if let Some(window) = app.get_webview_window("main") {
                            let is_visible = window.is_visible().unwrap_or(false);
                            if is_visible {
                                let _ = window.hide();
                                #[cfg(target_os = "macos")]
                                let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
                            } else {
                                #[cfg(target_os = "macos")]
                                let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    } else if id == "open_launcher" {
                        let _ = open_url("https://launcher.keychron.com/".to_string());
                    } else if id.starts_with("layer_") {
                        if let Ok(layer_idx) = id.trim_start_matches("layer_").parse::<u8>() {
                            let app_handle = app_handle.clone();
                            tauri::async_runtime::spawn(async move {
                                let state = app_handle.state::<ConfigState>();
                                let state_arc = state.0.clone();
                                let cfg_res = tauri::async_runtime::spawn_blocking(move || {
                                    let mut cfg = state_arc.lock().unwrap_or_else(|e| e.into_inner());
                                    cfg.device.active_layer = layer_idx;
                                    if cfg.device.is_connected {
                                        let mut written = false;
                                        if let Ok(api) = hidapi::HidApi::new() {
                                            for dev_info in api.device_list() {
                                                if config::is_target_nape_device(dev_info) {
                                                    if let Ok(hid_dev) = dev_info.open_device(&api) {
                                                        let mut req = [0u8; 33];
                                                        req[0] = 0x00;
                                                        req[1] = 0xA7;
                                                        req[2] = 45;
                                                        req[3] = layer_idx + 1;
                                                        if hid_dev.write(&req).is_ok() {
                                                            written = true;
                                                        }
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        if !written {
                                            cfg.device.is_connected = false;
                                        }
                                    }
                                    config::save_config_to_file(&cfg);
                                    cfg.clone()
                                }).await;

                                if let Ok(cfg) = cfg_res {
                                    sync_tray_menu(&app_handle, &cfg);
                                    let _ = app_handle.emit("config-updated", &cfg);
                                }
                            });
                        }
                    } else if id == "toggle_scroll" {
                        let app_handle = app_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            let state = app_handle.state::<ConfigState>();
                            let state_arc = state.0.clone();
                            let cfg_res = tauri::async_runtime::spawn_blocking(move || {
                                let mut cfg = state_arc.lock().unwrap_or_else(|e| e.into_inner());
                                let next_mode = !cfg.device.trackball_scroll_mode;
                                cfg.device.trackball_scroll_mode = next_mode;
                                if cfg.device.is_connected {
                                    let mut written = false;
                                    if let Ok(api) = hidapi::HidApi::new() {
                                        for dev_info in api.device_list() {
                                            if config::is_target_nape_device(dev_info) {
                                                if let Ok(hid_dev) = dev_info.open_device(&api) {
                                                    written = config::set_trackball_force_gesture_scroll_official(&hid_dev, cfg.device.trackball_gesture_mode, next_mode);
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    if !written {
                                        cfg.device.is_connected = false;
                                    }
                                }
                                config::save_config_to_file(&cfg);
                                cfg.clone()
                            }).await;

                            if let Ok(cfg) = cfg_res {
                                sync_tray_menu(&app_handle, &cfg);
                                let _ = app_handle.emit("config-updated", &cfg);
                            }
                        });
                    } else if id == "toggle_gesture" {
                        let app_handle = app_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            let state = app_handle.state::<ConfigState>();
                            let state_arc = state.0.clone();
                            let cfg_res = tauri::async_runtime::spawn_blocking(move || {
                                let mut cfg = state_arc.lock().unwrap_or_else(|e| e.into_inner());
                                let next_mode = !cfg.device.trackball_gesture_mode;
                                cfg.device.trackball_gesture_mode = next_mode;
                                if cfg.device.is_connected {
                                    let mut written = false;
                                    if let Ok(api) = hidapi::HidApi::new() {
                                        for dev_info in api.device_list() {
                                            if config::is_target_nape_device(dev_info) {
                                                if let Ok(hid_dev) = dev_info.open_device(&api) {
                                                    written = config::set_trackball_force_gesture_scroll_official(&hid_dev, next_mode, cfg.device.trackball_scroll_mode);
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    if !written {
                                        cfg.device.is_connected = false;
                                    }
                                }
                                config::save_config_to_file(&cfg);
                                cfg.clone()
                            }).await;

                            if let Ok(cfg) = cfg_res {
                                sync_tray_menu(&app_handle, &cfg);
                                let _ = app_handle.emit("config-updated", &cfg);
                            }
                        });
                    } else if id == "toggle_auto_switch" {
                        let app_handle = app_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            let state = app_handle.state::<ConfigState>();
                            let state_arc = state.0.clone();
                            let cfg_res = tauri::async_runtime::spawn_blocking(move || {
                                let mut cfg = state_arc.lock().unwrap_or_else(|e| e.into_inner());
                                cfg.auto_switch_enabled = !cfg.auto_switch_enabled;
                                config::save_config_to_file(&cfg);
                                cfg.clone()
                            }).await;

                            if let Ok(cfg) = cfg_res {
                                sync_tray_menu(&app_handle, &cfg);
                                let _ = app_handle.emit("config-updated", &cfg);
                            }
                        });
                    } else if id.starts_with("dpi_") {
                        if let Ok(dpi_val) = id.trim_start_matches("dpi_").parse::<u16>() {
                            let app_handle = app_handle.clone();
                            tauri::async_runtime::spawn(async move {
                                let state = app_handle.state::<ConfigState>();
                                let state_arc = state.0.clone();
                                let cfg_res = tauri::async_runtime::spawn_blocking(move || {
                                    let mut cfg = state_arc.lock().unwrap_or_else(|e| e.into_inner());
                                    cfg.device.pointer_dpi = dpi_val;
                                    if cfg.device.is_connected {
                                        let mut written = false;
                                        if let Ok(api) = hidapi::HidApi::new() {
                                            for dev_info in api.device_list() {
                                                if config::is_target_nape_device(dev_info) {
                                                    if let Ok(hid_dev) = dev_info.open_device(&api) {
                                                        written = config::set_pointer_dpi_official(&hid_dev, dpi_val);
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        if !written {
                                            cfg.device.is_connected = false;
                                        }
                                    }
                                    config::save_config_to_file(&cfg);
                                    cfg.clone()
                                }).await;

                                if let Ok(cfg) = cfg_res {
                                    sync_tray_menu(&app_handle, &cfg);
                                    let _ = app_handle.emit("config-updated", &cfg);
                                }
                            });
                        }
                    }
                })
                .build(app)?;

            // Start background monitors (Auto App Switcher & HID Connection Monitor)
            start_auto_switch_monitor(app.handle().clone());
            start_hid_connection_monitor(app.handle().clone());

            let is_autostart = std::env::args().any(|arg| arg == "--autostart");
            if is_autostart {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                    #[cfg(target_os = "macos")]
                    let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = window.hide();
                #[cfg(target_os = "macos")]
                let _ = window.app_handle().set_activation_policy(tauri::ActivationPolicy::Accessory);
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            check_connection,
            set_active_layer,
            set_octashift_angle,
            set_pointer_dpi,
            set_trackball_scroll_mode,
            set_trackball_gesture_mode,
            update_button_mapping,
            refresh_from_hardware,
            debug_dump_eeprom,
            open_keychron_launcher,
            open_url,
            get_active_app_info,
            get_active_app_info_delayed,
            update_general_config,
            update_auto_switch_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
