// scripts/create-icns.js - Create basic ICNS file from PNG
// Note: This creates a simplified ICNS that may not be fully compatible
// For production, use `iconutil` on macOS or `pnpm tauri icon`

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import sharp from 'sharp';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// ICNS format: 4-byte magic + 4-byte size + chunks
const ICNS_MAKER = (type, data) => {
  const size = data.length + 8;
  return Buffer.concat([
    Buffer.from(type, 'ascii'),
    Buffer.from([size >> 24, (size >> 16) & 0xFF, (size >> 8) & 0xFF, size & 0xFF]),
    data
  ]);
};

async function createIcns() {
  const iconsDir = path.join(__dirname, '..', 'src-tauri', 'icons');
  
  // ICNS uses specific chunk types for different sizes
  // ic07 = 128x128, ic08 = 256x256, ic09 = 512x512, ic10 = 1024x1024
  // ic04 = 16x16, ic05 = 32x32, ic06 = 48x48, ic13 = 64x64
  
  const chunks = [];
  
  // Read 512x512 PNG
  const png512 = await sharp(path.join(iconsDir, '512x512.png'))
    .png()
    .toBuffer();
  
  // ic09 = 512x512
  chunks.push(ICNS_MAKER('ic09', png512));
  
  // ic08 = 256x256
  const png256 = await sharp(path.join(iconsDir, '256x256.png'))
    .png()
    .toBuffer();
  chunks.push(ICNS_MAKER('ic08', png256));
  
  // ic07 = 128x128
  const png128 = await sharp(path.join(iconsDir, '128x128.png'))
    .png()
    .toBuffer();
  chunks.push(ICNS_MAKER('ic07', png128));
  
  // Combine all chunks
  const totalSize = chunks.reduce((sum, c) => sum + c.length, 0) + 8;
  const icns = Buffer.concat([
    Buffer.from('icns', 'ascii'),
    Buffer.from([totalSize >> 24, (totalSize >> 16) & 0xFF, (totalSize >> 8) & 0xFF, totalSize & 0xFF]),
    ...chunks
  ]);
  
  fs.writeFileSync(path.join(iconsDir, 'icon.icns'), icns);
  console.log('Generated icon.icns');
}

createIcns().catch(console.error);
