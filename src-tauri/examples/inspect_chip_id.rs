use hidapi::HidApi;
use std::thread::sleep;
use std::time::Duration;

fn print_hex_and_ascii(label: &str, data: &[u8]) {
    let hex: Vec<String> = data.iter().map(|b| format!("{:02X}", b)).collect();
    let ascii: String = data
        .iter()
        .map(|&b| if b >= 32 && b <= 126 { b as char } else { '.' })
        .collect();
    println!("  {:<25}: [{}]  \"{}\"", label, hex.join(" "), ascii);
}

fn query_command(device: &hidapi::HidDevice, label: &str, cmd: u8, sub: Option<u8>) {
    let mut req = [0u8; 33];
    req[0] = 0x00; // Report ID
    req[1] = cmd;
    if let Some(s) = sub {
        req[2] = s;
    }

    if device.write(&req).is_ok() {
        sleep(Duration::from_millis(30));
        let mut buf = [0u8; 64];
        if let Ok(n) = device.read_timeout(&mut buf, 200) {
            print_hex_and_ascii(label, &buf[..n.min(32)]);
        } else {
            println!("  {:<25}: (read timeout)", label);
        }
    } else {
        println!("  {:<25}: (write failed)", label);
    }
}

fn main() {
    println!("==================================================");
    println!(" Nape Pro Device & Chip ID Inspection Tool");
    println!("==================================================\n");

    let api = match HidApi::new() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Failed to initialize HidApi: {}", e);
            return;
        }
    };

    let mut found_count = 0;

    for dev_info in api.device_list() {
        let vid = dev_info.vendor_id();
        let pid = dev_info.product_id();
        let prod = dev_info.product_string().unwrap_or("Unknown");

        let is_nape = (vid == 0x3434 && pid == 0x0440) || prod.to_lowercase().contains("nape");
        if !is_nape {
            continue;
        }

        if dev_info.usage_page() != 0xff60 || dev_info.usage() != 0x0061 {
            continue;
        }

        found_count += 1;
        println!("--- Device #{} ---", found_count);
        println!("  Manufacturer : {}", dev_info.manufacturer_string().unwrap_or("(none)"));
        println!("  Product      : {}", dev_info.product_string().unwrap_or("(none)"));
        println!("  Serial Number: {}", dev_info.serial_number().unwrap_or("(none)"));
        println!("  Path         : {:?}", dev_info.path());
        println!("  VID / PID    : 0x{:04X} / 0x{:04X}", vid, pid);
        println!();

        match dev_info.open_device(&api) {
            Ok(device) => {
                println!("--- Sending Vendor Commands ---");
                // 0xA1: Firmware Version
                query_command(&device, "0xA1 Firmware Ver", 0xA1, None);

                // 0xA2: Support Feature Flags
                query_command(&device, "0xA2 Support Feature", 0xA2, None);

                // 0xAB: KC_FACTORY / Chip ID queries
                query_command(&device, "0xAB Factory (sub 0)", 0xAB, Some(0x00));
                query_command(&device, "0xAB Factory (sub 1)", 0xAB, Some(0x01));
                query_command(&device, "0xAB Factory (sub 2)", 0xAB, Some(0x02));
                query_command(&device, "0xAB Factory (sub FF)", 0xAB, Some(0xFF));

                // 0xA7: Misc commands
                query_command(&device, "0xA7 Sub 0x20 (Get Ori)", 0xA7, Some(0x20));
                query_command(&device, "0xA7 Sub 0x24 (Get DPI)", 0xA7, Some(0x24));
                query_command(&device, "0xA7 Sub 0x2C (Get Profile)", 0xA7, Some(0x2C));

                // VIA EEPROM Offset 0..32
                println!("\n--- EEPROM Sample (Offset 0..28) ---");
                let mut req = [0u8; 33];
                req[0] = 0x00;
                req[1] = 0x12; // DYNAMIC_KEYMAP_GET_BUFFER
                req[2] = 0x00;
                req[3] = 0x00;
                req[4] = 28;
                if device.write(&req).is_ok() {
                    sleep(Duration::from_millis(30));
                    let mut buf = [0u8; 64];
                    if let Ok(n) = device.read_timeout(&mut buf, 200) {
                        print_hex_and_ascii("EEPROM Layer 0 Head", &buf[..n.min(32)]);
                    }
                }
                println!("\n");
            }
            Err(e) => {
                println!("  Failed to open device: {}\n", e);
            }
        }
    }

    if found_count == 0 {
        println!("No Nape Pro device found via VIA Raw HID (0xFF60 / 0x0061).");
        println!("Please make sure the device is plugged in.");
    } else {
        println!("Inspection completed. Total devices found: {}", found_count);
    }
}
