use hidapi::HidApi;

#[path = "../config.rs"]
mod config;

fn main() {
    let api = HidApi::new().unwrap();
    let mut target_devices = Vec::new();
    for dev in api.device_list() {
        let vid = dev.vendor_id();
        let pid = dev.product_id();
        let product_str = dev.product_string().unwrap_or("Unknown");
        if (vid == 0x3434 && pid == 0x0440 || product_str.to_lowercase().contains("nape")) && dev.usage_page() == 0xff60 && dev.usage() == 0x0061 {
            target_devices.push(dev.clone());
        }
    }
    if target_devices.is_empty() { return; }
    let device = target_devices[0].open_device(&api).unwrap();

    for layer in [0u8, 1u8, 2u8] {
        println!("=== MAPPED BUTTONS ON LAYER {} ===", layer);
        if let Some(mappings) = config::read_layer_button_mappings(&device, layer) {
            for m in mappings {
                println!("  Button ID {:02} ({}): {} [{}]", m.button_id, m.name, m.description, m.key_code);
            }
        }
    }
}
