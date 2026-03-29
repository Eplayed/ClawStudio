// scripts/generate-icons.js - Generate app icons for Tauri using Sharp
import sharp from 'sharp';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const sizes = [32, 128, 256, 512];

// SVG template for the icon
const svgIcon = (size) => `
<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}" viewBox="0 0 ${size} ${size}">
  <defs>
    <linearGradient id="bg" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#0a0e1a"/>
      <stop offset="100%" style="stop-color:#0f1629"/>
    </linearGradient>
    <linearGradient id="claw" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#06d6d6"/>
      <stop offset="100%" style="stop-color:#0891b2"/>
    </linearGradient>
  </defs>
  
  <!-- Background with rounded corners -->
  <rect width="${size}" height="${size}" rx="${Math.round(size * 0.15)}" fill="url(#bg)"/>
  
  <!-- Border glow -->
  <rect x="${size * 0.02}" y="${size * 0.02}" width="${size * 0.96}" height="${size * 0.96}" 
        rx="${Math.round(size * 0.14)}" fill="none" stroke="rgba(6,214,214,0.3)" stroke-width="${Math.max(1, size * 0.005)}"/>
  
  <!-- Claw pad -->
  <ellipse cx="${size/2}" cy="${size/2 + size * 0.08}" rx="${size * 0.15}" ry="${size * 0.1}" fill="url(#claw)" opacity="0.9"/>
  
  <!-- Three claw marks -->
  <path d="M${size/2 - size * 0.12},${size/2 - size * 0.18} 
           Q${size/2 - size * 0.1},${size/2 - size * 0.28} ${size/2 - size * 0.06},${size/2 - size * 0.32} 
           Q${size/2 - size * 0.04},${size/2 - size * 0.28} ${size/2 - size * 0.08},${size/2 - size * 0.18}" fill="url(#claw)"/>
  <path d="M${size/2},${size/2 - size * 0.2} 
           Q${size/2 + size * 0.02},${size/2 - size * 0.32} ${size/2 + size * 0.06},${size/2 - size * 0.36} 
           Q${size/2 + size * 0.08},${size/2 - size * 0.32} ${size/2 + size * 0.04},${size/2 - size * 0.2}" fill="url(#claw)"/>
  <path d="M${size/2 + size * 0.12},${size/2 - size * 0.18} 
           Q${size/2 + size * 0.14},${size/2 - size * 0.28} ${size/2 + size * 0.18},${size/2 - size * 0.32} 
           Q${size/2 + size * 0.2},${size/2 - size * 0.28} ${size/2 + size * 0.16},${size/2 - size * 0.18}" fill="url(#claw)"/>
  
  <!-- CS text -->
  <text x="${size/2}" y="${size/2 + size * 0.38}" text-anchor="middle" 
        font-family="JetBrains Mono, monospace" font-size="${size * 0.12}" font-weight="700" 
        fill="#06d6d6" opacity="0.8">CS</text>
</svg>
`;

async function generateIcons() {
  // Go up from scripts/ to project root, then into src-tauri/icons
  const iconsDir = path.join(__dirname, '..', 'src-tauri', 'icons');
  
  // Ensure directory exists
  if (!fs.existsSync(iconsDir)) {
    fs.mkdirSync(iconsDir, { recursive: true });
  }
  
  for (const size of sizes) {
    const svg = svgIcon(size);
    const filename = size === 128 ? '128x128.png' : `${size}x${size}.png`;
    
    await sharp(Buffer.from(svg))
      .resize(size, size)
      .png()
      .toFile(path.join(iconsDir, filename));
    
    console.log(`Generated ${filename}`);
  }
  
  // Generate 128x128@2x (256px for HiDPI)
  await sharp(Buffer.from(svgIcon(256)))
    .resize(256, 256)
    .png()
    .toFile(path.join(iconsDir, '128x128@2x.png'));
  console.log('Generated 128x128@2x.png');
  
  // Generate icon.icns placeholder (macOS) - sharp can create ICNS
  const icnsBuffer = await sharp(Buffer.from(svgIcon(512)))
    .resize(512, 512)
    .png()
    .toBuffer();
  
  // For .ico and .icns, we'll create PNG files that can be converted
  // Note: Tauri's `tauri icon` command can generate these properly
  console.log('\n✅ PNG icons generated!');
  console.log('\nTo generate .ico and .icns files, run:');
  console.log('  pnpm tauri icon src-tauri/icons/icon.svg');
}

generateIcons().catch(console.error);
