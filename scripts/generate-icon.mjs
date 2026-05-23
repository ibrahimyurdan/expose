import { writeFileSync, mkdirSync } from "node:fs";
import { deflateSync } from "node:zlib";
import path from "node:path";

// generates a placeholder hextech app icon: a gold diamond ring on deep navy.
// it has no external dependencies so the project can be built from a clean
// checkout. to ship custom art, replace src-tauri/icons/source.png and rerun
// `npm run tauri icon src-tauri/icons/source.png`.

const SIZE = 1024;

const NAVY = [10, 20, 40];
const GOLD = [200, 170, 110];
const GOLD_DEEP = [120, 90, 40];

function lerp(a, b, t) {
  return Math.round(a + (b - a) * t);
}

const pixels = Buffer.alloc(SIZE * SIZE * 4);
const cx = SIZE / 2;
const cy = SIZE / 2;
const radius = SIZE * 0.42;

for (let y = 0; y < SIZE; y++) {
  for (let x = 0; x < SIZE; x++) {
    // manhattan distance from the center traces a diamond shape
    const d = (Math.abs(x - cx) + Math.abs(y - cy)) / radius;
    let color;
    if (d <= 0.46) {
      color = GOLD;
    } else if (d <= 0.82) {
      color = NAVY;
    } else if (d <= 1.0) {
      const t = (d - 0.82) / 0.18;
      color = [
        lerp(GOLD[0], GOLD_DEEP[0], t),
        lerp(GOLD[1], GOLD_DEEP[1], t),
        lerp(GOLD[2], GOLD_DEEP[2], t),
      ];
    } else {
      color = NAVY;
    }
    const i = (y * SIZE + x) * 4;
    pixels[i] = color[0];
    pixels[i + 1] = color[1];
    pixels[i + 2] = color[2];
    pixels[i + 3] = 255;
  }
}

const CRC_TABLE = (() => {
  const table = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) {
      c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    }
    table[n] = c >>> 0;
  }
  return table;
})();

function crc32(buf) {
  let c = 0xffffffff;
  for (let i = 0; i < buf.length; i++) {
    c = CRC_TABLE[(c ^ buf[i]) & 0xff] ^ (c >>> 8);
  }
  return (c ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const typeBuf = Buffer.from(type, "ascii");
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length, 0);
  const body = Buffer.concat([typeBuf, data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(body), 0);
  return Buffer.concat([len, body, crc]);
}

function encodePng(width, height, rgba) {
  const signature = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);

  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8; // bit depth
  ihdr[9] = 6; // color type: truecolor with alpha
  ihdr[10] = 0; // compression
  ihdr[11] = 0; // filter
  ihdr[12] = 0; // interlace

  // every scanline is prefixed with a filter-type byte (0 means no filter)
  const raw = Buffer.alloc(height * (width * 4 + 1));
  for (let y = 0; y < height; y++) {
    const src = y * width * 4;
    const dst = y * (width * 4 + 1);
    raw[dst] = 0;
    rgba.copy(raw, dst + 1, src, src + width * 4);
  }
  const idat = deflateSync(raw, { level: 9 });

  return Buffer.concat([
    signature,
    chunk("IHDR", ihdr),
    chunk("IDAT", idat),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

const outDir = path.resolve(import.meta.dirname, "../src-tauri/icons");
mkdirSync(outDir, { recursive: true });
const outFile = path.join(outDir, "source.png");
writeFileSync(outFile, encodePng(SIZE, SIZE, pixels));
console.log(`wrote ${outFile} (${SIZE}x${SIZE})`);
