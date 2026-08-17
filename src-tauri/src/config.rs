#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonMapping {
    pub button_id: u8,
    pub name: String,
    pub action_type: String,
    pub key_code: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceProfile {
    pub id: String,
    pub name: String,
    pub interface_type: String,
    pub serial_number: String,
    pub is_connected: bool,
    pub active_layer: u8,
    pub octashift_angle: u16,
    #[serde(default)]
    pub layer_octashift_angles: HashMap<u8, u16>,
    pub pointer_dpi: u16,
    pub trackball_scroll_mode: bool,
    #[serde(default)]
    pub trackball_gesture_mode: bool,
    pub layer_names: HashMap<u8, String>,
    pub button_mappings: HashMap<u8, Vec<ButtonMapping>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSwitchRule {
    pub id: String,
    pub name: String,
    pub app_name: String,
    pub target_layer: u8,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub autostart: bool,
    pub minimize_to_tray: bool,
    pub show_notifications: bool,
    #[serde(default)]
    pub auto_switch_enabled: bool,
    #[serde(default)]
    pub auto_switch_default_layer: Option<u8>,
    #[serde(default)]
    pub auto_switch_rules: Vec<AutoSwitchRule>,
    pub device: DeviceProfile,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            autostart: true,
            minimize_to_tray: true,
            show_notifications: true,
            auto_switch_enabled: false,
            auto_switch_default_layer: Some(0),
            auto_switch_rules: Vec::new(),
            device: create_default_device("dev-nape-01", "Keychron Nape Pro", "USB / 2.4GHz", "", false),
        }
    }
}

pub fn is_layer_customized(_layer: u8, sub_codes: &[u16]) -> bool {
    !sub_codes.iter().all(|&c| c == 0)
}

/// Official Keychron Active Layer Query (0xA3 / 163 KC_GET_CURRENT_LAYER)
/// Strictly Read-Only. Keychron 0xA3 protocol returns 1-based layer index (1 for Layer 0, 2 for Layer 1, etc.)
pub fn read_active_layer_official(device: &hidapi::HidDevice) -> Option<u8> {
    let mut req = [0u8; 33];
    req[0] = 0x00;
    req[1] = 0xA3; // KC_GET_CURRENT_LAYER
    req[2] = 0x00;
    req[3] = 0xFF; // 255

    if device.write(&req).is_ok() {
        let mut buf = [0u8; 64];
        if let Ok(n) = device.read_timeout(&mut buf, 200) {
            let start_idx = if n >= 3 && buf[0] == 0xA3 {
                0
            } else if n >= 4 && buf[1] == 0xA3 {
                1
            } else {
                999
            };

            if start_idx != 999 {
                let default_layer = buf[start_idx + 1];
                let overlay_layer = buf[start_idx + 2];
                let raw_active = if overlay_layer != 255 && overlay_layer != 0 {
                    default_layer.max(overlay_layer)
                } else {
                    default_layer
                };

                // Convert 1-based hardware layer index (1..8) to 0-based internal layer index (0..7)
                let active = if raw_active >= 1 && raw_active <= 8 {
                    raw_active - 1
                } else if raw_active < 8 {
                    raw_active
                } else {
                    0
                };
                if active < 8 {
                    return Some(active);
                }
            }
        }
    }
    None
}

/// Official Keychron Nape Pro OctaShift Angle Query (0xA7 / 167 KC_MISC_CMD_GROUP)
pub fn read_octashift_angle_official(device: &hidapi::HidDevice, layer: u8) -> Option<u16> {
    let mut req = [0u8; 33];
    req[0] = 0x00;
    req[1] = 0xA7; // KC_MISC_CMD_GROUP
    req[2] = 56;   // KC_USER_CMD_NAPE_GET_LAYER_ORI (56 / 0x38)
    req[3] = layer; // 0-based layer index (0..7)

    if device.write(&req).is_ok() {
        let mut buf = [0u8; 64];
        if let Ok(n) = device.read_timeout(&mut buf, 200) {
            let start_idx = if n >= 4 && buf[0] == 0xA7 && buf[1] == 56 {
                0
            } else if n >= 5 && buf[1] == 0xA7 && buf[2] == 56 {
                1
            } else {
                999
            };

            if start_idx != 999 {
                // Response format: [0xA7, 56, angle_div_45, ...] or [0xA7, 56, layer_id, angle_div_45]
                let raw_val = if buf[start_idx + 2] < 8 {
                    buf[start_idx + 2]
                } else if buf[start_idx + 3] < 8 {
                    buf[start_idx + 3]
                } else {
                    0
                };
                let angle = (raw_val as u16) * 45;
                if angle < 360 {
                    return Some(angle);
                }
            }
        }
    }
    None
}

/// Official Keychron Nape Pro OctaShift Angle Writer (0xA7 / 167 KC_MISC_CMD_GROUP)
pub fn set_octashift_angle_official(device: &hidapi::HidDevice, layer: u8, angle: u16) -> bool {
    let angle_val = ((angle % 360) / 45) as u8;

    // 1. KC_USER_CMD_NAPE_SET_LAYER_ORI (57 / 0x39) - 0-based layer index
    let mut req1 = [0u8; 33];
    req1[0] = 0x00;
    req1[1] = 0xA7; // KC_MISC_CMD_GROUP
    req1[2] = 57;   // KC_USER_CMD_NAPE_SET_LAYER_ORI
    req1[3] = layer; // 0-based layer index (0..7)
    req1[4] = angle_val;
    let res1 = device.write(&req1).is_ok();

    // 2. KC_USER_CMD_NAPE_SET_ORI (52 / 0x34) - Immediate Sensor Angle Apply
    let mut req2 = [0u8; 33];
    req2[0] = 0x00;
    req2[1] = 0xA7;
    req2[2] = 52;   // KC_USER_CMD_NAPE_SET_ORI
    req2[3] = angle_val;
    let _ = device.write(&req2);

    res1
}

