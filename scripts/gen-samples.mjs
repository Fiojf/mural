#!/usr/bin/env node
// Generates 5 simple gradient PNGs as bundled wallpaper samples.
// These are CC0 because they are trivial procedurally-generated content.

import { writeFileSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { deflateSync } from "node:zlib";

const here = dirname(fileURLToPath(import.meta.url));
const out = resolve(here, "../src-tauri/resources/samples");
mkdirSync(out, { recursive: true });

function crc32Buf(buf) {
  const table = [];
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    table[n] = c;
  }
  let crc = 0xffffffff;
  for (let i = 0; i < buf.length; i++) crc = table[(crc ^ buf[i]) & 0xff] ^ (crc >>> 8);
  return (crc ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const t = Buffer.from(type, "ascii");
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32Buf(Buffer.concat([t, data])));
  return Buffer.concat([len, t, data, crc]);
}

function makePng(w, h, painter) {
  const raw = Buffer.alloc(h * (1 + w * 3));
  for (let y = 0; y < h; y++) {
    raw[y * (1 + w * 3)] = 0;
    for (let x = 0; x < w; x++) {
      const [r, g, b] = painter(x, y, w, h);
      const o = y * (1 + w * 3) + 1 + x * 3;
      raw[o] = r;
      raw[o + 1] = g;
      raw[o + 2] = b;
    }
  }
  const idat = deflateSync(raw);
  const sig = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(w, 0);
  ihdr.writeUInt32BE(h, 4);
  ihdr[8] = 8;
  ihdr[9] = 2;
  return Buffer.concat([sig, chunk("IHDR", ihdr), chunk("IDAT", idat), chunk("IEND", Buffer.alloc(0))]);
}

const samples = [
  ["sample-aurora.png", (x, y, w, h) => {
    const t = y / h;
    const r = Math.round(20 + 80 * t);
    const g = Math.round(40 + 180 * Math.sin((x / w) * Math.PI));
    const b = Math.round(100 + 120 * (1 - t));
    return [r, g, b];
  }],
  ["sample-sunset.png", (x, y, w, h) => {
    const t = y / h;
    return [Math.round(255 * (1 - t * 0.5)), Math.round(120 * (1 - t)), Math.round(60 * (1 - t))];
  }],
  ["sample-mint.png", (x, y, w, h) => {
    const dx = x / w - 0.5, dy = y / h - 0.5;
    const d = Math.sqrt(dx * dx + dy * dy);
    return [Math.round(140 + 60 * d), Math.round(220 - 80 * d), Math.round(180 - 40 * d)];
  }],
  ["sample-night.png", (x, y, w, h) => {
    const t = y / h;
    return [Math.round(10 + 20 * t), Math.round(15 + 30 * t), Math.round(35 + 80 * t)];
  }],
  ["sample-rose.png", (x, y, w, h) => {
    return [Math.round(180 + 60 * (x / w)), 80, Math.round(110 + 40 * (1 - y / h))];
  }],
];

const W = 1024, H = 640;
for (const [name, painter] of samples) {
  writeFileSync(resolve(out, name), makePng(W, H, painter));
}
console.log(`wrote ${samples.length} samples to ${out}`);
