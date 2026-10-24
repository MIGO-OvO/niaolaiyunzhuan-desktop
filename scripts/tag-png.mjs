import { readFile, writeFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import { crc32 } from 'node:zlib';

// Only PNG metadata changes; IDAT pixel data is copied byte for byte.
const [input, output, promptFile] = process.argv.slice(2);
if (!input || !output) throw new Error('Usage: node scripts/tag-png.mjs input.png output.png [prompt.txt]');
const source = await readFile(input);
if (source.subarray(0, 8).toString('hex') !== '89504e470d0a1a0a') throw new Error('Expected a PNG');
function chunk(type, data) {
  const name = Buffer.from(type);
  const result = Buffer.alloc(data.length + 12);
  result.writeUInt32BE(data.length, 0);
  name.copy(result, 4); data.copy(result, 8);
  result.writeUInt32BE(crc32(Buffer.concat([name, data])), data.length + 8);
  return result;
}
const density = Buffer.alloc(9);
density.writeUInt32BE(Math.round(300 / .0254), 0);
density.writeUInt32BE(Math.round(300 / .0254), 4);
density[8] = 1;
const chunks = [source.subarray(0, 8)];
for (let offset = 8; offset < source.length;) {
  const length = source.readUInt32BE(offset);
  const type = source.toString('ascii', offset + 4, offset + 8);
  if (type !== 'pHYs') chunks.push(source.subarray(offset, offset + length + 12));
  if (type === 'IHDR') {
    chunks.push(chunk('pHYs', density));
    if (promptFile) chunks.push(chunk('iTXt', Buffer.from(`GenerationPrompt\0\0\0\0\0${await readFile(promptFile, 'utf8')}`, 'utf8')));
  }
  offset += length + 12;
}
await mkdir(path.dirname(output), { recursive: true });
await writeFile(output, Buffer.concat(chunks));
console.log(JSON.stringify({ path: output, width: source.readUInt32BE(16), height: source.readUInt32BE(20), dpi: 300 }));