/// Official Keychron Nape Pro & VIA Pointer DPI Writer
pub fn set_pointer_dpi_official(device: &hidapi::HidDevice, dpi: u16) -> bool {
    let dpi_lo = (dpi & 0xFF) as u8;
    let dpi_hi = ((dpi >> 8) & 0xFF) as u8;

    let dpi_index = match dpi {
        400 => 0u8,
        800 => 1u8,
        1800 => 2u8,
        3200 => 3u8,
        4000 => 4u8,
        0..=599 => 0u8,
        600..=1299 => 1u8,
        1300..=2499 => 2u8,
        2500..=3599 => 3u8,
        _ => 4u8,
    };

    // 1. Keychron Nape Pro KC_USER_CMD_NAPE_SET_DPI_VALUE (35 / 0x23) - Set DPI value for dpi_index
    let mut req_val = [0u8; 33];
    req_val[0] = 0x00;
    req_val[1] = 0xA7; // 167 (KC_MISC_CMD_GROUP)
    req_val[2] = 35;   // KC_USER_CMD_NAPE_SET_DPI_VALUE
    req_val[3] = dpi_index;
    req_val[4] = dpi_lo;
    req_val[5] = dpi_hi;
    let _ = device.write(&req_val);

    // 2. Keychron Nape Pro KC_USER_CMD_NAPE_SET_DPI (34 / 0x22) - Select active DPI level index
    let mut req_idx = [0u8; 33];
    req_idx[0] = 0x00;
    req_idx[1] = 0xA7;
    req_idx[2] = 34;
    req_idx[3] = dpi_index;
    let res_idx = device.write(&req_idx).is_ok();

    // 3. Keychron Nape Pro KC_USER_CMD_NAPE_SET_CUSTOM_DPI_VALUE (55 / 0x37) - u16 LE (dpi_lo, dpi_hi)
    let mut req_cust = [0u8; 33];
    req_cust[0] = 0x00;
    req_cust[1] = 0xA7;
    req_cust[2] = 55;
    req_cust[3] = dpi_lo;
    req_cust[4] = dpi_hi;
    let _ = device.write(&req_cust);

    // 4. VIA Cmd 0x0D (Set Custom Value)
    let mut req_via = [0u8; 33];
    req_via[0] = 0x00;
    req_via[1] = 0x0D;
    req_via[2] = (dpi / 100) as u8;
    let _ = device.write(&req_via);

    res_idx
}

/// Official Keychron Nape Pro Force Gesture & Scroll Mode Writer (0xA7 / 167 Subcommand 50)
/// Packet structure: [0x00, 167, 50, gesture, scroll]
pub fn set_trackball_force_gesture_scroll_official(device: &hidapi::HidDevice, gesture: bool, scroll: bool) -> bool {
    let mut req = [0u8; 33];
    req[0] = 0x00;
    req[1] = 0xA7; // 167 (KC_MISC_CMD_GROUP)
    req[2] = 50;   // Set_Force_Gesture_Scroll (0x32)
    req[3] = if gesture { 1 } else { 0 }; // gesture flag (kt[2])
    req[4] = if scroll { 1 } else { 0 };  // scroll flag (kt[3])
    device.write(&req).is_ok()
}

/// Official Keychron Nape Pro Force Gesture & Scroll Mode Reader (0xA7 / 167 Subcommand 51)
pub fn read_trackball_force_gesture_scroll_official(device: &hidapi::HidDevice) -> Option<(bool, bool)> {
    let mut req = [0u8; 33];
    req[0] = 0x00;
    req[1] = 0xA7; // 167
    req[2] = 51;   // Get_Force_Gesture_Scroll (0x33)

    if device.write(&req).is_ok() {
        let mut buf = [0u8; 64];
        if let Ok(n) = device.read_timeout(&mut buf, 200) {
            let start_idx = if n >= 4 && buf[0] == 0xA7 && buf[1] == 51 {
                0
            } else if n >= 5 && buf[1] == 0xA7 && buf[2] == 51 {
                1
            } else {
                999
            };

            if start_idx != 999 {
                let gesture = buf[start_idx + 2] != 0;
                let scroll = buf[start_idx + 3] != 0;
                return Some((gesture, scroll));
            }
        }
    }
    None
}

/// Official Keychron Nape Pro Custom DPI Reader (0xA7 / 167 Subcommand 54)
pub fn read_custom_dpi_official(device: &hidapi::HidDevice) -> Option<u16> {
    let mut req = [0u8; 33];
    req[0] = 0x00;
    req[1] = 0xA7; // 167
    req[2] = 54;   // Get_Custom_Dpi_Value (0x36)

    if device.write(&req).is_ok() {
        let mut buf = [0u8; 64];
        if let Ok(n) = device.read_timeout(&mut buf, 200) {
            let start_idx = if n >= 4 && buf[0] == 0xA7 && buf[1] == 54 {
                0
            } else if n >= 5 && buf[1] == 0xA7 && buf[2] == 54 {
                1
            } else {
                999
            };

            if start_idx != 999 {
                let dpi_lo = buf[start_idx + 2] as u16;
                let dpi_hi = buf[start_idx + 3] as u16;
                let dpi = dpi_lo | (dpi_hi << 8);
                if dpi >= 100 && dpi <= 8000 {
                    return Some(dpi);
                }
            }
        }
    }
    None
}

