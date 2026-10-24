import assert from 'node:assert/strict';
import test from 'node:test';
import catalog from '../data/catalog.json' with { type: 'json' };
import { countdown, matchingAtoms, statsFor } from './model.ts';
import type { Item, ItemView } from './types.ts';

function view(id: string, checked: string[] = []): ItemView {
  const item = catalog.items.find(item => item.id === id)! as Item;
  const atoms = item.children.length ? item.children : [{ id: item.id, label: item.condition, quantity: item.quantity, isNew: item.isNew }];
  return { ...item, atoms: atoms.map(atom => ({ ...atom, checked: checked.includes(atom.id), unseen: false })) };
}

test('group child search and completion filters apply to independent targets', () => {
  const item = view('src-005', ['src-005-01', 'src-005-02']);
  assert.equal(item.atoms.length, 8);
  assert.equal(matchingAtoms(item, item.atoms[0].label, 'all').length, 1);
  assert.equal(matchingAtoms(item, '清河 以武会友', 'all').length, 8);
  assert.equal(matchingAtoms(item, '', 'done').length, 2);
  assert.equal(matchingAtoms(item, '', 'todo').length, 6);
  assert.equal(matchingAtoms(item, '不存在的名字', 'all').length, 0);
});

test('region totals count children and quantities independently', () => {
  const items = catalog.items.filter(item => item.module === 'region' && item.region === '清河').map(item => view(item.id, ['src-001', 'src-002', 'src-005-01', 'src-005-02']));
  const stats = statsFor(items);
  assert.deepEqual(stats, { totalTasks: 13, doneTasks: 4, totalSounds: 15, doneSounds: 4, newCount: 0 });
});

test('countdown changes at the actual activity boundary', () => {
  const end = '2026-11-13T04:59:59+08:00';
  assert.equal(countdown(end, new Date(end).getTime()), '已结束');
  assert.equal(countdown(end, new Date(end).getTime() - 3600000), '剩余 1 小时 0 分');
});
