pub mod config;

use config::{AppConfig, ConfigState};
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager, State, WindowEvent};

pub struct TrayMenuState {
    pub layer_items: Mutex<Vec<MenuItem<tauri::Wry>>>,
    pub scroll_item: Mutex<MenuItem<tauri::Wry>>,
    pub gesture_item: Mutex<MenuItem<tauri::Wry>>,
    pub dpi_items: Mutex<Vec<(u16, MenuItem<tauri::Wry>)>>,
    pub dpi_submenu: Mutex<Option<Submenu<tauri::Wry>>>,
}

pub fn sync_tray_menu(app: &tauri::AppHandle, cfg: &AppConfig) {
    if let Some(tray_state) = app.try_state::<TrayMenuState>() {
        let dev = &cfg.device;

        // 1. Sync Layer Items
        if let Ok(items) = tray_state.layer_items.lock() {
            for (i, item) in items.iter().enumerate() {
                let layer_id = i as u8;
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

                let _ = item.set_text(text);
            }
        }

        // 2. Sync Trackball Scroll Mode Item
        if let Ok(item) = tray_state.scroll_item.lock() {
            let mark = if dev.trackball_scroll_mode { "✓ " } else { "   " };
            let status = if dev.trackball_scroll_mode { "ON" } else { "OFF" };
            let _ = item.set_text(format!("{}📜 ボールスクロール ({})", mark, status));
        }

        // 3. Sync Trackball Gesture Mode Item
        if let Ok(item) = tray_state.gesture_item.lock() {
            let mark = if dev.trackball_gesture_mode { "✓ " } else { "   " };
            let status = if dev.trackball_gesture_mode { "ON" } else { "OFF" };
            let _ = item.set_text(format!("{}🖐️ ジェスチャー機能 ({})", mark, status));
        }

        // 4. Sync DPI Items & Submenu Title
        if let Ok(dpi_list) = tray_state.dpi_items.lock() {
            for (dpi_val, item) in dpi_list.iter() {
                let mark = if dev.pointer_dpi == *dpi_val { "✓ " } else { "   " };
                let _ = item.set_text(format!("{}{} DPI", mark, dpi_val));
            }
        }
        if let Ok(sub_guard) = tray_state.dpi_submenu.lock() {
            if let Some(ref sub) = *sub_guard {
                let _ = sub.set_text(format!("🎯 DPI 設定 (現在 {} DPI)", dev.pointer_dpi));
            }
        }
    }
}

#[tauri::command]
async fn get_config(app: tauri::AppHandle, state: State<'_, ConfigState>) -> Result<AppConfig, String> {
    let state_arc = state.0.clone();
    let cfg = tauri::async_runtime::spawn_blocking(move || {
        let mut cfg = state_arc.lock().map_err(|e| e.to_string())?;
        config::scan_hid_devices(&mut cfg);
        config::save_config_to_file(&cfg);
        Ok::<AppConfig, String>(cfg.clone())
    })
    .await
    .map_err(|e| e.to_string())??;

    sync_tray_menu(&app, &cfg);
    Ok(cfg)
}

#[tauri::command]
async fn check_connection(app: tauri::AppHandle, state: State<'_, ConfigState>) -> Result<AppConfig, String> {
    let state_arc = state.0.clone();
    let cfg = tauri::async_runtime::spawn_blocking(move || {
        let mut cfg = state_arc.lock().map_err(|e| e.to_string())?;
        config::scan_hid_devices(&mut cfg);
        Ok::<AppConfig, String>(cfg.clone())
    })
    .await
    .map_err(|e| e.to_string())??;

    sync_tray_menu(&app, &cfg);
    Ok(cfg)
}

