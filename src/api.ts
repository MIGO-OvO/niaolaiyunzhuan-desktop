import { invoke, isTauri } from '@tauri-apps/api/core';
import type { RestorePreview, Snapshot } from './types';

export const desktop = isTauri();

// 浏览器预览也调用同一份 Rust 核心，使用独立的预览存档目录。
async function command<T>(name: string, args: Record<string, unknown> = {}): Promise<T> {
  if (desktop) return invoke<T>(name, args);
  const response = await fetch(`/api/${name}`, {
    method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(args),
  });
  if (!response.ok) throw new Error(await response.text());
  return response.json() as Promise<T>;
}

export const api = {
  snapshot: () => command<Snapshot>('get_snapshot'),
  check: (atomIds: string[], checked: boolean) => command<Snapshot>('set_checked', { atomIds, checked }),
  seen: (module: string) => command<Snapshot>('mark_seen', { module }),
  reset: (module: string | null) => command<Snapshot>('reset_progress', { module }),
  restore: (payload: string) => command<Snapshot>('restore_backup', { payload }),
  preview: async () => {
    if (desktop) return command<RestorePreview | null>('preview_restore');
    const file = await new Promise<File | null>(resolve => {
      const input = document.createElement('input');
      input.type = 'file'; input.accept = '.png,.jpg,.jpeg';
      input.onchange = () => resolve(input.files?.[0] ?? null);
      input.oncancel = () => resolve(null);
      input.click();
    });
    if (!file) return null;
    if (file.size > 16 * 1024 * 1024) throw new Error('备份图片不能超过 16 MB');
    return command<RestorePreview>('preview_restore', { bytes: Array.from(new Uint8Array(await file.arrayBuffer())) });
  },
  export: async () => {
    const result = await command<{ snapshot: Snapshot; path: string; bytes?: number[] } | null>('export_backup');
    if (!desktop && result?.bytes) {
      const url = URL.createObjectURL(new Blob([new Uint8Array(result.bytes)], { type: 'image/png' }));
      const link = document.createElement('a'); link.href = url; link.download = result.path; link.click();
      window.setTimeout(() => URL.revokeObjectURL(url), 1000);
    }
    return result;
  },
};
