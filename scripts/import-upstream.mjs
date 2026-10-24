import { readFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { resolve } from 'node:path';
import vm from 'node:vm';

// Print an apply_patch patch; keep the source snapshot unchanged.
const source = process.argv[2];
if (!source) throw new Error('Usage: node scripts/import-upstream.mjs <upstream checkout>');
const context = { window: {} };
vm.runInNewContext(readFileSync(resolve(source, 'data.js'), 'utf8'), context, { timeout: 1000 });
const data = context.window.NIAONIAO_DATA;
const revision = spawnSync('git', ['-C', source, 'rev-parse', 'HEAD'], { encoding: 'utf8' });
if (revision.status !== 0) throw new Error(revision.stderr);
const expected = '8a88ea6c8532f1f67d51660bbb1dd46850af9f18';
if (revision.stdout.trim() !== expected) throw new Error('Review the new upstream revision before importing it.');
const contents = JSON.stringify(data, null, 2) + '\n';
process.stdout.write('*** Begin Patch\n*** Add File: data/catalog.json\n' + contents.trimEnd().split('\n').map(line => '+' + line).join('\n') + '\n*** End Patch\n');