/// Official Keychron Nape Pro Active Pointer DPI Reader (Subcommand 33 -> Subcommand 36)
pub fn read_active_pointer_dpi_official(device: &hidapi::HidDevice) -> Option<u16> {
    // 1. Get active DPI level index (Subcommand 33 / 0x21)
    let mut req33 = [0u8; 33];
    req33[0] = 0x00;
    req33[1] = 0xA7; // 167
    req33[2] = 33;   // KC_USER_CMD_NAPE_GET_DPI

    let mut active_index: Option<u8> = None;
    if device.write(&req33).is_ok() {
        let mut buf = [0u8; 64];
        if let Ok(n) = device.read_timeout(&mut buf, 200) {
            let si = if n >= 3 && buf[0] == 0xA7 && buf[1] == 33 {
                0
            } else if n >= 4 && buf[1] == 0xA7 && buf[2] == 33 {
                1
            } else {
                999
            };
            if si != 999 {
                active_index = Some(buf[si + 2]);
            }
        }
    }

    // 2. Read DPI value for the active level index (Subcommand 36 / 0x24)
    if let Some(idx) = active_index {
        let mut req36 = [0u8; 33];
        req36[0] = 0x00;
        req36[1] = 0xA7; // 167
        req36[2] = 36;   // KC_USER_CMD_NAPE_GET_DPI_VALUE
        req36[3] = idx;

        if device.write(&req36).is_ok() {
            let mut buf = [0u8; 64];
            if let Ok(n) = device.read_timeout(&mut buf, 200) {
                let si = if n >= 4 && buf[0] == 0xA7 && buf[1] == 36 {
                    0
                } else if n >= 5 && buf[1] == 0xA7 && buf[2] == 36 {
                    1
                } else {
                    999
                };
                if si != 999 {
                    let dpi_lo = buf[si + 2] as u16;
                    let dpi_hi = buf[si + 3] as u16;
                    let dpi = dpi_lo | (dpi_hi << 8);
                    if dpi >= 100 && dpi <= 8000 {
                        return Some(dpi);
                    }
                }
            }
        }
    }

    // 3. Fallback to Custom DPI (Subcommand 54 / 0x36)
    if let Some(dpi) = read_custom_dpi_official(device) {
        return Some(dpi);
    }

    // 4. Fallback to VIA Cmd 0x0C
    let mut req_via = [0u8; 33];
    req_via[0] = 0x00;
    req_via[1] = 0x0C;
    if device.write(&req_via).is_ok() {
        let mut buf = [0u8; 64];
        if let Ok(n) = device.read_timeout(&mut buf, 200) {
            let si = if n >= 2 && buf[0] == 0x0C { 0 } else if n >= 3 && buf[1] == 0x0C { 1 } else { 999 };
            if si != 999 {
                let raw = buf[si + 1] as u16;
                let dpi = match raw {
                    0 => 400,
                    1 => 800,
                    2 => 1800,
                    3 => 3200,
                    4 => 4000,
                    _ => raw * 100,
                };
                if dpi >= 100 && dpi <= 8000 {
                    return Some(dpi);
                }
            }
        }
    }

    None
}

pub fn get_eeprom_offset_and_row_start_for_layer(layer: u8) -> (u16, usize) {
    if layer == 0 {
        (0u16, 7usize)
    } else {
        let chunk_idx = ((layer as u16) + 1) / 2;
        let start_slot = if layer % 2 == 1 { 0usize } else { 7usize };
        (chunk_idx * 28, start_slot)
    }
}

pub fn parse_layer_button_mappings_from_bytes(keycodes: &[u16], layer: u8) -> Option<Vec<ButtonMapping>> {
    if keycodes.len() < 14 {
        return None;
    }
    let (_, row_start) = get_eeprom_offset_and_row_start_for_layer(layer);

    let get_slot_keycode = |slot_idx: usize, default_code: u16| -> u16 {
        let target_idx = row_start + slot_idx;
        let code = keycodes.get(target_idx).copied().unwrap_or(0);
        if code != 0 {
            return code;
        }
        default_code
    };

    let mut mappings = Vec::new();

    // Check if default factory 0° upright orientation markers are present on Layer 0
    let is_factory_0deg_defaults = keycodes.get(9).copied() == Some(0x00D4) && keycodes.get(10).copied() == Some(0x00D5) && layer == 0;

    // Check if default factory uncustomized keymap is present on Layer 4..7
    let first_code = keycodes.get(row_start).copied().unwrap_or(0);
    let is_layer4_defaults = layer >= 4 && (first_code == 0x522A || first_code == 0x0000);

    // Button 03 (G3, Top-Left) (Button ID 5) -> Slot 0
    let g3_c = if is_layer4_defaults { 0x522B } else { get_slot_keycode(0, 0x522B) };
    let (act5, code5, desc5) = parse_qmk_keycode(g3_c);
    mappings.push(ButtonMapping { button_id: 5, name: "ボタン 03 (G3)".into(), action_type: act5, key_code: code5, description: desc5 });

    // Button 04 (G4, Top-Right) (Button ID 6) -> Slot 1
    let g4_c = if is_layer4_defaults { 0x522A } else { get_slot_keycode(1, 0x522A) };
    let (act6, code6, desc6) = parse_qmk_keycode(g4_c);
    mappings.push(ButtonMapping { button_id: 6, name: "ボタン 04 (G4)".into(), action_type: act6, key_code: code6, description: desc6 });

    // Button 01 (G1, Bottom-Left) (Button ID 3) -> Slot 2
    let g1_c = if is_factory_0deg_defaults || is_layer4_defaults { 0x00D2 } else { get_slot_keycode(2, 0x00D2) };
    let (act3, code3, desc3) = match g1_c {
        0x00D2 | 0x0002 => ("key".into(), "Browser_Back".into(), "戻る".into()),
        _ => parse_qmk_keycode(g1_c),
    };
    mappings.push(ButtonMapping { button_id: 3, name: "ボタン 01 (G1)".into(), action_type: act3, key_code: code3, description: desc3 });

    // Button 02 (G2, Bottom-Right) (Button ID 4) -> Slot 3
    let g2_c = if is_factory_0deg_defaults {
        0x00D1
    } else if is_layer4_defaults {
        0x7E2D
    } else {
        get_slot_keycode(3, 0x7E2D)
    };
    let (act4, code4, desc4) = parse_qmk_keycode(g2_c);
    mappings.push(ButtonMapping { button_id: 4, name: "ボタン 02 (G2)".into(), action_type: act4, key_code: code4, description: desc4 });

    // Button M1 (Button ID 1) -> Slot 4 (0x0001, 0x00D1, 0x7E29 all represent Left Click)
    let m1_c = if is_factory_0deg_defaults || is_layer4_defaults { 0x0001 } else { get_slot_keycode(4, 0x0001) };
    let (act1, code1, desc1) = match m1_c {
        0x0001 | 0x00D1 | 0x7E29 => ("key".into(), "Click_Left".into(), "左クリック".into()),
        _ => parse_qmk_keycode(m1_c),
    };
    mappings.push(ButtonMapping { button_id: 1, name: "ボタン M1".into(), action_type: act1, key_code: code1, description: desc1 });

    // Button M2 (Button ID 2) -> Slot 5 (0x0002, 0x00D2, 0x7E29 all represent Right Click)
    let m2_c = if is_factory_0deg_defaults || is_layer4_defaults { 0x0002 } else { get_slot_keycode(5, 0x0002) };
    let (act2, code2, desc2) = match m2_c {
        0x0002 | 0x00D2 | 0x7E29 => ("key".into(), "Click_Right".into(), "右クリック".into()),
        _ => parse_qmk_keycode(m2_c),
    };
    mappings.push(ButtonMapping { button_id: 2, name: "ボタン M2".into(), action_type: act2, key_code: code2, description: desc2 });

    // Scroll Ring / Jog Dial (Button ID 7: 時計回り CW / Button ID 8: 反時計回り CCW) -> Slot 6
    let ring_c = if is_layer4_defaults { 0x7E2B } else { get_slot_keycode(6, 0x522B) };
    let (act7, code7, desc7, act8, code8, desc8) = match ring_c {
        0x7E2B => (
            "media".into(), "Vol_Up".into(), "Volume Up".into(),
            "media".into(), "Vol_Down".into(), "Volume Down".into(),
        ),
        0x522B => {
            if layer == 2 {
                (
                    "octashift".into(), "G(KC_PPLS)".into(), "G(KC_PPLS)".into(),
                    "octashift".into(), "G(KC_PMNS)".into(), "G(KC_PMNS)".into(),
                )
            } else if layer == 3 {
                (
                    "octashift".into(), "C(KC_PPLS)".into(), "C(KC_PPLS)".into(),
                    "octashift".into(), "C(KC_PMNS)".into(), "C(KC_PMNS)".into(),
                )
            } else {
                (
                    "key".into(), "Scroll_Down".into(), "下スクロール".into(),
                    "key".into(), "Scroll_Up".into(), "上にスクロール".into(),
                )
            }
        }
        _ => {
            let (a7, c7, d7) = parse_qmk_keycode(ring_c);
            let (a8, c8, d8) = match ring_c {
                // Cmd/Ctrl/Shift + (= / 0x2E) -> pair with - / 0x2D
                0x082E | 0x012E | 0x022E | 0x042E => parse_qmk_keycode(ring_c - 1),
                // Cmd/Ctrl/Shift - (- / 0x2D) -> pair with = / 0x2E
                0x082D | 0x012D | 0x022D | 0x042D => parse_qmk_keycode(ring_c + 1),
                // Cmd/Ctrl/Shift Keypad + (0x57) -> pair with Keypad - (0x56)
                0x0857 | 0x0157 | 0x0257 | 0x0457 => parse_qmk_keycode(ring_c - 1),
                // Cmd/Ctrl/Shift Keypad - (0x56) -> pair with Keypad + (0x57)
                0x0856 | 0x0156 | 0x0256 | 0x0456 => parse_qmk_keycode(ring_c + 1),
                _ => parse_qmk_keycode(ring_c),
            };
            (a7, c7, d7, a8, c8, d8)
        }
    };
    mappings.push(ButtonMapping { button_id: 7, name: "スクロールリング (時計回り ↻)".into(), action_type: act7, key_code: code7, description: desc7 });
    mappings.push(ButtonMapping { button_id: 8, name: "スクロールリング (反時計回り ↺)".into(), action_type: act8, key_code: code8, description: desc8 });

    Some(mappings)
}

