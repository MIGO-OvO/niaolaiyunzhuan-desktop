import type { AtomView, ItemView, Stats, StatusFilter } from './types.ts';

export function statsFor(items: ItemView[]): Stats {
  return items.reduce<Stats>((stats, item) => {
    for (const atom of item.atoms) {
      const quantity = atom.quantity ?? (item.children.length ? 1 : 0);
      stats.totalTasks++;
      stats.totalSounds += quantity;
      stats.doneTasks += Number(atom.checked);
      stats.doneSounds += atom.checked ? quantity : 0;
      stats.newCount += Number(atom.unseen);
    }
    return stats;
  }, { totalTasks: 0, doneTasks: 0, totalSounds: 0, doneSounds: 0, newCount: 0 });
}

export function matchesStatus(atom: AtomView, status: StatusFilter): boolean {
  return status === 'all' || atom.checked === (status === 'done');
}

export function matchingAtoms(item: ItemView, query: string, status: StatusFilter): AtomView[] {
  const words = query.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean);
  const shared = `${item.displayTitle} ${item.children.length ? '' : item.condition} ${item.note} ${item.region} ${item.type}`.toLocaleLowerCase();
  return item.atoms.filter(atom => matchesStatus(atom, status)
    && words.every(word => `${shared} ${atom.label.toLocaleLowerCase()}`.includes(word)));
}

export function periodLabel(period: string): string {
  return ({ once: '一次性', weekly: '每周刷新', monthly: '每月刷新', season: '赛季', phase: '本期', event: '活动' } as Record<string, string>)[period] ?? period;
}

export function dateLabel(value: string | number, withTime = false): string {
  return new Intl.DateTimeFormat('zh-CN', {
    timeZone: 'Asia/Shanghai', month: '2-digit', day: '2-digit',
    ...(withTime ? { hour: '2-digit', minute: '2-digit', hour12: false } : { year: 'numeric' }),
  }).format(new Date(value));
}

export function countdown(end: string, now: number): string {
  const minutes = Math.ceil((new Date(end).getTime() - now) / 60000);
  if (minutes <= 0) return '已结束';
  const days = Math.floor(minutes / 1440);
  const hours = Math.floor(minutes % 1440 / 60);
  return days ? `剩余 ${days} 天 ${hours} 小时` : `剩余 ${hours} 小时 ${minutes % 60} 分`;
}
