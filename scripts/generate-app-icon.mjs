// Generates every Cliply app icon asset from one vector definition.
//
// Direction: the mark is the *stack*, not a clipboard. Cliply's job is clipboard
// history — a deck of things you can page back through — so the icon is three
// offset sheets: the front one is the live clip (brand blue, white rules) and
// the two behind recede into lighter blues. No container square, so the
// silhouette itself is identifiable rather than "pictogram in a box".
// Gradients stay within a single hue and a narrow range, which reads as
// material; the two-hue saturated ramp is what makes an icon look generated.
//
// Usage: node scripts/generate-app-icon.mjs

import { deflateSync } from "node:zlib";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..");

// Front sheet carries the product accent; sheets behind recede (aerial
// perspective) so depth reads without outlines.
const SHEET_FRONT = { from: [0x30, 0x82, 0xd6], to: [0x1a, 0x6a, 0xbe] };
const SHEET_MID = { from: [0x7f, 0xb1, 0xe8], to: [0x5f, 0x9b, 0xdd] };
const SHEET_BACK = { from: [0xb6, 0xd5, 0xf3], to: [0x9c, 0xc5, 0xee] };
const RULE = [0xff, 0xff, 0xff];
const CAST_SHADOW = [0x0d, 0x3c, 0x6e];

// Geometry in unit-square coordinates (0..1), rasterised at any size.
// Each sheet steps up-and-right by a constant offset. The stack is sized to
// fill ~85% of the canvas — taskbar icons look undersized below that.
const STACK_LARGE = {
  sheets: 3,
  sheet: { w: 0.56, h: 0.66, r: 0.06 },
  step: { x: 0.11, y: -0.09 },
  origin: { x: 0.11, y: 0.25 },
};

// Below ~32px the standard step collapses into a plain rounded square, so the
// small variant drops a sheet and widens the offset — the usual hand-tuning
// every real icon set does for 16/24px.
const STACK_SMALL = {
  sheets: 2,
  sheet: { w: 0.62, h: 0.7, r: 0.085 },
  step: { x: 0.15, y: -0.15 },
  origin: { x: 0.115, y: 0.22 },
};

const SHEET_FILLS = [SHEET_FRONT, SHEET_MID, SHEET_BACK];

function stackFor(size) {
  return size <= 24 ? STACK_SMALL : STACK_LARGE;
}

function sheetRect(stack, level) {
  const x0 = stack.origin.x + stack.step.x * level;
  const y0 = stack.origin.y + stack.step.y * level;
  return {
    x0,
    y0,
    x1: x0 + stack.sheet.w,
    y1: y0 + stack.sheet.h,
    r: stack.sheet.r,
  };
}

/// Soft cast shadow for a sheet, nudged away from the light so it lands on the
/// sheet behind rather than ringing the shape.
function sheetShadow(stack, level, spread, alpha) {
  const rect = sheetRect(stack, level);
  return {
    x0: rect.x0 - 0.005,
    y0: rect.y0 + 0.009,
    x1: rect.x1 - 0.005,
    y1: rect.y1 + 0.009,
    r: rect.r,
    fill: CAST_SHADOW,
    shadow: spread,
    alpha,
  };
}

const RULES = [
  { x0: 0.185, x1: 0.595, cy: 0.46 },
  { x0: 0.185, x1: 0.595, cy: 0.575 },
  { x0: 0.185, x1: 0.47, cy: 0.69 },
];
const RULE_HEIGHT = 0.038;
const RULES_MIN_SIZE = 48;

function layersFor(size) {
  const stack = stackFor(size);
  const layers = [];

  // Back to front, with each sheet casting onto the one behind it.
  for (let level = stack.sheets - 1; level >= 0; level -= 1) {
    if (level < stack.sheets - 1) {
      layers.push(sheetShadow(stack, level, 0.022 + level * 0.002, 0.28));
    }
    layers.push({ ...sheetRect(stack, level), fill: SHEET_FILLS[level] });
  }

  if (size >= RULES_MIN_SIZE) {
    for (const rule of RULES) {
      layers.push({
        x0: rule.x0,
        y0: rule.cy - RULE_HEIGHT / 2,
        x1: rule.x1,
        y1: rule.cy + RULE_HEIGHT / 2,
        r: RULE_HEIGHT / 2,
        fill: RULE,
        alpha: 0.92,
      });
    }
  }

  return layers;
}

