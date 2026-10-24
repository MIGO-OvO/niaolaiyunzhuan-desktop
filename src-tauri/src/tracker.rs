use crate::backup::{self, Backup};
use crate::catalog::{Atom, Catalog, Item, MODULES};
use crate::state::{completion_key, notice_keys, resolve_id, BackupStatus, Progress};
use crate::storage::{atomic_write, Store};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub struct Tracker {
    catalog: Catalog,
    progress: Progress,
    store: Store,
    warning: Option<String>,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub total_tasks: u32,
    pub done_tasks: u32,
    pub total_sounds: u32,
    pub done_sounds: u32,
    pub new_count: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AtomView {
    #[serde(flatten)]
    pub atom: Atom,
    pub checked: bool,
    pub unseen: bool,
}

#[derive(Serialize)]
pub struct ItemView {
    #[serde(flatten)]
    pub item: Item,
    pub atoms: Vec<AtomView>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub meta: serde_json::Value,
    pub items: Vec<ItemView>,
    pub modules: BTreeMap<String, Stats>,
    pub overall: Stats,
    pub saved_at: i64,
    pub now: i64,
    pub backup: BackupStatus,
    pub backup_current: bool,
    pub storage_path: String,
    pub warning: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorePreview {
    pub payload: String,
    pub created_at: i64,
    pub completed_count: usize,
    pub added_count: usize,
    pub unknown_ids: Vec<String>,
}

impl Tracker {
    pub fn open(directory: &Path, now: DateTime<Utc>) -> Result<Self, String> {
        let catalog = Catalog::bundled()?;
        let store = Store::new(directory);
        let loaded = store.load(&catalog)?;
        let fresh = loaded.progress.is_none();
        let progress = loaded
            .progress
            .unwrap_or_else(|| Progress::new(&catalog, now));
        let mut tracker = Self {
            catalog,
            progress,
            store,
            warning: loaded.warning,
        };
        let mut refreshed = tracker.progress.clone();
        refreshed.refresh(&tracker.catalog, now);
        if fresh || tracker.warning.is_some() || refreshed != tracker.progress {
            tracker.commit(refreshed, now)?;
        }
        Ok(tracker)
    }

    pub fn progress(&self) -> &Progress {
        &self.progress
    }

    fn commit(&mut self, mut candidate: Progress, now: DateTime<Utc>) -> Result<(), String> {
        candidate.saved_at = now.timestamp_millis().max(self.progress.saved_at + 1);
        self.store.save(&candidate, &self.progress)?;
        self.progress = candidate;
        Ok(())
    }

    pub fn refresh(&mut self, now: DateTime<Utc>) -> Result<(), String> {
        let mut candidate = self.progress.clone();
        candidate.refresh(&self.catalog, now);
        if candidate != self.progress {
            self.commit(candidate, now)?;
        }
        Ok(())
    }

    pub fn set_checked(
        &mut self,
        atom_id: &str,
        checked: bool,
        now: DateTime<Utc>,
    ) -> Result<(), String> {
        self.set_checked_many(&[atom_id], checked, now)
    }

    pub fn set_checked_many(
        &mut self,
        atom_ids: &[&str],
        checked: bool,
        now: DateTime<Utc>,
    ) -> Result<(), String> {
        let mut candidate = self.progress.clone();
        candidate.refresh(&self.catalog, now);
        for atom_id in atom_ids {
            let (item, _) = self
                .catalog
                .find_atom(atom_id)
                .ok_or("找不到这个勾选目标")?;
            if !item.visible(now) {
                return Err("这个来源已结束，请刷新列表".into());
            }
            let key = completion_key(item, atom_id, now);
            if checked {
                candidate.completed.insert(key, true);
            } else {
                candidate.completed.remove(&key);
            }
        }
        if candidate != self.progress {
            self.commit(candidate, now)?;
        }
        Ok(())
    }

    pub fn mark_seen(&mut self, module: &str, now: DateTime<Utc>) -> Result<(), String> {
        if !MODULES.contains(&module) {
            return Err("未知分类".into());
        }
        let mut candidate = self.progress.clone();
        candidate.refresh(&self.catalog, now);
        for item in self
            .catalog
            .items
            .iter()
            .filter(|item| item.module == module && item.visible(now))
        {
            for atom in item.atoms() {
                for key in notice_keys(item, &atom.id, atom.is_new, now) {
                    candidate.seen_new_ids.insert(key, true);
                }
            }
        }
        if candidate != self.progress {
            self.commit(candidate, now)?;
        }
        Ok(())
    }

    pub fn reset(&mut self, module: Option<&str>, now: DateTime<Utc>) -> Result<(), String> {
        if module.is_some_and(|module| !MODULES.contains(&module)) {
            return Err("未知分类".into());
        }
        let mut candidate = self.progress.clone();
        if let Some(module) = module {
            for item in self
                .catalog
                .items
                .iter()
                .filter(|item| item.module == module)
            {
                candidate.clear_item(item);
            }
        } else {
            candidate.completed.clear();
            candidate.backup = BackupStatus::default();
        }
        self.commit(candidate, now)
    }

    fn current_backup(&self, now: DateTime<Utc>) -> Backup {
        Backup {
            schema_version: 2,
            created_at: now.timestamp_millis(),
            completed_ids: self
                .catalog
                .items
                .iter()
                .flat_map(|item| {
                    item.atoms()
                        .into_iter()
                        .filter(|atom| self.progress.checked(item, &atom.id, now))
                        .map(|atom| atom.id)
                })
                .collect(),
            seen_new_ids: self.progress.seen_new_ids.keys().cloned().collect(),
        }
    }

    fn fingerprint(mut backup: Backup) -> Result<String, String> {
        backup.created_at = 0;
        backup::encode(&backup)
    }

    pub fn export_backup(&mut self, path: &Path, now: DateTime<Utc>) -> Result<(), String> {
        if !path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("png"))
        {
            return Err("备份文件请使用 .png 扩展名".into());
        }
        self.refresh(now)?;
        let backup = self.current_backup(now);
        let payload = backup::encode(&backup)?;
        atomic_write(path, &backup::to_png(&payload)?)?;
        let mut candidate = self.progress.clone();
        candidate.backup = BackupStatus {
            last_at: backup.created_at,
            last_fingerprint: Self::fingerprint(backup)?,
        };
        self.commit(candidate, now)
            .map_err(|error| format!("备份图片已保存，但本地备份时间未写入：{error}"))
    }

    fn migrated_backup(&self, mut backup: Backup) -> (Backup, Vec<String>) {
        let mut completed: BTreeSet<String> = backup
            .completed_ids
            .iter()
            .map(|id| resolve_id(id).to_owned())
            .collect();
        if backup.schema_version < 2 && completed.contains("src-088") {
            completed.insert("src-088-02".into());
        }
        let unknown: Vec<_> = completed
            .iter()
            .filter(|id| self.catalog.find_atom(id).is_none())
            .cloned()
            .collect();
        backup.completed_ids = completed
            .into_iter()
            .filter(|id| self.catalog.find_atom(id).is_some())
            .collect();
        backup.seen_new_ids = backup
            .seen_new_ids
            .iter()
            .map(|id| resolve_id(id).to_owned())
            .filter(|id| self.catalog.find_atom(id).is_some())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        (backup, unknown)
    }

    pub fn preview_restore(
        &self,
        bytes: &[u8],
        now: DateTime<Utc>,
    ) -> Result<RestorePreview, String> {
        let (payload, decoded) = backup::from_image(bytes)?;
        let (backup, unknown_ids) = self.migrated_backup(decoded);
        let mut candidate = self.progress.clone();
        candidate.refresh(&self.catalog, now);
        let added_count = backup
            .completed_ids
            .iter()
            .filter(|id| {
                self.catalog
                    .find_atom(id)
                    .is_some_and(|(item, _)| !candidate.checked(item, id, now))
            })
            .count();
        Ok(RestorePreview {
            payload,
            created_at: backup.created_at,
            completed_count: backup.completed_ids.len(),
            added_count,
            unknown_ids,
        })
    }

    pub fn restore_payload(&mut self, payload: &str, now: DateTime<Utc>) -> Result<(), String> {
        let (backup, _) = self.migrated_backup(backup::decode(payload)?);
        let mut candidate = self.progress.clone();
        candidate.refresh(&self.catalog, now);
        for id in &backup.completed_ids {
            if let Some((item, _)) = self.catalog.find_atom(id) {
                candidate
                    .completed
                    .insert(completion_key(item, id, now), true);
            }
        }
        for id in &backup.seen_new_ids {
            candidate.seen_new_ids.insert(id.clone(), true);
        }
        candidate.backup = BackupStatus {
            last_at: backup.created_at,
            last_fingerprint: Self::fingerprint(backup)?,
        };
        self.commit(candidate, now)
    }

    pub fn snapshot(&self, now: DateTime<Utc>) -> Result<Snapshot, String> {
        let mut modules: BTreeMap<String, Stats> = MODULES
            .iter()
            .map(|module| ((*module).into(), Stats::default()))
            .collect();
        let mut overall = Stats::default();
        let mut items = Vec::new();
        for item in self.catalog.items.iter().filter(|item| item.visible(now)) {
            let atoms: Vec<_> = item
                .atoms()
                .into_iter()
                .map(|atom| {
                    let checked = self.progress.checked(item, &atom.id, now);
                    let unseen = notice_keys(item, &atom.id, atom.is_new, now)
                        .iter()
                        .any(|key| self.progress.seen_new_ids.get(key) != Some(&true));
                    AtomView {
                        atom,
                        checked,
                        unseen,
                    }
                })
                .collect();
            let stats = modules.get_mut(&item.module).ok_or("数据分类无效")?;
            for atom in &atoms {
                let quantity =
                    atom.atom
                        .quantity
                        .unwrap_or(if item.children.is_empty() { 0 } else { 1 });
                for stats in std::iter::once(&mut *stats)
                    .chain((item.module != "chance").then_some(&mut overall))
                {
                    stats.total_tasks += 1;
                    stats.total_sounds += quantity;
                    stats.done_tasks += u32::from(atom.checked);
                    stats.done_sounds += if atom.checked { quantity } else { 0 };
                    stats.new_count += u32::from(atom.unseen);
                }
            }
            items.push(ItemView {
                item: item.clone(),
                atoms,
            });
        }
        let fingerprint = Self::fingerprint(self.current_backup(now))?;
        Ok(Snapshot {
            meta: self.catalog.meta.clone(),
            items,
            modules,
            overall,
            saved_at: self.progress.saved_at,
            now: now.timestamp_millis(),
            backup: self.progress.backup.clone(),
            backup_current: self.progress.backup.last_at > 0
                && self.progress.backup.last_fingerprint == fingerprint,
            storage_path: self.store.directory().display().to_string(),
            warning: self.warning.clone(),
        })
    }
}
