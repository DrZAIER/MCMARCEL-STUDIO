#!/usr/bin/env node
/**
 * Generates the McMarcel Studio application icons.
 *
 * The icons are drawn from the same mark used in the UI header so the desktop
 * shell, the installer and the browser tab stay visually consistent. No binary
 * asset is committed: run `node scripts/generate-icons.mjs` to recreate them.
 */
import { deflateSync } from 'node:zlib';
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const iconDir = resolve(here, '..', 'src-tauri', 'icons');

const BACKGROUND = [11, 79, 72, 255];
const MARK = [232, 201, 143, 255];

// Mark geometry in the 48x48 space used by the UI logo.
const POLYLINE = [
  [8, 35],
  [18, 12],
  [25, 27],
  [30, 18],
  [40, 35],
];
const DOT = { x: 37, y: 12, r: 4 };
const STROKE = 3.4;

function crc32(buffer) {
  let crc = 0xffffffff;
  for (const byte of buffer) {
    crc ^= byte;
    for (let bit = 0; bit < 8; bit += 1) {
      const mask = -(crc & 1);
      crc = (crc >>> 1) ^ (0xedb88320 & mask);
    }
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length, 0);
  const typeAndData = Buffer.concat([Buffer.from(type, 'ascii'), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(typeAndData), 0);
  return Buffer.concat([length, typeAndData, crc]);
}

function encodePng(size, pixels) {
  const stride = size * 4;
  const raw = Buffer.alloc((stride + 1) * size);
  for (let y = 0; y < size; y += 1) {
    raw[y * (stride + 1)] = 0; // filter: none
    pixels.copy(raw, y * (stride + 1) + 1, y * stride, (y + 1) * stride);
  }
  const header = Buffer.alloc(13);
  header.writeUInt32BE(size, 0);
  header.writeUInt32BE(size, 4);
  header[8] = 8; // bit depth
  header[9] = 6; // colour type: RGBA
  header[10] = 0;
  header[11] = 0;
  header[12] = 0;
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk('IHDR', header),
    chunk('IDAT', deflateSync(raw, { level: 9 })),
    chunk('IEND', Buffer.alloc(0)),
  ]);
}

function draw(size) {
  const pixels = Buffer.alloc(size * size * 4);
  const scale = size / 48;
  const centre = (value) => value * scale;
  const points = POLYLINE.map(([x, y]) => [centre(x), centre(y)]);
  const strokeRadius = (STROKE * scale) / 2;

  for (let y = 0; y < size; y += 1) {
    for (let x = 0; x < size; x += 1) {
      const offset = (y * size + x) * 4;
      pixels[offset] = BACKGROUND[0];
      pixels[offset + 1] = BACKGROUND[1];
      pixels[offset + 2] = BACKGROUND[2];
      pixels[offset + 3] = BACKGROUND[3];
    }
  }

  const paint = (px, py) => {
    const offset = (Math.round(py) * size + Math.round(px)) * 4;
    if (offset < 0 || offset + 3 >= pixels.length) return;
    pixels[offset] = MARK[0];
    pixels[offset + 1] = MARK[1];
    pixels[offset + 2] = MARK[2];
    pixels[offset + 3] = MARK[3];
  };

  // Anti-aliased strokes along each segment.
  for (let index = 0; index < points.length - 1; index += 1) {
    const [x1, y1] = points[index];
    const [x2, y2] = points[index + 1];
    const steps = Math.ceil(Math.hypot(x2 - x1, y2 - y1) * 2);
    for (let step = 0; step <= steps; step += 1) {
      const t = steps === 0 ? 0 : step / steps;
      paint(x1 + (x2 - x1) * t, y1 + (y2 - y1) * t);
    }
  }

  // Stroke thickness by stamping a disc at every sampled point.
  for (let index = 0; index < points.length - 1; index += 1) {
    const [x1, y1] = points[index];
    const [x2, y2] = points[index + 1];
    const steps = Math.ceil(Math.hypot(x2 - x1, y2 - y1) * 2);
    for (let step = 0; step <= steps; step += 1) {
      const t = steps === 0 ? 0 : step / steps;
      const cx = x1 + (x2 - x1) * t;
      const cy = y1 + (y2 - y1) * t;
      for (let dy = -Math.ceil(strokeRadius); dy <= Math.ceil(strokeRadius); dy += 1) {
        for (let dx = -Math.ceil(strokeRadius); dx <= Math.ceil(strokeRadius); dx += 1) {
          if (Math.hypot(dx, dy) <= strokeRadius) paint(cx + dx, cy + dy);
        }
      }
    }
  }

  // Filled dot.
  const dotX = centre(DOT.x);
  const dotY = centre(DOT.y);
  const dotRadius = DOT.r * scale;
  for (let y = 0; y < size; y += 1) {
    for (let x = 0; x < size; x += 1) {
      if (Math.hypot(x + 0.5 - dotX, y + 0.5 - dotY) <= dotRadius) paint(x, y);
    }
  }

  return encodePng(size, pixels);
}

function encodeIco(images) {
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0);
  header.writeUInt16LE(1, 2); // type: icon
  header.writeUInt16LE(images.length, 4);
  const entries = Buffer.alloc(16 * images.length);
  let offset = 6 + entries.length;
  images.forEach((image, index) => {
    const base = index * 16;
    entries[base] = image.size >= 256 ? 0 : image.size;
    entries[base + 1] = image.size >= 256 ? 0 : image.size;
    entries[base + 2] = 0; // palette
    entries[base + 3] = 0; // reserved
    entries.writeUInt16LE(1, base + 4); // colour planes
    entries.writeUInt16LE(32, base + 6); // bits per pixel
    entries.writeUInt32LE(image.data.length, base + 8);
    entries.writeUInt32LE(offset, base + 12);
    offset += image.data.length;
  });
  return Buffer.concat([header, entries, ...images.map((image) => image.data)]);
}

mkdirSync(iconDir, { recursive: true });
const sizes = [32, 128, 256, 512];
const rendered = sizes.map((size) => ({ size, data: draw(size) }));
rendered.forEach(({ size, data }) => {
  const name = size === 256 ? '128x128@2x.png' : size === 512 ? 'icon.png' : `${size}x${size}.png`;
  writeFileSync(resolve(iconDir, name), data);
  console.log(`wrote ${name} (${size}x${size})`);
});
writeFileSync(
  resolve(iconDir, 'icon.ico'),
  encodeIco([
    { size: 32, data: rendered[0].data },
    { size: 256, data: rendered[2].data },
  ]),
);
console.log('wrote icon.ico');