/// Signed distance to a rounded rectangle, in pixels. Negative inside.
function roundedRectDistance(px, py, cx, cy, halfWidth, halfHeight, radius) {
  const r = Math.min(radius, halfWidth, halfHeight);
  const qx = Math.abs(px - cx) - (halfWidth - r);
  const qy = Math.abs(py - cy) - (halfHeight - r);
  const outside = Math.hypot(Math.max(qx, 0), Math.max(qy, 0));
  const inside = Math.min(Math.max(qx, qy), 0);
  return outside + inside - r;
}

/// Vertical ramp sampled over the full canvas, so any shape sharing a gradient
/// lines up seamlessly with the board behind it.
function sampleFill(fill, y, size) {
  if (Array.isArray(fill)) {
    return fill;
  }
  const t = size <= 1 ? 0 : y / (size - 1);
  return [
    fill.from[0] + (fill.to[0] - fill.from[0]) * t,
    fill.from[1] + (fill.to[1] - fill.from[1]) * t,
    fill.from[2] + (fill.to[2] - fill.from[2]) * t,
  ];
}

/// Renders the icon at `size` into straight (non-premultiplied) RGBA bytes.
function renderIcon(size) {
  // Premultiplied accumulation keeps source-over compositing branch-free.
  const premul = new Float64Array(size * size * 4);

  for (const shape of layersFor(size)) {
    const cx = ((shape.x0 + shape.x1) / 2) * size;
    const cy = ((shape.y0 + shape.y1) / 2) * size;
    const halfWidth = ((shape.x1 - shape.x0) / 2) * size;
    const halfHeight = ((shape.y1 - shape.y0) / 2) * size;
    const radius = shape.r * size;
    const spread = (shape.shadow ?? 0) * size;
    const maxAlpha = shape.alpha ?? 1;

    for (let y = 0; y < size; y += 1) {
      const [sr, sg, sb] = sampleFill(shape.fill, y, size);

      for (let x = 0; x < size; x += 1) {
        const distance = roundedRectDistance(
          x + 0.5,
          y + 0.5,
          cx,
          cy,
          halfWidth,
          halfHeight,
          radius,
        );

        let coverage;
        if (spread > 0) {
          // Quadratic falloff over `spread` gives a genuinely soft shadow
          // straight from the distance field — no blur pass needed.
          const t = Math.min(Math.max(1 - distance / spread, 0), 1);
          coverage = t * t * maxAlpha;
        } else {
          // Analytic 1px antialias band around the edge.
          coverage = Math.min(Math.max(0.5 - distance, 0), 1) * maxAlpha;
        }
        if (coverage <= 0) {
          continue;
        }

        const offset = (y * size + x) * 4;
        const inverse = 1 - coverage;
        premul[offset] = sr * coverage + premul[offset] * inverse;
        premul[offset + 1] = sg * coverage + premul[offset + 1] * inverse;
        premul[offset + 2] = sb * coverage + premul[offset + 2] * inverse;
        premul[offset + 3] = coverage + premul[offset + 3] * inverse;
      }
    }
  }

  const rgba = Buffer.alloc(size * size * 4);
  for (let i = 0; i < size * size; i += 1) {
    const offset = i * 4;
    const alpha = premul[offset + 3];
    if (alpha <= 0) {
      continue;
    }
    rgba[offset] = Math.round(Math.min(premul[offset] / alpha, 255));
    rgba[offset + 1] = Math.round(Math.min(premul[offset + 1] / alpha, 255));
    rgba[offset + 2] = Math.round(Math.min(premul[offset + 2] / alpha, 255));
    rgba[offset + 3] = Math.round(alpha * 255);
  }
  return rgba;
}

const CRC_TABLE = (() => {
  const table = new Uint32Array(256);
  for (let n = 0; n < 256; n += 1) {
    let c = n;
    for (let k = 0; k < 8; k += 1) {
      c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    }
    table[n] = c >>> 0;
  }
  return table;
})();

