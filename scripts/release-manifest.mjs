import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

const read = path => readFileSync(path, 'utf8');
const pkg = JSON.parse(read('package.json'));
const config = JSON.parse(read('src-tauri/tauri.conf.json'));
const cargo = read('src-tauri/Cargo.toml').match(/^version = "([^"]+)"/m)?.[1];
const lock = JSON.parse(read('package-lock.json'));
const version = pkg.version;
assert.match(version, /^\d+\.\d+\.\d+$/, 'Release version must use x.y.z');
for (const value of [config.version, cargo, lock.version, lock.packages[''].version]) {
  assert.equal(value, version, 'All package versions must agree');
}
const repository = 'MIGO-OvO/niaolaiyunzhuan-desktop';
assert.deepEqual(config.plugins.updater.endpoints, [
  `https://github.com/${repository}/releases/latest/download/latest.json`,
]);
assert.equal(config.plugins.updater.requireSignedVersion, true);
assert.equal(config.bundle.createUpdaterArtifacts, true);
if (process.argv.includes('--tag')) {
  assert.equal(process.env.GITHUB_REF_TYPE, 'tag', 'Run release from a version tag');
  assert.equal(process.env.GITHUB_REF_NAME, `v${version}`, 'Tag must match the package version');
}
if (process.argv.includes('--check')) {
  console.log(`Release configuration ready: v${version}`);
  process.exit(0);
}

const destination = resolve('.local/release');
const installer = `niaolaiyunzhuan_${version}_x64-setup.exe`;
const source = `target/release/bundle/nsis/${config.productName}_${version}_x64-setup.exe`;
const signature = read(`${source}.sig`).trim();
const notes = read(`docs/releases/v${version}.md`).trim();
mkdirSync(destination, { recursive: true });
copyFileSync(source, resolve(destination, installer));
writeFileSync(resolve(destination, `${installer}.sig`), `${signature}\n`);
writeFileSync(resolve(destination, 'latest.json'), `${JSON.stringify({
  version,
  notes,
  pub_date: new Date().toISOString(),
  platforms: {
    'windows-x86_64': {
      signature,
      url: `https://github.com/${repository}/releases/download/v${version}/${installer}`,
    },
  },
}, null, 2)}\n`);
const files = [installer, `${installer}.sig`, 'latest.json'];
writeFileSync(resolve(destination, 'SHA256SUMS'), files.map(file =>
  `${createHash('sha256').update(readFileSync(resolve(destination, file))).digest('hex')}  ${file}\n`,
).join(''));
console.log(`Prepared v${version} installer, signature, manifest and SHA256SUMS in ${destination}`);
