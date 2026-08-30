// Generates every Cliply app icon asset from one vector definition.
//
// Direction: two offset paper shapes express clipboard history without using
// a literal clipboard outline. Coral is the active clip, mint is its history,
// and the folded corner gives the silhouette a recognizable action. The mark
// stays flat and typographic: no gradient, glow, glass orb, sparkle, or letter.
//
// Usage: node scripts/generate-app-icon.mjs

import { deflateSync } from "node:zlib";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..");

const PAPER_FRONT = [0x3b, 0xa9, 0x7c];
// Small sizes (titlebar 20px, tray/taskbar 32-48px) sit on light titlebars and
// dark taskbars where the mid celadon washes out, so they run one step brighter.
const PAPER_FRONT_SMALL = [0x58, 0xc7, 0x9e];
const PAPER_BACK = [0xec, 0x7a, 0x64];
const PAPER_BACK_SMALL = [0xff, 0x9d, 0x8b];
const PAPER_FOLD = [0xff, 0xe8, 0xce];
const INK = [0x16, 0x34, 0x2a];
const INK_SMALL = [0x0f, 0x2a, 0x20];
const CAST_SHADOW = [0x21, 0x28, 0x2d];
const SMALL_MAX_SIZE = 48;

// Windows taskbar icons need a denser optical fill than in-app artwork. The
// sheets use roughly 90% of the 32px canvas so the narrow paper silhouette has
// the same perceived size as circular and square neighboring app icons.
const MARK_LARGE = {
  back: { x0: 0.276, y0: 0.068, x1: 0.931, y1: 0.738, r: 0.113 },
  front: { x0: 0.069, y0: 0.208, x1: 0.805, y1: 0.932, r: 0.113 },
};

const MARK_SMALL = {
  back: { x0: 0.299, y0: 0.073, x1: 0.931, y1: 0.743, r: 0.13 },
  front: { x0: 0.069, y0: 0.214, x1: 0.805, y1: 0.927, r: 0.13 },
};

const RULES = [
  { x0: 0.207, x1: 0.598, cy: 0.554 },
  { x0: 0.207, x1: 0.494, cy: 0.694 },
];
const RULE_HEIGHT = 0.037;
const DETAIL_SCALE_X = 1.15;
const DETAIL_SCALE_Y = 1.08;
const FOLD_MIN_SIZE = 20;
const RULES_MIN_SIZE = 32;

function layersFor(size) {
  const mark = size <= 24 ? MARK_SMALL : MARK_LARGE;
  const paperFront = size <= SMALL_MAX_SIZE ? PAPER_FRONT_SMALL : PAPER_FRONT;
  const paperBack = size <= SMALL_MAX_SIZE ? PAPER_BACK_SMALL : PAPER_BACK;
  const ink = size <= SMALL_MAX_SIZE ? INK_SMALL : INK;
  const layers = [
    {
      ...mark.back,
      y0: mark.back.y0 + 0.014,
      y1: mark.back.y1 + 0.014,
      fill: CAST_SHADOW,
      shadow: 0.028,
      alpha: 0.14,
    },
    { ...mark.back, fill: paperBack },
    {
      ...mark.front,
      x0: mark.front.x0 - 0.004,
      x1: mark.front.x1 - 0.004,
      y0: mark.front.y0 + 0.014,
      y1: mark.front.y1 + 0.014,
      fill: CAST_SHADOW,
      shadow: 0.03,
      alpha: 0.18,
    },
    { ...mark.front, fill: paperFront },
  ];

  if (size >= FOLD_MIN_SIZE) {
    layers.push({
      points: [
        {
          x: mark.front.x1 - 0.215 * DETAIL_SCALE_X,
          y: mark.front.y0 + 0.025 * DETAIL_SCALE_Y,
        },
        {
          x: mark.front.x1 - 0.045 * DETAIL_SCALE_X,
          y: mark.front.y0 + 0.025 * DETAIL_SCALE_Y,
        },
        {
          x: mark.front.x1 - 0.045 * DETAIL_SCALE_X,
          y: mark.front.y0 + 0.18 * DETAIL_SCALE_Y,
        },
      ],
      fill: PAPER_FOLD,
    });
  }

  if (size >= RULES_MIN_SIZE) {
    for (const rule of RULES) {
      layers.push({
        x0: rule.x0,
        y0: rule.cy - RULE_HEIGHT / 2,
        x1: rule.x1,
        y1: rule.cy + RULE_HEIGHT / 2,
        r: RULE_HEIGHT / 2,
        fill: ink,
        alpha: 0.82,
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

function pointInTriangle(px, py, points, size) {
  const [a, b, c] = points.map((point) => ({ x: point.x * size, y: point.y * size }));
  const edge = (p1, p2) => (px - p2.x) * (p1.y - p2.y) - (p1.x - p2.x) * (py - p2.y);
  const d1 = edge(a, b);
  const d2 = edge(b, c);
  const d3 = edge(c, a);
  const hasNegative = d1 < 0 || d2 < 0 || d3 < 0;
  const hasPositive = d1 > 0 || d2 > 0 || d3 > 0;
  return !(hasNegative && hasPositive);
}

function triangleCoverage(shape, x, y, size) {
  const samples = size <= 32 ? 4 : 2;
  let covered = 0;
  for (let sampleY = 0; sampleY < samples; sampleY += 1) {
    for (let sampleX = 0; sampleX < samples; sampleX += 1) {
      const px = x + (sampleX + 0.5) / samples;
      const py = y + (sampleY + 0.5) / samples;
      if (pointInTriangle(px, py, shape.points, size)) {
        covered += 1;
      }
    }
  }
  return covered / (samples * samples);
}

/// Renders the icon at `size` into straight (non-premultiplied) RGBA bytes.
function renderIcon(size) {
  // Premultiplied accumulation keeps source-over compositing branch-free.
  const premul = new Float64Array(size * size * 4);

  for (const shape of layersFor(size)) {
    const cx = shape.points ? 0 : ((shape.x0 + shape.x1) / 2) * size;
    const cy = shape.points ? 0 : ((shape.y0 + shape.y1) / 2) * size;
    const halfWidth = shape.points ? 0 : ((shape.x1 - shape.x0) / 2) * size;
    const halfHeight = shape.points ? 0 : ((shape.y1 - shape.y0) / 2) * size;
    const radius = shape.points ? 0 : shape.r * size;
    const spread = (shape.shadow ?? 0) * size;
    const maxAlpha = shape.alpha ?? 1;

    for (let y = 0; y < size; y += 1) {
      const [sr, sg, sb] = sampleFill(shape.fill, y, size);

      for (let x = 0; x < size; x += 1) {
        let coverage;
        if (shape.points) {
          coverage = triangleCoverage(shape, x, y, size) * maxAlpha;
        } else if (spread > 0) {
          const distance = roundedRectDistance(
            x + 0.5,
            y + 0.5,
            cx,
            cy,
            halfWidth,
            halfHeight,
            radius,
          );
          // Quadratic falloff over `spread` gives a genuinely soft shadow
          // straight from the distance field — no blur pass needed.
          const t = Math.min(Math.max(1 - distance / spread, 0), 1);
          coverage = t * t * maxAlpha;
        } else {
          const distance = roundedRectDistance(
            x + 0.5,
            y + 0.5,
            cx,
            cy,
            halfWidth,
            halfHeight,
            radius,
          );
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
  ["src/assets/cliply-logo-20.png", 20],
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
