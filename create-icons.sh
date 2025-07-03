#!/bin/bash

# Create icons directory
mkdir -p src-tauri/icons

# Create a simple 32x32 black PNG icon using Python
python3 << 'EOF'
import struct
import zlib

def create_png(width, height, filename):
    # PNG signature
    signature = b'\x89PNG\r\n\x1a\n'
    
    # IHDR chunk (8-bit RGBA)
    ihdr_data = struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)  # 6 = RGBA
    ihdr_crc = zlib.crc32(b'IHDR' + ihdr_data)
    ihdr = struct.pack('>I', 13) + b'IHDR' + ihdr_data + struct.pack('>I', ihdr_crc & 0xffffffff)
    
    # Create 8-bit style 'V' icon
    scanlines = b''
    bg_color = b'\x14\x14\x1e\xff'  # Dark background
    v_color = b'\x00\xff\x80\xff'   # Bright green
    highlight = b'\x80\xff\xc8\xff' # Light green
    
    for y in range(height):
        scanlines += b'\x00'  # filter byte
        for x in range(width):
            # Scale to 0-1
            nx = x / width
            ny = y / height
            
            # Create V shape
            if ny < 0.7:
                # Left diagonal
                if abs(nx - ny * 0.7) < 0.15 and nx < 0.5:
                    if abs(nx - ny * 0.7) < 0.05:
                        scanlines += highlight
                    else:
                        scanlines += v_color
                # Right diagonal  
                elif abs((1 - nx) - ny * 0.7) < 0.15 and nx > 0.5:
                    if abs((1 - nx) - ny * 0.7) < 0.05:
                        scanlines += highlight
                    else:
                        scanlines += v_color
                else:
                    scanlines += bg_color
            else:
                scanlines += bg_color
    
    compressed = zlib.compress(scanlines, 9)
    idat_crc = zlib.crc32(b'IDAT' + compressed)
    idat = struct.pack('>I', len(compressed)) + b'IDAT' + compressed + struct.pack('>I', idat_crc & 0xffffffff)
    
    # IEND chunk
    iend_crc = zlib.crc32(b'IEND')
    iend = struct.pack('>I', 0) + b'IEND' + struct.pack('>I', iend_crc & 0xffffffff)
    
    # Write PNG file
    with open(filename, 'wb') as f:
        f.write(signature + ihdr + idat + iend)
    print(f"Created {filename}")

# Create all required icon sizes
create_png(32, 32, 'src-tauri/icons/32x32.png')
create_png(128, 128, 'src-tauri/icons/128x128.png')
create_png(256, 256, 'src-tauri/icons/128x128@2x.png')
create_png(32, 32, 'src-tauri/icons/icon.png')

# Copy for Windows and macOS formats (they'll need proper conversion later)
import shutil
shutil.copy('src-tauri/icons/32x32.png', 'src-tauri/icons/icon.ico')
shutil.copy('src-tauri/icons/128x128.png', 'src-tauri/icons/icon.icns')

print("All icons created successfully!")
EOF

echo "Icons created in src-tauri/icons/"