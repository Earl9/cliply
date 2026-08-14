// Generates the bitmap artwork used by the custom NSIS installer.
// Keep these resources on the same flat two-sheet mark as the app icon so the
// installer never presents a second, outdated visual identity.
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const COLORS = {
  coral: [0xff, 0x62, 0x57],
  mint: [0x22, 0xb7, 0x93],
  fold: [0xff, 0xdf, 0xc9],
  ink: [0x21, 0x28, 0x2d],
  white: [0xff, 0xff, 0xff],
  canvas: [0xf7, 0xf9, 0xfb],
  canvasWarm: [0xff, 0xfb, 0xfa],
  line: [0xe5, 0xea, 0xee],
};

function createCanvas(width, height, background) {
  return { width, height, pixels: new Uint8Array(width * height * 3).fill(0), background };
}

function setPixel(canvas, x, y, color, alpha = 1) {
  if (x < 0 || y < 0 || x >= canvas.width || y >= canvas.height || alpha <= 0) return;
  const offset = (y * canvas.width + x) * 3;
  const inverse = 1 - alpha;
  canvas.pixels[offset] = Math.round(color[0] * alpha + canvas.pixels[offset] * inverse);
  canvas.pixels[offset + 1] = Math.round(color[1] * alpha + canvas.pixels[offset + 1] * inverse);
  canvas.pixels[offset + 2] = Math.round(color[2] * alpha + canvas.pixels[offset + 2] * inverse);
}

function fillCanvas(canvas, color) {
  for (let y = 0; y < canvas.height; y += 1) {
    for (let x = 0; x < canvas.width; x += 1) setPixel(canvas, x, y, color);
  }
}

function roundedRect(canvas, x0, y0, x1, y1, radius, color, alpha = 1) {
  for (let y = Math.floor(y0); y < Math.ceil(y1); y += 1) {
    for (let x = Math.floor(x0); x < Math.ceil(x1); x += 1) {
      const cx = Math.max(x0 + radius, Math.min(x1 - radius, x + 0.5));
      const cy = Math.max(y0 + radius, Math.min(y1 - radius, y + 0.5));
      const distance = Math.hypot(x + 0.5 - cx, y + 0.5 - cy);
      setPixel(canvas, x, y, color, distance <= radius ? alpha : 0);
    }
  }
}

function triangle(canvas, points, color, alpha = 1) {
  const minX = Math.floor(Math.min(...points.map((point) => point[0])));
  const maxX = Math.ceil(Math.max(...points.map((point) => point[0])));
  const minY = Math.floor(Math.min(...points.map((point) => point[1])));
  const maxY = Math.ceil(Math.max(...points.map((point) => point[1])));
  const edge = (a, b, px, py) => (px - b[0]) * (a[1] - b[1]) - (a[0] - b[0]) * (py - b[1]);
  for (let y = minY; y <= maxY; y += 1) {
    for (let x = minX; x <= maxX; x += 1) {
      const px = x + 0.5;
      const py = y + 0.5;
      const values = [edge(points[0], points[1], px, py), edge(points[1], points[2], px, py), edge(points[2], points[0], px, py)];
      if (values.every((value) => value >= 0) || values.every((value) => value <= 0)) setPixel(canvas, x, y, color, alpha);
    }
  }
}

function drawMark(canvas, x, y, size) {
  const s = size / 100;
  const rect = (a, b, c, d, r, color, alpha = 1) => roundedRect(canvas, x + a * s, y + b * s, x + c * s, y + d * s, r * s, color, alpha);
  rect(29, 11, 85, 73, 11, COLORS.ink, 0.14);
  rect(28, 10, 85, 72, 11, COLORS.mint);
  rect(9, 25, 75, 91, 11, COLORS.ink, 0.18);
  rect(10, 23, 74, 90, 11, COLORS.coral);
  triangle(canvas, [[x + 54 * s, y + 25 * s], [x + 74 * s, y + 25 * s], [x + 74 * s, y + 45 * s]], COLORS.fold);
  if (size >= 46) {
    rect(22, 54, 57, 58, 2, COLORS.ink, 0.82);
    rect(22, 67, 49, 71, 2, COLORS.ink, 0.82);
  }
}

function encodeBmp(canvas) {
  const rowStride = Math.ceil((canvas.width * 3) / 4) * 4;
  const pixelBytes = rowStride * canvas.height;
  const header = Buffer.alloc(54);
  header.write("BM", 0, 2, "ascii");
  header.writeUInt32LE(54 + pixelBytes, 2);
  header.writeUInt32LE(54, 10);
  header.writeUInt32LE(40, 14);
  header.writeInt32LE(canvas.width, 18);
  header.writeInt32LE(canvas.height, 22);
  header.writeUInt16LE(1, 26);
  header.writeUInt16LE(24, 28);
  header.writeUInt32LE(0, 30);
  header.writeUInt32LE(pixelBytes, 34);
  header.writeInt32LE(2835, 38);
  header.writeInt32LE(2835, 42);

  const pixels = Buffer.alloc(pixelBytes);
  for (let y = 0; y < canvas.height; y += 1) {
    const sourceY = canvas.height - 1 - y;
    for (let x = 0; x < canvas.width; x += 1) {
      const source = (sourceY * canvas.width + x) * 3;
      const target = y * rowStride + x * 3;
      pixels[target] = canvas.pixels[source + 2];
      pixels[target + 1] = canvas.pixels[source + 1];
      pixels[target + 2] = canvas.pixels[source];
    }
  }
  return Buffer.concat([header, pixels]);
}

function writeArtwork(relativePath, canvas) {
  const target = join(repoRoot, relativePath);
  mkdirSync(dirname(target), { recursive: true });
  writeFileSync(target, encodeBmp(canvas));
  console.log(`${relativePath.padEnd(46)} ${canvas.width}x${canvas.height}`);
}

const logo = createCanvas(96, 96, COLORS.white);
fillCanvas(logo, COLORS.white);
drawMark(logo, 9, 8, 80);
writeArtwork("src-tauri/installer/cliply-logo-96.bmp", logo);

const header = createCanvas(150, 57, COLORS.canvasWarm);
fillCanvas(header, COLORS.canvasWarm);
for (let y = 0; y < header.height; y += 1) setPixel(header, 0, y, COLORS.coral);
drawMark(header, 13, 5, 46);
for (let x = 72; x < 136; x += 1) setPixel(header, x, 42, COLORS.line);
writeArtwork("src-tauri/nsis/header.bmp", header);

const sidebar = createCanvas(164, 314, COLORS.canvas);
fillCanvas(sidebar, COLORS.canvas);
for (let y = 0; y < sidebar.height; y += 1) setPixel(sidebar, 0, y, COLORS.coral);
drawMark(sidebar, 31, 42, 100);
roundedRect(sidebar, 31, 185, 133, 191, 3, COLORS.coral, 0.92);
roundedRect(sidebar, 31, 201, 110, 207, 3, COLORS.mint, 0.92);
roundedRect(sidebar, 31, 217, 123, 223, 3, COLORS.line, 1);
writeArtwork("src-tauri/nsis/sidebar.bmp", sidebar);
