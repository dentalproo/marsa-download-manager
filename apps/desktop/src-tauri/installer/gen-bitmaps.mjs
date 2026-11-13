// Generate Marsa installer brand bitmaps (NSIS MUI 24-bit BMP).
// Sidebar 164x314 + header 164x63. Flat navy + harbour blue, anchor mark.
// Pure Node, no deps.
import { writeFileSync } from "node:fs";
import { join } from "node:path";

const NAVY = [0x0e, 0x13, 0x1c];
const SURFACE = [0x14, 0x1b, 0x27];
const BLUE = [0x3b, 0x82, 0xf6];
const BLUE_SOFT = [0x16, 0x24, 0x3c];
const WHITE = [0xe9, 0xee, 0xf8];
const MUTED = [0x95, 0xa1, 0xb6];

function hex([r, g, b]) {
  return (r << 16) | (g << 8) | b;
}

function make(width, height) {
  const px = new Uint8Array(width * height * 3);
  const set = (x, y, rgb) => {
    if (x < 0 || y < 0 || x >= width || y >= height) return;
    const o = (y * width + x) * 3;
    px[o] = rgb[2]; px[o + 1] = rgb[1]; px[o + 2] = rgb[0]; // B,G,R for BMP
  };
  const rect = (x0, y0, x1, y1, rgb) => {
    for (let y = y0; y < y1; y++) for (let x = x0; x < x1; x++) set(x, y, rgb);
  };
  const circle = (cx, cy, r, rgb, ring) => {
    for (let y = cy - r - 1; y <= cy + r + 1; y++) for (let x = cx - r - 1; x <= cx + r + 1; x++) {
      const d = Math.hypot(x - cx, y - cy);
      if (ring ? Math.abs(d - r) < 2.5 : d <= r) set(x, y, rgb);
    }
  };
  const line = (x0, y0, x1, y1, w, rgb) => {
    const steps = Math.ceil(Math.hypot(x1 - x0, y1 - y0)) * 2;
    for (let i = 0; i <= steps; i++) {
      const x = x0 + ((x1 - x0) * i) / steps;
      const y = y0 + ((y1 - y0) * i) / steps;
      circle(Math.round(x), Math.round(y), Math.round(w / 2), rgb, false);
    }
  };
  return { px, set, rect, circle, line };
}

function toBmp(width, height, px) {
  const pad = (4 - ((width * 3) % 4)) % 4;
  const data = Buffer.alloc(14 + 40 + (width * 3 + pad) * height);
  data.write("BM", 0);
  data.writeUInt32LE(data.length, 2);
  data.writeUInt32LE(54, 10);
  data.writeUInt32LE(40, 14);
  data.writeInt32LE(width, 18);
  data.writeInt32LE(height, 22);
  data.writeUInt16LE(1, 26);
  data.writeUInt16LE(24, 28);
  let p = 54;
  for (let y = height - 1; y >= 0; y--) {
    px.copy(data, p, y * width * 3, (y + 1) * width * 3);
    p += width * 3 + pad;
  }
  return data;
}

// ---- Sidebar 164x314 ----
{
  const w = 164, h = 314;
  const { px, rect, circle, line } = make(w, h);
  rect(0, 0, w, h, NAVY);
  rect(0, 0, w, 6, BLUE);
  // soft card behind mark
  rect(42, 78, 122, 158, SURFACE);
  // anchor mark (ring, shaft, crossbar, flukes)
  circle(82, 96, 7, WHITE, true);
  line(82, 103, 82, 140, 5, BLUE);
  line(66, 118, 98, 140 - 22, 5, WHITE); // crossbar approx
  line(64, 112, 100, 112, 0, WHITE);
  line(66, 112, 98, 112, 5, WHITE);
  line(58, 130, 82, 142, 5, BLUE);
  line(106, 130, 82, 142, 5, BLUE);
  // waves
  line(62, 150, 76, 150, 3, BLUE);
  line(88, 150, 102, 150, 3, BLUE);
  // title bars (abstract, no font needed)
  rect(52, 186, 112, 194, WHITE);
  rect(66, 202, 98, 208, BLUE);
  rect(62, 222, 102, 226, MUTED);
  rect(66, 231, 98, 235, MUTED);
  rect(0, 300, w, h, BLUE_SOFT);
  writeFileSync(join(process.cwd(), "welcome-sidebar.bmp"), toBmp(w, h, Buffer.from(px)));
}

// ---- Header 164x63 ----
{
  const w = 164, h = 63;
  const { px, rect, circle, line } = make(w, h);
  rect(0, 0, w, h, NAVY);
  rect(0, 0, 6, h, BLUE);
  circle(30, 32, 8, WHITE, true);
  line(30, 40, 30, 52, 4, BLUE);
  line(20, 44, 40, 44, 4, WHITE);
  line(18, 50, 30, 54, 4, BLUE);
  line(42, 50, 30, 54, 4, BLUE);
  rect(52, 20, 118, 28, WHITE);
  rect(52, 34, 96, 39, MUTED);
  writeFileSync(join(process.cwd(), "header.bmp"), toBmp(w, h, Buffer.from(px)));
}
console.log("installer bitmaps written");