#[tauri::command]
async fn set_active_layer(app: tauri::AppHandle, _device_id: Option<String>, layer_id: u8, state: State<'_, ConfigState>) -> Result<AppConfig, String> {
    let state_arc = state.0.clone();
    let cfg = tauri::async_runtime::spawn_blocking(move || {
        let mut cfg = state_arc.lock().map_err(|e| e.to_string())?;
        cfg.device.active_layer = layer_id;
        if cfg.device.is_connected {
            if let Ok(api) = hidapi::HidApi::new() {
                for dev_info in api.device_list() {
                    let vid = dev_info.vendor_id();
                    let pid = dev_info.product_id();
                    let prod = dev_info.product_string().unwrap_or("").to_lowercase();
                    let is_nape = (vid == 0x3434 && pid == 0x0440) || prod.contains("nape");
                    if is_nape && dev_info.usage_page() == 0xff60 && dev_info.usage() == 0x0061 {
                        if let Ok(hid_dev) = dev_info.open_device(&api) {
                            let mut req = [0u8; 33];
                            req[0] = 0x00;
                            req[1] = 0xA7; // KC_MISC_CMD_GROUP
                            req[2] = 45;   // KC_USER_CMD_NAPE_SET_LAYER (45 / 0x2D)
                            req[3] = layer_id + 1; // 1-based layer index
                            let _ = hid_dev.write(&req);
                            break;
                        }
                    }
                }
            }
        }
        config::save_config_to_file(&cfg);
        Ok::<AppConfig, String>(cfg.clone())
    })
    .await
    .map_err(|e| e.to_string())??;

    sync_tray_menu(&app, &cfg);
    let _ = app.emit("config-updated", &cfg);
    Ok(cfg)
}

#[tauri::command]
async fn set_octashift_angle(app: tauri::AppHandle, _device_id: Option<String>, layer_id: Option<u8>, angle: u16, state: State<'_, ConfigState>) -> Result<AppConfig, String> {
    let state_arc = state.0.clone();
    let cfg = tauri::async_runtime::spawn_blocking(move || {
        let mut cfg = state_arc.lock().map_err(|e| e.to_string())?;
        let target_layer = layer_id.unwrap_or(cfg.device.active_layer);
        cfg.device.layer_octashift_angles.insert(target_layer, angle);
        if target_layer == cfg.device.active_layer {
            cfg.device.octashift_angle = angle;
        }

        if cfg.device.is_connected {
            if let Ok(api) = hidapi::HidApi::new() {
                for dev_info in api.device_list() {
                    let vid = dev_info.vendor_id();
                    let pid = dev_info.product_id();
                    let prod = dev_info.product_string().unwrap_or("").to_lowercase();
                    let is_nape = (vid == 0x3434 && pid == 0x0440) || prod.contains("nape");
                    if is_nape && dev_info.usage_page() == 0xff60 && dev_info.usage() == 0x0061 {
                        if let Ok(hid_dev) = dev_info.open_device(&api) {
                            config::set_octashift_angle_official(&hid_dev, target_layer, angle);
                            break;
                        }
                    }
                }
            }
        }
        config::save_config_to_file(&cfg);
        Ok::<AppConfig, String>(cfg.clone())
    })
    .await
    .map_err(|e| e.to_string())??;

    sync_tray_menu(&app, &cfg);
    let _ = app.emit("config-updated", &cfg);
    Ok(cfg)
}

#[tauri::command]
async fn set_pointer_dpi(app: tauri::AppHandle, _device_id: Option<String>, dpi: u16, state: State<'_, ConfigState>) -> Result<AppConfig, String> {
    let state_arc = state.0.clone();
    let cfg = tauri::async_runtime::spawn_blocking(move || {
        let mut cfg = state_arc.lock().map_err(|e| e.to_string())?;
        cfg.device.pointer_dpi = dpi;

        if cfg.device.is_connected {
            if let Ok(api) = hidapi::HidApi::new() {
                for dev_info in api.device_list() {
                    let vid = dev_info.vendor_id();
                    let pid = dev_info.product_id();
                    let prod = dev_info.product_string().unwrap_or("").to_lowercase();
                    let is_nape = (vid == 0x3434 && pid == 0x0440) || prod.contains("nape");
                    if is_nape && dev_info.usage_page() == 0xff60 && dev_info.usage() == 0x0061 {
                        if let Ok(hid_dev) = dev_info.open_device(&api) {
                            config::set_pointer_dpi_official(&hid_dev, dpi);
                            break;
                        }
                    }
                }
            }
        }
        config::save_config_to_file(&cfg);
        Ok::<AppConfig, String>(cfg.clone())
    })
    .await
    .map_err(|e| e.to_string())??;

    sync_tray_menu(&app, &cfg);
    let _ = app.emit("config-updated", &cfg);
    Ok(cfg)
}