pub fn read_layer_button_mappings(device: &hidapi::HidDevice, layer: u8) -> Option<Vec<ButtonMapping>> {
    let (offset_bytes, _) = get_eeprom_offset_and_row_start_for_layer(layer);

    let mut req_map = [0u8; 33];
    req_map[0] = 0x00;
    req_map[1] = 0x12; // DYNAMIC_KEYMAP_GET_BUFFER
    req_map[2] = ((offset_bytes >> 8) & 0xFF) as u8;
    req_map[3] = (offset_bytes & 0xFF) as u8;
    req_map[4] = 28; // Read 28 bytes (14 keycodes)

    if device.write(&req_map).is_ok() {
        let mut buf = [0u8; 64];
        if let Ok(n) = device.read_timeout(&mut buf, 200) {
            let start_idx = if n >= 4 && buf[0] == 0x12 {
                0
            } else if n >= 5 && buf[1] == 0x12 {
                1
            } else {
                999
            };

            if start_idx != 999 {
                let data_idx = start_idx + 4;
                let mut keycodes: Vec<u16> = Vec::new();
                for i in (data_idx..n.min(data_idx + 28)).step_by(2) {
                    if i + 1 < n {
                        let code = u16::from_be_bytes([buf[i], buf[i + 1]]);
                        keycodes.push(code);
                    }
                }

                return parse_layer_button_mappings_from_bytes(&keycodes, layer);
            }
        }
    }
    None
}



pub fn create_default_device(
    id: &str,
    name: &str,
    interface_type: &str,
    serial_number: &str,
    is_connected: bool,
) -> DeviceProfile {
    let mut layer_names = HashMap::new();
    let mut layer_octashift_angles = HashMap::new();
    let mut button_mappings: HashMap<u8, Vec<ButtonMapping>> = HashMap::new();

    for i in 0..8u8 {
        layer_names.insert(i, format!("Layer {}", i));
        layer_octashift_angles.insert(i, 0);

        let mappings = vec![
            ButtonMapping { button_id: 1, name: "ボタン M1".to_string(), action_type: "key".to_string(), key_code: "Click_Left".to_string(), description: "左クリック".to_string() },
            ButtonMapping { button_id: 2, name: "ボタン M2".to_string(), action_type: "key".to_string(), key_code: "Click_Right".to_string(), description: "右クリック".to_string() },
            ButtonMapping { button_id: 3, name: "ボタン 01 (G1)".to_string(), action_type: "key".to_string(), key_code: "Browser_Back".to_string(), description: "戻る".to_string() },
            ButtonMapping { button_id: 4, name: "ボタン 02 (G2)".to_string(), action_type: "key".to_string(), key_code: "Cycle_DPI".to_string(), description: "Cycle DPI".to_string() },
            ButtonMapping { button_id: 5, name: "ボタン 03 (G3)".to_string(), action_type: "octashift".to_string(), key_code: "Ball_Scroll".to_string(), description: "ボールスクロール".to_string() },
            ButtonMapping { button_id: 6, name: "ボタン 04 (G4)".to_string(), action_type: "octashift".to_string(), key_code: "Switch_8Dir".to_string(), description: "8方向を切り替え".to_string() },
            ButtonMapping { button_id: 7, name: "スクロールリング (上 / Vol Up)".to_string(), action_type: "media".to_string(), key_code: "Vol_Up".to_string(), description: "Volume Up".to_string() },
            ButtonMapping { button_id: 8, name: "スクロールリング (下 / Vol Down)".to_string(), action_type: "media".to_string(), key_code: "Vol_Down".to_string(), description: "Volume Down".to_string() },
        ];

        button_mappings.insert(i, mappings);
    }

    DeviceProfile {
        id: id.to_string(),
        name: name.to_string(),
        interface_type: interface_type.to_string(),
        serial_number: serial_number.to_string(),
        is_connected,
        active_layer: 0,
        octashift_angle: 0,
        layer_octashift_angles,
        pointer_dpi: 1600,
        trackball_scroll_mode: false,
        trackball_gesture_mode: false,
        layer_names,
        button_mappings,
    }
}

