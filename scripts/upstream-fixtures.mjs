import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import vm from 'node:vm';

const source = process.argv[2];
if (!source) throw new Error('Usage: node scripts/upstream-fixtures.mjs <upstream checkout>');
process.env.TZ = 'Asia/Shanghai';
const now = Date.parse('2026-10-24T12:00:00+08:00');
class FixedDate extends Date {
  constructor(...args) { super(...(args.length ? args : [now])); }
  static now() { return now; }
}
const context = {
  window: {}, Date: FixedDate,
  document: { getElementById: () => ({ addEventListener() {} }) },
  localStorage: { getItem: () => null },
};
vm.createContext(context);
for (const file of ['data.js', 'backup.js', 'qrcode.js']) {
  vm.runInContext(readFileSync(resolve(source, file), 'utf8'), context, { timeout: 1000 });
}
let app = readFileSync(resolve(source, 'app.js'), 'utf8');
app = app.replace('  markContainerEnvironment();', '').replace('  enableFlexGapEnhancement();', '');
const boot = app.lastIndexOf('  renderHome();\n  hydrateFromMirror()');
if (boot < 0) throw new Error('Upstream boot changed; review fixture extraction.');
app = app.slice(0, boot) + 'window.oracle = { normalizeState, getVisibleItems, getStats, getAtoms, getPeriodScope };})();';
vm.runInContext(app, context, { timeout: 1000 });
const backup = context.window.NIAONIAO_BACKUP;
const allIds = context.window.NIAONIAO_DATA.items.flatMap(item => context.window.oracle.getAtoms(item).map(atom => atom.id));
const cases = [
  { createdAt: 0, completedIds: [], seenNewIds: [] },
  { createdAt: now, completedIds: ['src-001', 'src-005-02', 'src-130'], seenNewIds: ['src-130', 'src-084::refresh:month:2026-10'] },
  { createdAt: now, completedIds: allIds, seenNewIds: allIds },
  { createdAt: now, completedIds: ['src-027-05', 'src-088'], seenNewIds: ['src-027-05'] },
].map(input => ({ input, payload: backup.encode(input) }));
const qr = context.qrcode(0, 'M');
qr.addData(cases[1].payload, 'Byte');
qr.make();
const matrix = Array.from({ length: qr.getModuleCount() }, (_, row) =>
  Array.from({ length: qr.getModuleCount() }, (_, column) => qr.isDark(row, column) ? '1' : '0').join(''));
const oldState = {
  schemaVersion: 1, savedAt: now,
  completed: { 'src-027-05::once': true, 'src-088::event:current': true, 'src-072-09::once': true, 'src-096::season:current': true, 'src-001::once': false },
  seenNewIds: { 'src-027-05': true, 'src-080::refresh:at:2026-12-25T04:59:59+08:00': true },
  refreshCycles: { 'src-080': 'at:2026-12-25T04:59:59+08:00', 'src-096': 'cycle:old' },
};
const oracle = context.window.oracle;
const visible = oracle.getVisibleItems(now);
const fixtures = {
  sourceRevision: '8a88ea6c8532f1f67d51660bbb1dd46850af9f18', now, cases, matrix,
  oldState, migratedState: oracle.normalizeState(oldState),
  visibleIds: visible.map(item => item.id),
  overall: oracle.getStats(visible.filter(item => item.module !== 'chance')),
};
const contents = JSON.stringify(fixtures, null, 2);
process.stdout.write('*** Begin Patch\n*** Add File: src-tauri/tests/fixtures/upstream.json\n' + contents.split('\n').map(line => '+' + line).join('\n') + '\n*** End Patch\n');