#[tauri::command]
async fn set_trackball_scroll_mode(app: tauri::AppHandle, _device_id: Option<String>, enabled: bool, state: State<'_, ConfigState>) -> Result<AppConfig, String> {
    let state_arc = state.0.clone();
    let cfg = tauri::async_runtime::spawn_blocking(move || {
        let mut cfg = state_arc.lock().map_err(|e| e.to_string())?;
        cfg.device.trackball_scroll_mode = enabled;

        if cfg.device.is_connected {
            if let Ok(api) = hidapi::HidApi::new() {
                for dev_info in api.device_list() {
                    let vid = dev_info.vendor_id();
                    let pid = dev_info.product_id();
                    let prod = dev_info.product_string().unwrap_or("").to_lowercase();
                    let is_nape = (vid == 0x3434 && pid == 0x0440) || prod.contains("nape");
                    if is_nape && dev_info.usage_page() == 0xff60 && dev_info.usage() == 0x0061 {
                        if let Ok(hid_dev) = dev_info.open_device(&api) {
                            config::set_trackball_force_gesture_scroll_official(&hid_dev, cfg.device.trackball_gesture_mode, cfg.device.trackball_scroll_mode);
                            break;
                        }
                    }
                }
            }
        }
        config::save_config_to_file(&cfg);
        Ok::<AppConfig, String>(cfg.clone())
    })
    .await
    .map_err(|e| e.to_string())??;

    sync_tray_menu(&app, &cfg);
    let _ = app.emit("config-updated", &cfg);
    Ok(cfg)
}

#[tauri::command]
async fn set_trackball_gesture_mode(app: tauri::AppHandle, _device_id: Option<String>, enabled: bool, state: State<'_, ConfigState>) -> Result<AppConfig, String> {
    let state_arc = state.0.clone();
    let cfg = tauri::async_runtime::spawn_blocking(move || {
        let mut cfg = state_arc.lock().map_err(|e| e.to_string())?;
        cfg.device.trackball_gesture_mode = enabled;

        if cfg.device.is_connected {
            if let Ok(api) = hidapi::HidApi::new() {
                for dev_info in api.device_list() {
                    let vid = dev_info.vendor_id();
                    let pid = dev_info.product_id();
                    let prod = dev_info.product_string().unwrap_or("").to_lowercase();
                    let is_nape = (vid == 0x3434 && pid == 0x0440) || prod.contains("nape");
                    if is_nape && dev_info.usage_page() == 0xff60 && dev_info.usage() == 0x0061 {
                        if let Ok(hid_dev) = dev_info.open_device(&api) {
                            config::set_trackball_force_gesture_scroll_official(&hid_dev, cfg.device.trackball_gesture_mode, cfg.device.trackball_scroll_mode);
                            break;
                        }
                    }
                }
            }
        }
        config::save_config_to_file(&cfg);
        Ok::<AppConfig, String>(cfg.clone())
    })
    .await
    .map_err(|e| e.to_string())??;

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
        let mut cfg = state_arc.lock().map_err(|e| e.to_string())?;
        if let Some(mappings) = cfg.device.button_mappings.get_mut(&layer_id) {
            if let Some(b) = mappings.iter_mut().find(|m| m.button_id == button_id) {
                b.action_type = action_type;
                b.key_code = key_code;
                b.description = description;
            }
        }
        config::save_config_to_file(&cfg);
        Ok::<AppConfig, String>(cfg.clone())
    })
    .await
    .map_err(|e| e.to_string())??;

    sync_tray_menu(&app, &cfg);
    let _ = app.emit("config-updated", &cfg);
    Ok(cfg)
}

#[tauri::command]
async fn refresh_from_hardware(app: tauri::AppHandle, _device_id: Option<String>, state: State<'_, ConfigState>) -> Result<AppConfig, String> {
    let state_arc = state.0.clone();
    let cfg = tauri::async_runtime::spawn_blocking(move || {
        let mut cfg = state_arc.lock().map_err(|e| e.to_string())?;
        config::refresh_device_from_hardware(&mut cfg, None);
        Ok::<AppConfig, String>(cfg.clone())
    })
    .await
    .map_err(|e| e.to_string())??;

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
            let vid = dev_info.vendor_id();
            let pid = dev_info.product_id();
            let is_nape = (vid == 0x3434 && pid == 0x0440)
                || dev_info.product_string().unwrap_or("").to_lowercase().contains("nape");

            if !is_nape {
                continue;
            }
            if dev_info.usage_page() != 0xff60 || dev_info.usage() != 0x0061 {
                continue;
            }

            output.push_str(&format!("=== Device: VID={:04X} PID={:04X} serial={} ===\n",
                vid, pid, dev_info.serial_number().unwrap_or("?")));

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
    let url = "https://launcher.keychron.com/";
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();

    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd").args(["/C", "start", "", url]).spawn();

    Ok(())
}

