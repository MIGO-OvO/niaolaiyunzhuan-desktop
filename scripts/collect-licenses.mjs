import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';

const sections = ['Third-party notices for 袅来运转桌面版\n\nOriginal project: https://github.com/MiaTheamateur/niaolaiyunzhuan-web\nOriginal author: 薄荷小鹿（燕云十六声）\nDesktop project license: PolyForm-Noncommercial-1.0.0\nLicense text: https://polyformproject.org/licenses/noncommercial/1.0.0\nUpstream project-level license: not declared.\n'];
const missing = [];
function collect(name, version, license, directory, licenseFile) {
  const files = new Set(readdirSync(directory).filter(file =>
    /^(licen[sc]e|copying|notice|copyright)([._-]|$)/i.test(file),
  ).map(file => resolve(directory, file)).filter(file => statSync(file).isFile()));
  if (licenseFile && existsSync(resolve(directory, licenseFile))) files.add(resolve(directory, licenseFile));
  sections.push(`\n${'='.repeat(72)}\n${name} ${version}\nLicense: ${license ?? 'See package notice'}\n`);
  for (const file of files) sections.push(readFileSync(file, 'utf8'));
  if (!files.size) missing.push(`${name}@${version}`);
}
const metadata = JSON.parse(execFileSync('cargo', [
  'metadata', '--locked', '--all-features', '--format-version', '1', '--filter-platform', 'x86_64-pc-windows-msvc',
], { encoding: 'utf8', maxBuffer: 32 * 1024 * 1024 }));
const resolved = new Set(metadata.resolve.nodes.map(node => node.id));
for (const pkg of metadata.packages.filter(pkg => resolved.has(pkg.id) && !metadata.workspace_members.includes(pkg.id)).sort((a, b) => a.name.localeCompare(b.name) || a.version.localeCompare(b.version))) {
  collect(pkg.name, pkg.version, pkg.license, dirname(pkg.manifest_path), pkg.license_file);
}
const lock = JSON.parse(readFileSync('package-lock.json', 'utf8'));
for (const [path, entry] of Object.entries(lock.packages).filter(([path, entry]) => path && !entry.dev)) {
  const pkg = JSON.parse(readFileSync(resolve(path, 'package.json'), 'utf8'));
  collect(pkg.name, pkg.version, pkg.license, resolve(path));
}
sections.push('\nUpstream QR component notices\n', readFileSync('THIRD_PARTY_LICENSES.js', 'utf8'),
  '\nMa Shan Zheng font\n', readFileSync('public/fonts/OFL-MaShanZheng.txt', 'utf8'));
mkdirSync('public/licenses', { recursive: true });
writeFileSync('public/licenses/THIRD_PARTY_NOTICES.txt', sections.join('\n'));
console.log('Third-party notices written to public/licenses/THIRD_PARTY_NOTICES.txt');
if (missing.length) console.log(`Packages without a root license file (SPDX recorded): ${missing.join(', ')}`);
