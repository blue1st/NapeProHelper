import os
import struct
import zlib

FONT_5x7 = {
    0: [
        "01110",
        "10001",
        "10011",
        "10101",
        "11001",
        "10001",
        "01110",
    ],
    1: [
        "00100",
        "01100",
        "00100",
        "00100",
        "00100",
        "00100",
        "01110",
    ],
    2: [
        "01110",
        "10001",
        "00001",
        "00010",
        "00100",
        "01000",
        "11111",
    ],
    3: [
        "11110",
        "00001",
        "00001",
        "01110",
        "00001",
        "00001",
        "11110",
    ],
    4: [
        "00010",
        "00110",
        "01010",
        "10010",
        "11111",
        "00010",
        "00010",
    ],
    5: [
        "11111",
        "10000",
        "11110",
        "00001",
        "00001",
        "10001",
        "01110",
    ],
    6: [
        "01110",
        "10000",
        "11110",
        "10001",
        "10001",
        "10001",
        "01110",
    ],
    7: [
        "11111",
        "00001",
        "00010",
        "00100",
        "01000",
        "01000",
        "01000",
    ],
}

def parse_png_rgba(path):
    with open(path, 'rb') as f:
        data = f.read()
    assert data[:8] == b'\x89PNG\r\n\x1a\n'
    idx = 8
    width, height = 0, 0
    idat = b''
    while idx < len(data):
        length = struct.unpack('>I', data[idx:idx+4])[0]
        ctype = data[idx+4:idx+8]
        cdata = data[idx+8:idx+8+length]
        idx += 8 + length + 4
        if ctype == b'IHDR':
            width, height, bitdepth, colortype, comp, filt, inter = struct.unpack('>IIBBBBB', cdata)
            assert bitdepth == 8 and colortype == 6, f"Expected 8-bit RGBA, got bitdepth={bitdepth}, colortype={colortype}"
        elif ctype == b'IDAT':
            idat += cdata
    decomp = zlib.decompress(idat)
    
    stride = 1 + width * 4
    pixels = []
    prev_row = [0] * (width * 4)
    
    for y in range(height):
        row_data = decomp[y * stride : (y + 1) * stride]
        filter_type = row_data[0]
        raw_bytes = list(row_data[1:])
        unfiltered = [0] * (width * 4)
        
        for i in range(width * 4):
            x = raw_bytes[i]
            a = unfiltered[i - 4] if i >= 4 else 0
            b = prev_row[i]
            c = prev_row[i - 4] if i >= 4 else 0
            
            if filter_type == 0:
                val = x
            elif filter_type == 1:
                val = (x + a) & 0xFF
            elif filter_type == 2:
                val = (x + b) & 0xFF
            elif filter_type == 3:
                val = (x + ((a + b) >> 1)) & 0xFF
            elif filter_type == 4:
                p = a + b - c
                pa = abs(p - a)
                pb = abs(p - b)
                pc = abs(p - c)
                if pa <= pb and pa <= pc:
                    pr = a
                elif pb <= pc:
                    pr = b
                else:
                    pr = c
                val = (x + pr) & 0xFF
            else:
                val = x
            unfiltered[i] = val
            
        prev_row = unfiltered
        row_pixels = []
        for x in range(width):
            r = unfiltered[x * 4]
            g = unfiltered[x * 4 + 1]
            b = unfiltered[x * 4 + 2]
            alpha = unfiltered[x * 4 + 3]
            row_pixels.append([r, g, b, alpha])
        pixels.append(row_pixels)
        
    return width, height, pixels

def save_png_rgba(width, height, pixels, output_path):
    raw_data = bytearray()
    for y in range(height):
        raw_data.append(0)
        for x in range(width):
            r, g, b, a = pixels[y][x]
            raw_data.extend([r & 0xFF, g & 0xFF, b & 0xFF, a & 0xFF])
            
    compressed = zlib.compress(bytes(raw_data), 9)
    
    png = bytearray(b'\x89PNG\r\n\x1a\n')
    ihdr_data = struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)
    ihdr_crc = zlib.crc32(b'IHDR' + ihdr_data)
    png.extend(struct.pack('>I', len(ihdr_data)) + b'IHDR' + ihdr_data + struct.pack('>I', ihdr_crc))
    
    idat_crc = zlib.crc32(b'IDAT' + compressed)
    png.extend(struct.pack('>I', len(compressed)) + b'IDAT' + compressed + struct.pack('>I', idat_crc))
    
    iend_crc = zlib.crc32(b'IEND')
    png.extend(struct.pack('>I', 0) + b'IEND' + struct.pack('>I', iend_crc))
    
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    with open(output_path, 'wb') as f:
        f.write(png)

def save_raw_rgba(width, height, pixels, output_path):
    raw_data = bytearray()
    for y in range(height):
        for x in range(width):
            r, g, b, a = pixels[y][x]
            raw_data.extend([r & 0xFF, g & 0xFF, b & 0xFF, a & 0xFF])
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    with open(output_path, 'wb') as f:
        f.write(raw_data)

def generate_layer_icons():
    base_w, base_h, base_pixels = parse_png_rgba('/Users/t-kawasaki/src/desktop-apps/napepro-helper/src-tauri/icons/32x32.png')
    
    out_dir = '/Users/t-kawasaki/src/desktop-apps/napepro-helper/src-tauri/icons/tray'
    os.makedirs(out_dir, exist_ok=True)
    
    for layer in range(8):
        new_pixels = [[list(p) for p in row] for row in base_pixels]
        
        # Badge dimensions & position (15x15 bottom-right at 16,16)
        bx0, by0 = 16, 16
        bw, bh = 15, 15
        
        bg_r, bg_g, bg_b = 15, 23, 42       # Slate 900
        border_r, border_g, border_b = 99, 102, 241 # Indigo 500
        text_r, text_g, text_b = 255, 255, 255     # Pure White
        
        for dy in range(bh):
            for dx in range(bw):
                px = bx0 + dx
                py = by0 + dy
                if px >= base_w or py >= base_h:
                    continue
                
                is_border = (
                    dx == 0 or dx == bw - 1 or dy == 0 or dy == bh - 1 or
                    (dx == 1 and dy in (1, bh-2)) or
                    (dx == bw-2 and dy in (1, bh-2))
                )
                
                if (dx == 0 and dy in (0, bh-1)) or (dx == bw-1 and dy in (0, bh-1)):
                    continue
                elif is_border:
                    new_pixels[py][px] = [border_r, border_g, border_b, 255]
                else:
                    new_pixels[py][px] = [bg_r, bg_g, bg_b, 255]
                    
        font_data = FONT_5x7[layer]
        fx0 = bx0 + (bw - 5) // 2
        fy0 = by0 + (bh - 7) // 2
        
        for f_y, line in enumerate(font_data):
            for f_x, ch in enumerate(line):
                if ch == '1':
                    px = fx0 + f_x
                    py = fy0 + f_y
                    if 0 <= px < base_w and 0 <= py < base_h:
                        new_pixels[py][px] = [text_r, text_g, text_b, 255]
                        
        png_path = os.path.join(out_dir, f'layer_{layer}.png')
        rgba_path = os.path.join(out_dir, f'layer_{layer}.rgba')
        save_png_rgba(base_w, base_h, new_pixels, png_path)
        save_raw_rgba(base_w, base_h, new_pixels, rgba_path)
        print(f'Generated: {png_path} & {rgba_path}')

if __name__ == '__main__':
    generate_layer_icons()