pub fn run() {
    let config_state = ConfigState::new();
    let initial_config = {
        let guard = config_state.0.lock().unwrap();
        guard.clone()
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .manage(config_state)
        .setup(move |app| {
            let quit_i = MenuItem::with_id(app, "quit", "終了 (Quit)", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "toggle_window", "ウィンドウの表示 / 非表示", true, None::<&str>)?;
            let launcher_i = MenuItem::with_id(app, "open_launcher", "🌐 Keychron Launcher (公式Web設定) を開く", true, None::<&str>)?;
            let sep1 = PredefinedMenuItem::separator(app)?;
            let sep2 = PredefinedMenuItem::separator(app)?;
            let sep3 = PredefinedMenuItem::separator(app)?;

            // 8 Layer Items (Layer 0 to 7) directly in main menu
            let mut layer_items = Vec::new();
            for i in 0..8u8 {
                let item = MenuItem::with_id(app, &format!("layer_{}", i), &format!("Layer {}", i), true, None::<&str>)?;
                layer_items.push(item);
            }

            // Trackball Controls (Scroll Toggle, Gesture Toggle)
            let scroll_item = MenuItem::with_id(app, "toggle_scroll", "📜 ボールスクロール", true, None::<&str>)?;
            let gesture_item = MenuItem::with_id(app, "toggle_gesture", "🖐️ ジェスチャー機能", true, None::<&str>)?;

            // DPI Submenu Items (Official Keychron Presets: 400 / 800 / 1800 / 3200 / 4000)
            let dpi_preset_values: Vec<u16> = vec![400, 800, 1800, 3200, 4000];
            let mut dpi_items = Vec::new();
            for dpi in &dpi_preset_values {
                let item = MenuItem::with_id(app, &format!("dpi_{}", dpi), &format!("{} DPI", dpi), true, None::<&str>)?;
                dpi_items.push((*dpi, item));
            }

            let dpi_item_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> =
                dpi_items.iter().map(|(_, item)| item as &dyn tauri::menu::IsMenuItem<tauri::Wry>).collect();
            let dpi_sub = Submenu::with_items(app, "🎯 DPI 設定", true, &dpi_item_refs)?;

            let mut menu_items: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = Vec::new();
            menu_items.push(&show_i);
            menu_items.push(&launcher_i);
            menu_items.push(&sep1);

            for item in &layer_items {
                menu_items.push(item as &dyn tauri::menu::IsMenuItem<tauri::Wry>);
            }

            menu_items.push(&sep2);
            menu_items.push(&scroll_item);
            menu_items.push(&gesture_item);
            menu_items.push(&dpi_sub);
            menu_items.push(&sep3);
            menu_items.push(&quit_i);

            let menu = Menu::with_items(app, &menu_items)?;

            let tray_state = TrayMenuState {
                layer_items: Mutex::new(layer_items),
                scroll_item: Mutex::new(scroll_item),
                gesture_item: Mutex::new(gesture_item),
                dpi_items: Mutex::new(dpi_items),
                dpi_submenu: Mutex::new(Some(dpi_sub)),
            };

            sync_tray_menu(app.handle(), &initial_config);
            app.manage(tray_state);

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
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
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    } else if id == "open_launcher" {
                        let url = "https://launcher.keychron.com/";
                        #[cfg(target_os = "macos")]
                        let _ = std::process::Command::new("open").arg(url).spawn();

                        #[cfg(target_os = "windows")]
                        let _ = std::process::Command::new("cmd").args(["/C", "start", "", url]).spawn();
                    } else if id.starts_with("layer_") {
                        if let Ok(layer_idx) = id.trim_start_matches("layer_").parse::<u8>() {
                            let app_handle = app_handle.clone();
                            tauri::async_runtime::spawn(async move {
                                let state = app_handle.state::<ConfigState>();
                                let state_arc = state.0.clone();
                                let cfg_res = tauri::async_runtime::spawn_blocking(move || {
                                    let mut cfg = state_arc.lock().map_err(|e| e.to_string())?;
                                    cfg.device.active_layer = layer_idx;
                                    if cfg.device.is_connected {
                                        if let Ok(api) = hidapi::HidApi::new() {
                                            for dev_info in api.device_list() {
                                                let vid = dev_info.vendor_id();
                                                let pid = dev_info.product_id();
                                                let prod = dev_info.product_string().unwrap_or("").to_lowercase();
                                                if ((vid == 0x3434 && pid == 0x0440) || prod.contains("nape"))
                                                    && dev_info.usage_page() == 0xff60
                                                    && dev_info.usage() == 0x0061
                                                {
                                                    if let Ok(hid_dev) = dev_info.open_device(&api) {
                                                        let mut req = [0u8; 33];
                                                        req[0] = 0x00;
                                                        req[1] = 0xA7;
                                                        req[2] = 45;
                                                        req[3] = layer_idx + 1;
                                                        let _ = hid_dev.write(&req);
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    config::save_config_to_file(&cfg);
                                    Ok::<AppConfig, String>(cfg.clone())
                                }).await;

                                if let Ok(Ok(cfg)) = cfg_res {
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
                                let mut cfg = state_arc.lock().map_err(|e| e.to_string())?;
                                let next_mode = !cfg.device.trackball_scroll_mode;
                                cfg.device.trackball_scroll_mode = next_mode;
                                if cfg.device.is_connected {
                                    if let Ok(api) = hidapi::HidApi::new() {
                                        for dev_info in api.device_list() {
                                            let vid = dev_info.vendor_id();
                                            let pid = dev_info.product_id();
                                            let prod = dev_info.product_string().unwrap_or("").to_lowercase();
                                            if ((vid == 0x3434 && pid == 0x0440) || prod.contains("nape"))
                                                && dev_info.usage_page() == 0xff60
                                                && dev_info.usage() == 0x0061
                                            {
                                                if let Ok(hid_dev) = dev_info.open_device(&api) {
                                                    config::set_trackball_force_gesture_scroll_official(&hid_dev, cfg.device.trackball_gesture_mode, next_mode);
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                                config::save_config_to_file(&cfg);
                                Ok::<AppConfig, String>(cfg.clone())
                            }).await;

                            if let Ok(Ok(cfg)) = cfg_res {
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
                                let mut cfg = state_arc.lock().map_err(|e| e.to_string())?;
                                let next_mode = !cfg.device.trackball_gesture_mode;
                                cfg.device.trackball_gesture_mode = next_mode;
                                if cfg.device.is_connected {
                                    if let Ok(api) = hidapi::HidApi::new() {
                                        for dev_info in api.device_list() {
                                            let vid = dev_info.vendor_id();
                                            let pid = dev_info.product_id();
                                            let prod = dev_info.product_string().unwrap_or("").to_lowercase();
                                            if ((vid == 0x3434 && pid == 0x0440) || prod.contains("nape"))
                                                && dev_info.usage_page() == 0xff60
                                                && dev_info.usage() == 0x0061
                                            {
                                                if let Ok(hid_dev) = dev_info.open_device(&api) {
                                                    config::set_trackball_force_gesture_scroll_official(&hid_dev, next_mode, cfg.device.trackball_scroll_mode);
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                                config::save_config_to_file(&cfg);
                                Ok::<AppConfig, String>(cfg.clone())
                            }).await;

                            if let Ok(Ok(cfg)) = cfg_res {
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
                                    let mut cfg = state_arc.lock().map_err(|e| e.to_string())?;
                                    cfg.device.pointer_dpi = dpi_val;
                                    if cfg.device.is_connected {
                                        if let Ok(api) = hidapi::HidApi::new() {
                                            for dev_info in api.device_list() {
                                                let vid = dev_info.vendor_id();
                                                let pid = dev_info.product_id();
                                                let prod = dev_info.product_string().unwrap_or("").to_lowercase();
                                                if ((vid == 0x3434 && pid == 0x0440) || prod.contains("nape"))
                                                    && dev_info.usage_page() == 0xff60
                                                    && dev_info.usage() == 0x0061
                                                {
                                                    if let Ok(hid_dev) = dev_info.open_device(&api) {
                                                        config::set_pointer_dpi_official(&hid_dev, dpi_val);
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    config::save_config_to_file(&cfg);
                                    Ok::<AppConfig, String>(cfg.clone())
                                }).await;

                                if let Ok(Ok(cfg)) = cfg_res {
                                    sync_tray_menu(&app_handle, &cfg);
                                    let _ = app_handle.emit("config-updated", &cfg);
                                }
                            });
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = window.hide();
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