pub fn get_config_path() -> PathBuf {
    let mut dir = if let Some(appdata) = std::env::var_os("APPDATA") {
        PathBuf::from(appdata)
    } else if let Some(home) = std::env::var_os("HOME") {
        let mut p = PathBuf::from(home);
        p.push("Library");
        p.push("Application Support");
        p
    } else {
        PathBuf::from(".")
    };
    dir.push("napepro-helper");
    let _ = fs::create_dir_all(&dir);
    dir.push("config.json");
    dir
}

pub fn parse_qmk_keycode(code: u16) -> (String, String, String) {
    if code == 0x0000 {
        return ("key".into(), "KC_NO".into(), "なし".into());
    }

    // 1. Direct match for Mouse / Special hardware / Keychron / Language functions
    match code {
        0x0001 | 0x00CD => return ("key".into(), "Click_Left".into(), "左クリック".into()),
        0x0002 | 0x00CE => return ("key".into(), "Click_Right".into(), "右クリック".into()),
        0x0004 | 0x00CF => return ("key".into(), "Click_Middle".into(), "中クリック".into()),
        0x00D0 | 0x00D2 => return ("key".into(), "Browser_Back".into(), "戻る".into()),
        0x00D1 | 0x00D3 => return ("key".into(), "Browser_Forward".into(), "進む".into()),
        0x00D4 => return ("key".into(), "KC_LNG1".into(), "かな".into()),
        0x00D5 => return ("key".into(), "KC_LNG2".into(), "英数".into()),
        0x00D6 => return ("key".into(), "KC_LNG3".into(), "Language 3".into()),
        0x00D7 => return ("key".into(), "KC_LNG4".into(), "Language 4".into()),
        0x00D8 => return ("key".into(), "KC_LNG5".into(), "Language 5".into()),
        0x0088 => return ("key".into(), "KC_KANA".into(), "かな".into()),
        0x008A => return ("key".into(), "KC_HENK".into(), "変換".into()),
        0x008B => return ("key".into(), "KC_MHEN".into(), "無変換".into()),
        0x008C => return ("key".into(), "KC_KATA".into(), "カタカナ".into()),
        0x008D => return ("key".into(), "KC_HIRG".into(), "ひらがな".into()),
        0x0C52 => return ("key".into(), "Scroll_Left".into(), "左スクロール".into()),
        0x0C51 => return ("key".into(), "Scroll_Right".into(), "右スクロール".into()),
        0x522A => return ("octashift".into(), "Switch_8Dir".into(), "8方向を切り替え".into()),
        0x522B => return ("octashift".into(), "Ball_Scroll".into(), "ボールスクロール".into()),
        0x7E29 => return ("key".into(), "Default_Action".into(), "標準機能".into()),
        0x7E2B => return ("media".into(), "Vol_Up".into(), "Volume Up".into()),
        0x7E2C => return ("media".into(), "Vol_Down".into(), "Volume Down".into()),
        0x7E2D => return ("key".into(), "Cycle_DPI".into(), "Cycle DPI".into()),
        _ => {}
    }

    // 2. QMK Layer Operations
    if (0x5220..=0x523F).contains(&code) {
        let layer = (code - 0x5220) as u8;
        return ("layer_shift".into(), format!("MO({})", layer), format!("Layer {} Shift", layer));
    }
    if (0x5200..=0x521F).contains(&code) {
        let layer = (code - 0x5200) as u8;
        return ("layer_toggle".into(), format!("TG({})", layer), format!("Layer {} Toggle", layer));
    }

    // 3. VIA / QMK Macros
    if (0x7700..=0x770F).contains(&code) {
        let macro_id = (code - 0x7700) as u8;
        return ("macro".into(), format!("MACRO({})", macro_id), format!("マクロ {}", macro_id));
    }
    if (0x5F00..=0x5F0F).contains(&code) {
        let macro_id = (code - 0x5F00) as u8;
        return ("macro".into(), format!("MACRO({})", macro_id), format!("マクロ {}", macro_id));
    }

    if code >= 0x5000 {
        return ("key".into(), format!("0x{:04X}", code), format!("Key (0x{:04X})", code));
    }

    // 4. Modifiers & Basic Keycode Parsing
    let mod_byte = (code >> 8) as u8;
    let basic_code = (code & 0xFF) as u8;

    let basic_name = match basic_code {
        0x04..=0x1D => {
            let ch = (b'A' + (basic_code - 0x04)) as char;
            Box::leak(format!("KC_{}", ch).into_boxed_str()) as &str
        }
        0x1E..=0x26 => {
            let ch = (b'1' + (basic_code - 0x1E)) as char;
            Box::leak(format!("KC_{}", ch).into_boxed_str()) as &str
        }
        0x27 => "KC_0",
        0x28 => "KC_ENT",
        0x29 => "KC_ESC",
        0x2A => "KC_BSPC",
        0x2B => "KC_TAB",
        0x2C => "KC_SPC",
        0x2D => "KC_MINS",
        0x2E => "KC_EQL",
        0x2F => "KC_LBRC",
        0x30 => "KC_RBRC",
        0x31 => "KC_BSLS",
        0x33 => "KC_SCLN",
        0x34 => "KC_QUOT",
        0x35 => "KC_GRV",
        0x36 => "KC_COMM",
        0x37 => "KC_DOT",
        0x38 => "KC_SLSH",
        0x3A..=0x45 => {
            let f_num = basic_code - 0x3A + 1;
            Box::leak(format!("KC_F{}", f_num).into_boxed_str()) as &str
        }
        0x4F => "KC_RGHT",
        0x50 => "KC_LEFT",
        0x51 => "KC_DOWN",
        0x52 => "KC_UP",
        0x54 => "KC_PSLS",
        0x55 => "KC_PAST",
        0x56 => "KC_PMNS",
        0x57 => "KC_PPLS",
        0x58 => "KC_PENT",
        0x59 => "KC_P1",
        0x5A => "KC_P2",
        0x5B => "KC_P3",
        0x5C => "KC_P4",
        0x5D => "KC_P5",
        0x5E => "KC_P6",
        0x5F => "KC_P7",
        0x60 => "KC_P8",
        0x61 => "KC_P9",
        0x62 => "KC_P0",
        0x63 => "KC_PDOT",
        _ => "",
    };

    let basic_friendly = match basic_code {
        0x04..=0x1D => {
            let ch = (b'A' + (basic_code - 0x04)) as char;
            Box::leak(ch.to_string().into_boxed_str()) as &str
        }
        0x1E..=0x26 => {
            let ch = (b'1' + (basic_code - 0x1E)) as char;
            Box::leak(ch.to_string().into_boxed_str()) as &str
        }
        0x27 => "0",
        0x28 => "Enter",
        0x29 => "Escape",
        0x2A => "Backspace",
        0x2B => "Tab",
        0x2C => "Space",
        0x2D => "-",
        0x2E => "=",
        0x2F => "[",
        0x30 => "]",
        0x31 => "\\",
        0x33 => ";",
        0x34 => "'",
        0x35 => "`",
        0x36 => ",",
        0x37 => ".",
        0x38 => "/",
        0x3A..=0x45 => {
            let f_num = basic_code - 0x3A + 1;
            Box::leak(format!("F{}", f_num).into_boxed_str()) as &str
        }
        0x4F => "→",
        0x50 => "←",
        0x51 => "↓",
        0x52 => "↑",
        0x54 => "Keypad /",
        0x55 => "Keypad *",
        0x56 => "Keypad -",
        0x57 => "Keypad +",
        0x58 => "Keypad Enter",
        0x59 => "Keypad 1",
        0x5A => "Keypad 2",
        0x5B => "Keypad 3",
        0x5C => "Keypad 4",
        0x5D => "Keypad 5",
        0x5E => "Keypad 6",
        0x5F => "Keypad 7",
        0x60 => "Keypad 8",
        0x61 => "Keypad 9",
        0x62 => "Keypad 0",
        0x63 => "Keypad .",
        _ => "",
    };

    if mod_byte > 0 && !basic_name.is_empty() {
        let is_gui = (mod_byte & 0x08) != 0 || (mod_byte & 0x80) != 0;
        let is_shift = (mod_byte & 0x02) != 0 || (mod_byte & 0x20) != 0;
        let is_alt = (mod_byte & 0x04) != 0 || (mod_byte & 0x40) != 0;
        let is_ctrl = (mod_byte & 0x01) != 0 || (mod_byte & 0x10) != 0;

        let qmk_prefix = match (is_ctrl, is_shift, is_alt, is_gui) {
            (true, false, false, false) => "C",
            (false, true, false, false) => "S",
            (false, false, true, false) => "A",
            (false, false, false, true) => "G",
            (true, true, false, false) => "C(S",
            (true, false, true, false) => "C(A",
            (true, false, false, true) => "C(G",
            (false, true, true, false) => "S(A",
            (false, true, false, true) => "S(G",
            (false, false, true, true) => "A(G",
            (true, true, true, false) => "MEH",
            (true, true, true, true) => "HYPR",
            _ => "",
        };

        let mut mods = Vec::new();
        if is_gui { mods.push(if cfg!(target_os = "macos") { "Cmd" } else { "Win" }); }
        if is_alt { mods.push(if cfg!(target_os = "macos") { "Option" } else { "Alt" }); }
        if is_shift { mods.push("Shift"); }
        if is_ctrl { mods.push("Ctrl"); }

        let desc = format!("{} + {}", mods.join(" + "), basic_friendly);

        if !qmk_prefix.is_empty() {
            let kc = if qmk_prefix.contains('(') {
                format!("{}({}))", qmk_prefix, basic_name)
            } else {
                format!("{}({})", qmk_prefix, basic_name)
            };
            return ("key".into(), kc, desc);
        } else {
            let kc = format!("{}+{}", mods.join("+"), basic_name);
            return ("key".into(), kc, desc);
        }
    }

    if !basic_friendly.is_empty() {
        return ("key".into(), basic_name.into(), basic_friendly.into());
    }

    ("key".into(), format!("0x{:04X}", code), format!("Key (0x{:04X})", code))
}

