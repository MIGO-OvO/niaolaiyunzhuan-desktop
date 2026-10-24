export type ModuleId = 'region' | 'gameplay' | 'limited' | 'chance';
export type StatusFilter = 'all' | 'todo' | 'done';

export interface Atom {
  id: string;
  label: string;
  quantity: number | null;
  isNew: boolean;
}

export interface AtomView extends Atom {
  checked: boolean;
  unseen: boolean;
}

export interface Item {
  id: string;
  sourceSerial: number;
  module: ModuleId;
  region: string;
  type: string;
  regionTypeRank: number;
  quantity: number | null;
  condition: string;
  displayTitle: string;
  note: string;
  countdownEnd: string | null;
  refreshRule: string | null;
  refreshAt: string | null;
  refreshCycleId: string | null;
  expiresAt: string | null;
  period: string;
  children: Atom[];
  isNew: boolean;
}

export interface ItemView extends Item { atoms: AtomView[] }

export interface Stats {
  totalTasks: number;
  doneTasks: number;
  totalSounds: number;
  doneSounds: number;
  newCount: number;
}

export interface Snapshot {
  meta: { updatedAt: string; releaseVersion: string; sourceRevision: number };
  items: ItemView[];
  modules: Record<ModuleId, Stats>;
  overall: Stats;
  savedAt: number;
  now: number;
  backup: { lastAt: number; lastFingerprint: string };
  backupCurrent: boolean;
  storagePath: string;
  warning: string | null;
}

export interface RestorePreview {
  payload: string;
  createdAt: number;
  completedCount: number;
  addedCount: number;
  unknownIds: string[];
}