function crc32(buffer) {
  let crc = 0xffffffff;
  for (const byte of buffer) {
    crc = CRC_TABLE[(crc ^ byte) & 0xff] ^ (crc >>> 8);
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function pngChunk(type, data) {
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length, 0);
  const typeAndData = Buffer.concat([Buffer.from(type, "ascii"), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(typeAndData), 0);
  return Buffer.concat([length, typeAndData, crc]);
}

function encodePng(size, rgba) {
  const header = Buffer.alloc(13);
  header.writeUInt32BE(size, 0);
  header.writeUInt32BE(size, 4);
  header[8] = 8; // bit depth
  header[9] = 6; // colour type: RGBA
  header[10] = 0; // deflate
  header[11] = 0; // adaptive filtering
  header[12] = 0; // no interlace

  // Filter type 0 (None) per scanline keeps the encoder trivial; these icons
  // are tiny and compress fine regardless.
  const stride = size * 4;
  const raw = Buffer.alloc((stride + 1) * size);
  for (let y = 0; y < size; y += 1) {
    raw[y * (stride + 1)] = 0;
    rgba.copy(raw, y * (stride + 1) + 1, y * stride, (y + 1) * stride);
  }

  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    pngChunk("IHDR", header),
    pngChunk("IDAT", deflateSync(raw, { level: 9 })),
    pngChunk("IEND", Buffer.alloc(0)),
  ]);
}

/// Encodes one icon entry as an uncompressed DIB (BITMAPINFOHEADER + BGRA
/// bottom-up pixels + AND mask).
function encodeIcoDib(size, rgba) {
  const header = Buffer.alloc(40);
  header.writeUInt32LE(40, 0); // biSize
  header.writeInt32LE(size, 4); // biWidth
  header.writeInt32LE(size * 2, 8); // biHeight: XOR image + AND mask
  header.writeUInt16LE(1, 12); // biPlanes
  header.writeUInt16LE(32, 14); // biBitCount
  header.writeUInt32LE(0, 16); // biCompression: BI_RGB

  const xor = Buffer.alloc(size * size * 4);
  for (let y = 0; y < size; y += 1) {
    const sourceRow = size - 1 - y; // DIB rows run bottom-up
    for (let x = 0; x < size; x += 1) {
      const from = (sourceRow * size + x) * 4;
      const to = (y * size + x) * 4;
      xor[to] = rgba[from + 2]; // B
      xor[to + 1] = rgba[from + 1]; // G
      xor[to + 2] = rgba[from]; // R
      xor[to + 3] = rgba[from + 3]; // A
    }
  }

  // 1bpp AND mask, rows padded to 4 bytes. Left at zero so the 32bpp alpha
  // channel is what actually decides transparency.
  const maskStride = Math.ceil(size / 32) * 4;
  const mask = Buffer.alloc(maskStride * size);

  return Buffer.concat([header, xor, mask]);
}

/// Builds a multi-resolution .ico. Every entry is a classic DIB: GDI+
/// (System.Drawing) and some installer toolchains silently skip
/// PNG-compressed entries, which would make the 256px variant unusable.
function encodeIco(entries) {
  const directory = Buffer.alloc(6 + entries.length * 16);
  directory.writeUInt16LE(0, 0); // reserved
  directory.writeUInt16LE(1, 2); // type: icon
  directory.writeUInt16LE(entries.length, 4);

  let offset = directory.length;
  entries.forEach((entry, index) => {
    const at = 6 + index * 16;
    directory[at] = entry.size >= 256 ? 0 : entry.size;
    directory[at + 1] = entry.size >= 256 ? 0 : entry.size;
    directory[at + 2] = 0; // palette size
    directory[at + 3] = 0; // reserved
    directory.writeUInt16LE(1, at + 4); // colour planes
    directory.writeUInt16LE(32, at + 6); // bits per pixel
    directory.writeUInt32LE(entry.data.length, at + 8);
    directory.writeUInt32LE(offset, at + 12);
    offset += entry.data.length;
  });

  return Buffer.concat([directory, ...entries.map((entry) => entry.data)]);
}

const renderCache = new Map();
function renderFor(size) {
  if (!renderCache.has(size)) {
    renderCache.set(size, renderIcon(size));
  }
  return renderCache.get(size);
}