pub fn is_target_nape_device(dev_info: &hidapi::DeviceInfo) -> bool {
    let vid = dev_info.vendor_id();
    let pid = dev_info.product_id();
    let prod = dev_info.product_string().unwrap_or("").to_lowercase();

    // Must be Keychron Vendor (0x3434) AND specifically Nape Pro (PID 0x0440 OR product string containing "nape")
    // Strictly prevents matching other Keychron keyboards (e.g. Keychron Q8, Q1, K2, etc.)
    let is_keychron = vid == 0x3434 || prod.contains("keychron");
    let is_nape_product = pid == 0x0440 || prod.contains("nape");
    let is_raw_hid = dev_info.usage_page() == 0xff60 && dev_info.usage() == 0x0061;

    is_keychron && is_nape_product && is_raw_hid
}

pub fn scan_hid_devices(config: &mut AppConfig) -> bool {
    let mut nape_found = false;

    if let Ok(api) = hidapi::HidApi::new() {
        for dev_info in api.device_list() {
            if is_target_nape_device(dev_info) {
                if let Ok(device) = dev_info.open_device(&api) {
                    nape_found = true;
                    config.device.is_connected = true;

                    let raw_product = dev_info.product_string().unwrap_or("Keychron Nape Pro").to_string();
                    let prod_lower = raw_product.to_lowercase();
                    let interface_type = if prod_lower.contains("2.4g") || prod_lower.contains("receiver") || prod_lower.contains("dongle") {
                        "2.4GHz Wireless Mode".to_string()
                    } else if prod_lower.contains("bt") || prod_lower.contains("bluetooth") {
                        "Bluetooth Mode".to_string()
                    } else {
                        "USB Mode".to_string()
                    };

                    config.device.name = "Keychron Nape Pro".to_string();
                    config.device.interface_type = interface_type;

                    // Always refresh layer mappings from physical hardware on connection
                    let mut hw_mappings: HashMap<u8, Vec<ButtonMapping>> = HashMap::new();
                    for layer in 0..8u8 {
                        if let Some(mappings) = read_layer_button_mappings(&device, layer) {
                            hw_mappings.insert(layer, mappings);
                        }
                        std::thread::sleep(std::time::Duration::from_millis(5));
                    }
                    if !hw_mappings.is_empty() {
                        config.device.button_mappings = hw_mappings;
                    }

                    if let Some(hl) = read_active_layer_official(&device) {
                        config.device.active_layer = hl;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));

                    if let Some(ang) = read_octashift_angle_official(&device, config.device.active_layer) {
                        let active_l = config.device.active_layer;
                        let saved_angle = config.device.layer_octashift_angles.get(&active_l).copied();
                        if let Some(s_ang) = saved_angle {
                            config.device.octashift_angle = s_ang;
                        } else {
                            config.device.octashift_angle = ang;
                            config.device.layer_octashift_angles.insert(active_l, ang);
                        }
                    }

                    if let Some(dpi) = read_active_pointer_dpi_official(&device) {
                        config.device.pointer_dpi = dpi;
                    }

                    break; // Connect to the first active Nape Pro
                }
            }
        }
    }

    if !nape_found {
        config.device.is_connected = false;
    }

    nape_found
}

