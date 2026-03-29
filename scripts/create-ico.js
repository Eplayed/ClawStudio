// scripts/create-ico.js - Create basic ICO file from PNG
import sharp from 'sharp';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// ICO file format constants
const ICONDIR = (count) => Buffer.from([
  0, 0,       // Reserved
  1, 0,       // Type: 1 = ICO
  count & 0xFF, (count >> 8) & 0xFF  // Image count
]);

const ICONDIRENTRY = (width, height, bytes, offset) => {
  const w = width >= 256 ? 0 : width;
  const h = height >= 256 ? 0 : height;
  return Buffer.from([
    w,                    // Width
    h,                    // Height
    0,                    // Color palette
    0,                    // Reserved
    1, 0,                 // Color planes
    32, 0,                // Bits per pixel
    bytes & 0xFF, (bytes >> 8) & 0xFF, (bytes >> 16) & 0xFF, (bytes >> 24) & 0xFF,  // Size
    offset & 0xFF, (offset >> 8) & 0xFF, (offset >> 16) & 0xFF, (offset >> 24) & 0xFF  // Offset
  ]);
};

async function createIco() {
  const iconsDir = path.join(__dirname, '..', 'src-tauri', 'icons');
  
  // Read PNG files
  const sizes = [16, 32, 48, 64, 128, 256];
  const pngBuffers = [];
  
  for (const size of sizes) {
    const pngPath = path.join(iconsDir, size === 256 ? '256x256.png' : 
                                    size === 128 ? '128x128.png' :
                                    size === 64 ? '128x128.png' :
                                    size === 48 ? '128x128.png' :
                                    size === 32 ? '32x32.png' : '32x32.png');
    
    if (fs.existsSync(pngPath)) {
      let buffer = await sharp(pngPath)
        .resize(size, size)
        .png()
        .toBuffer();
      
      // Convert to DIB format for ICO (BGRA, bottom-up)
      const img = await sharp(buffer).raw().toBuffer({ resolveWithObject: true });
      const { data, info } = img;
      
      // Flip vertically and convert RGBA to BGRA
      const dibData = Buffer.alloc(data.length);
      for (let y = 0; y < info.height; y++) {
        const srcOffset = y * info.width * 4;
        const dstOffset = (info.height - 1 - y) * info.width * 4;
        for (let x = 0; x < info.width; x++) {
          const srcIdx = srcOffset + x * 4;
          const dstIdx = dstOffset + x * 4;
          dibData[dstIdx] = data[srcIdx + 2];     // B
          dibData[dstIdx + 1] = data[srcIdx + 1]; // G
          dibData[dstIdx + 2] = data[srcIdx];     // R
          dibData[dstIdx + 3] = data[srcIdx + 3]; // A
        }
      }
      
      // Create PNG again with the flipped data (simpler approach)
      buffer = await sharp(pngPath).resize(size, size).png().toBuffer();
      pngBuffers.push({ size, buffer });
    }
  }
  
  if (pngBuffers.length === 0) {
    console.log('No PNG files found');
    return;
  }
  
  // Build ICO file
  const headerSize = 6 + pngBuffers.length * 16;
  let offset = headerSize;
  const entries = [];
  const datas = [];
  
  for (const { size, buffer } of pngBuffers) {
    entries.push(ICONDIRENTRY(size, size, buffer.length, offset));
    datas.push(buffer);
    offset += buffer.length;
  }
  
  const ico = Buffer.concat([
    ICONDIR(pngBuffers.length),
    ...entries,
    ...datas
  ]);
  
  fs.writeFileSync(path.join(iconsDir, 'icon.ico'), ico);
  console.log('Generated icon.ico');
}

createIco().catch(console.error);