const pngCache = new Map();
function pngFor(size) {
  if (!pngCache.has(size)) {
    pngCache.set(size, encodePng(size, renderFor(size)));
  }
  return pngCache.get(size);
}

function icoEntryFor(size) {
  return { size, data: encodeIcoDib(size, renderFor(size)) };
}

function writeIcon(relativePath, size) {
  const target = join(repoRoot, relativePath);
  mkdirSync(dirname(target), { recursive: true });
  const png = pngFor(size);
  writeFileSync(target, png);
  console.log(`${relativePath.padEnd(56)} ${size}x${size}  ${png.length} B`);
}

const PNG_TARGETS = [
  ["src-tauri/icons/32x32.png", 32],
  ["src-tauri/icons/128x128.png", 128],
  ["src-tauri/icons/128x128@2x.png", 256],
  ["src-tauri/icons/tray-32.png", 32],
  ["src-tauri/icons/ico-sizes/16.png", 16],
  ["src-tauri/icons/ico-sizes/24.png", 24],
  ["src-tauri/icons/ico-sizes/32.png", 32],
  ["src-tauri/icons/ico-sizes/48.png", 48],
  ["src-tauri/icons/ico-sizes/64.png", 64],
  ["src/assets/cliply-logo.png", 512],
  ["src/assets/cliply-logo-256.png", 256],
  ["apps/cliply-installer/src-tauri/icons/32x32.png", 32],
  ["apps/cliply-installer/src-tauri/icons/128x128.png", 128],
  ["apps/cliply-installer/src-tauri/icons/128x128@2x.png", 256],
  ["apps/cliply-installer/public/cliply-logo.png", 512],
];

const ICO_SIZES = [16, 24, 32, 48, 64, 128, 256];
const ICO_TARGETS = [
  "src-tauri/icons/icon.ico",
  "apps/cliply-installer/src-tauri/icons/icon.ico",
];

// `--preview` writes a magnified contact sheet instead of touching any asset,
// so the small-size variants can actually be inspected.
if (process.argv.includes("--preview")) {
  const sizes = [16, 24, 32, 48];
  const scale = 8;
  const gap = 8;
  const cell = Math.max(...sizes) * scale;
  const width = sizes.length * cell + (sizes.length + 1) * gap;
  const height = cell + gap * 2;
  const canvas = Buffer.alloc(width * height * 4);

  sizes.forEach((size, index) => {
    const src = renderFor(size);
    const originX = gap + index * (cell + gap) + (cell - size * scale) / 2;
    const originY = gap + (cell - size * scale) / 2;
    for (let y = 0; y < size * scale; y += 1) {
      for (let x = 0; x < size * scale; x += 1) {
        const from = (Math.floor(y / scale) * size + Math.floor(x / scale)) * 4;
        const to = ((originY + y) * width + originX + x) * 4;
        src.copy(canvas, to, from, from + 4);
      }
    }
  });

  const header = Buffer.alloc(13);
  header.writeUInt32BE(width, 0);
  header.writeUInt32BE(height, 4);
  header[8] = 8;
  header[9] = 6;
  const stride = width * 4;
  const raw = Buffer.alloc((stride + 1) * height);
  for (let y = 0; y < height; y += 1) {
    canvas.copy(raw, y * (stride + 1) + 1, y * stride, (y + 1) * stride);
  }
  writeFileSync(
    join(repoRoot, "icon-preview.png"),
    Buffer.concat([
      Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
      pngChunk("IHDR", header),
      pngChunk("IDAT", deflateSync(raw, { level: 9 })),
      pngChunk("IEND", Buffer.alloc(0)),
    ]),
  );
  console.log(`icon-preview.png  ${sizes.join("/")} at ${scale}x`);
  process.exit(0);
}

for (const [relativePath, size] of PNG_TARGETS) {
  writeIcon(relativePath, size);
}

const ico = encodeIco(ICO_SIZES.map(icoEntryFor));
for (const relativePath of ICO_TARGETS) {
  const target = join(repoRoot, relativePath);
  mkdirSync(dirname(target), { recursive: true });
  writeFileSync(target, ico);
  console.log(
    `${relativePath.padEnd(56)} ${ICO_SIZES.join("/")}  ${ico.length} B`,
  );
}