/// Force re-read all hardware settings (EEPROM keymaps, DPI, angle) from connected Nape Pro.
pub fn refresh_device_from_hardware(config: &mut AppConfig, _device_id: Option<&str>) -> bool {
    let api = match hidapi::HidApi::new() {
        Ok(a) => a,
        Err(_) => return false,
    };

    for dev_info in api.device_list() {
        if !is_target_nape_device(dev_info) {
            continue;
        }

        let device = match dev_info.open_device(&api) {
            Ok(d) => d,
            Err(_) => continue,
        };

        config.device.is_connected = true;

        let mut hw_mappings: HashMap<u8, Vec<ButtonMapping>> = HashMap::new();
        for layer in 0..8u8 {
            if let Some(mappings) = read_layer_button_mappings(&device, layer) {
                hw_mappings.insert(layer, mappings);
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        let hw_layer = read_active_layer_official(&device);
        let hw_dpi = read_active_pointer_dpi_official(&device);
        let hw_gesture_scroll = read_trackball_force_gesture_scroll_official(&device);

        let mut hw_layer_angles: HashMap<u8, u16> = HashMap::new();
        for layer in 0..8u8 {
            if let Some(ang) = read_octashift_angle_official(&device, layer) {
                hw_layer_angles.insert(layer, ang);
            }
        }
        let hw_angle = hw_layer_angles.get(&0).cloned().or_else(|| read_octashift_angle_official(&device, 0));

        if !hw_mappings.is_empty() {
            config.device.button_mappings = hw_mappings;
        }
        if !hw_layer_angles.is_empty() {
            config.device.layer_octashift_angles = hw_layer_angles;
        }
        if let Some(l) = hw_layer {
            config.device.active_layer = l;
        }
        if let Some(dpi) = hw_dpi {
            config.device.pointer_dpi = dpi;
        }
        if let Some(ang) = hw_angle {
            config.device.octashift_angle = ang;
        }
        if let Some((gest, scr)) = hw_gesture_scroll {
            config.device.trackball_gesture_mode = gest;
            config.device.trackball_scroll_mode = scr;
        }

        save_config_to_file(config);
        return true;
    }

    config.device.is_connected = false;
    false
}

pub fn load_config_from_file() -> AppConfig {
    let path = get_config_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            serde_json::from_str::<AppConfig>(&content).unwrap_or_default()
        } else {
            AppConfig::default()
        }
    } else {
        AppConfig::default()
    }
}

pub fn save_config_to_file(config: &AppConfig) {
    let path = get_config_path();
    if let Ok(json) = serde_json::to_string_pretty(config) {
        let _ = fs::write(path, json);
    }
}

#[derive(Clone)]
pub struct ConfigState(pub std::sync::Arc<Mutex<AppConfig>>);

impl ConfigState {
    pub fn new() -> Self {
        ConfigState(std::sync::Arc::new(Mutex::new(load_config_from_file())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_eeprom_offsets() {
        assert_eq!(get_eeprom_offset_and_row_start_for_layer(0), (0, 7));
        assert_eq!(get_eeprom_offset_and_row_start_for_layer(1), (28, 0));
        assert_eq!(get_eeprom_offset_and_row_start_for_layer(2), (28, 7));
        assert_eq!(get_eeprom_offset_and_row_start_for_layer(3), (56, 0));
        assert_eq!(get_eeprom_offset_and_row_start_for_layer(4), (56, 7));
        assert_eq!(get_eeprom_offset_and_row_start_for_layer(5), (84, 0));
        assert_eq!(get_eeprom_offset_and_row_start_for_layer(6), (84, 7));
        assert_eq!(get_eeprom_offset_and_row_start_for_layer(7), (112, 0));
    }

    #[test]
    fn test_layer_0_eeprom_dummy_parsing() {
        // Chunk 0 (28 bytes = 14 u16 keycodes)
        let chunk0: Vec<u16> = vec![
            0x522A, 0x0000, 0x00D4, 0x0000, 0x00D1, 0x00D2, 0x522B, // Row 0
            0x032B, 0x012B, 0x00D4, 0x00D5, 0x00D1, 0x00D2, 0x522B, // Row 1 (Layer 0)
        ];
        let mappings = parse_layer_button_mappings_from_bytes(&chunk0, 0).expect("Layer 0 mapping failed");
        
        let m1 = mappings.iter().find(|m| m.button_id == 1).unwrap();
        assert_eq!(m1.description, "左クリック");

        let m2 = mappings.iter().find(|m| m.button_id == 2).unwrap();
        assert_eq!(m2.description, "右クリック");

        let ring_cw = mappings.iter().find(|m| m.button_id == 7).unwrap();
        assert_eq!(ring_cw.description, "下スクロール");

        let ring_ccw = mappings.iter().find(|m| m.button_id == 8).unwrap();
        assert_eq!(ring_ccw.description, "上にスクロール");
    }

    #[test]
    fn test_layer_1_eeprom_dummy_parsing() {
        // Chunk 1 (28 bytes) -> Layer 1 uses Row 0 (indices 0..6)
        let chunk1: Vec<u16> = vec![
            0x0A50, 0x0A4F, 0x0828, 0x0A28, 0x00D1, 0x00D2, 0x522B, // Row 0 (Layer 1)
            0x081D, 0x0A1D, 0x0A13, 0x0A08, 0x00D1, 0x00D2, 0x522B, // Row 1 (Layer 2)
        ];
        let mappings = parse_layer_button_mappings_from_bytes(&chunk1, 1).expect("Layer 1 mapping failed");

        let g3 = mappings.iter().find(|m| m.button_id == 5).unwrap();
        assert!(g3.description.contains("Cmd") || g3.description.contains("Win"));

        let m1 = mappings.iter().find(|m| m.button_id == 1).unwrap();
        assert_eq!(m1.description, "左クリック");
    }

    #[test]
    fn test_layer_2_eeprom_dummy_parsing() {
        // Chunk 1 (28 bytes) -> Layer 2 uses Row 1 (indices 7..13)
        let chunk1: Vec<u16> = vec![
            0x0A50, 0x0A4F, 0x0828, 0x0A28, 0x00D1, 0x00D2, 0x522B, // Row 0 (Layer 1)
            0x081D, 0x0A1D, 0x0A13, 0x0A08, 0x00D1, 0x00D2, 0x522B, // Row 1 (Layer 2)
        ];
        let mappings = parse_layer_button_mappings_from_bytes(&chunk1, 2).expect("Layer 2 mapping failed");

        let g3 = mappings.iter().find(|m| m.button_id == 5).unwrap();
        assert_eq!(g3.key_code, "G(KC_Z)");

        let ring_cw = mappings.iter().find(|m| m.button_id == 7).unwrap();
        assert_eq!(ring_cw.key_code, "G(KC_PPLS)");

        let ring_ccw = mappings.iter().find(|m| m.button_id == 8).unwrap();
        assert_eq!(ring_ccw.key_code, "G(KC_PMNS)");
    }

    #[test]
    fn test_layer_3_eeprom_dummy_parsing() {
        // Chunk 2 (28 bytes) -> Layer 3 uses Row 0 (indices 0..6)
        let chunk2: Vec<u16> = vec![
            0x011D, 0x031D, 0x0113, 0x0108, 0x00D1, 0x00D2, 0x522B, // Row 0 (Layer 3: C(KC_Z), C(S(KC_Z)), C(KC_P), C(KC_E))
            0x522A, 0x7E2B, 0x00D4, 0x7E2C, 0x00D1, 0x00D2, 0x522B, // Row 1 (Layer 4)
        ];
        let mappings = parse_layer_button_mappings_from_bytes(&chunk2, 3).expect("Layer 3 mapping failed");

        let g3 = mappings.iter().find(|m| m.button_id == 5).unwrap();
        assert_eq!(g3.key_code, "C(KC_Z)");

        let g4 = mappings.iter().find(|m| m.button_id == 6).unwrap();
        assert_eq!(g4.key_code, "C(S(KC_Z))");

        let ring_cw = mappings.iter().find(|m| m.button_id == 7).unwrap();
        assert_eq!(ring_cw.key_code, "C(KC_PPLS)");

        let ring_ccw = mappings.iter().find(|m| m.button_id == 8).unwrap();
        assert_eq!(ring_ccw.key_code, "C(KC_PMNS)");
    }

    #[test]
    fn test_layer_4_defaults_parsing() {
        // Chunk 2 (28 bytes) -> Layer 4 uses Row 1 (indices 7..13)
        let chunk2: Vec<u16> = vec![
            0x011D, 0x031D, 0x0113, 0x0108, 0x00D1, 0x00D2, 0x522B, // Row 0 (Layer 3)
            0x522A, 0x7E2B, 0x00D4, 0x7E2C, 0x00D1, 0x00D2, 0x522B, // Row 1 (Layer 4 defaults)
        ];
        let mappings = parse_layer_button_mappings_from_bytes(&chunk2, 4).expect("Layer 4 mapping failed");

        let g3 = mappings.iter().find(|m| m.button_id == 5).unwrap();
        assert_eq!(g3.description, "ボールスクロール");

        let g4 = mappings.iter().find(|m| m.button_id == 6).unwrap();
        assert_eq!(g4.description, "8方向を切り替え");

        let g1 = mappings.iter().find(|m| m.button_id == 3).unwrap();
        assert_eq!(g1.description, "戻る");

        let g2 = mappings.iter().find(|m| m.button_id == 4).unwrap();
        assert_eq!(g2.description, "Cycle DPI");

        let ring_cw = mappings.iter().find(|m| m.button_id == 7).unwrap();
        assert_eq!(ring_cw.description, "Volume Up");

        let ring_ccw = mappings.iter().find(|m| m.button_id == 8).unwrap();
        assert_eq!(ring_ccw.description, "Volume Down");
    }

    #[test]
    fn test_qmk_keycode_keypad_and_modifiers() {
        let (act, code, desc) = parse_qmk_keycode(0x0157); // Ctrl + KC_PPLS
        assert_eq!(act, "key");
        assert_eq!(code, "C(KC_PPLS)");
        assert_eq!(desc, "Ctrl + Keypad +");

        let (_, code_minus, _) = parse_qmk_keycode(0x0156); // Ctrl + KC_PMNS
        assert_eq!(code_minus, "C(KC_PMNS)");

        let (_, code_cmd_plus, _) = parse_qmk_keycode(0x0857); // Cmd + KC_PPLS
        assert_eq!(code_cmd_plus, "G(KC_PPLS)");

        let (_, code_csz, _) = parse_qmk_keycode(0x031D); // Ctrl + Shift + Z
        assert_eq!(code_csz, "C(S(KC_Z))");
    }
}
