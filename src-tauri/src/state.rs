use crate::catalog::{timestamp, Catalog, Item};
use chrono::{DateTime, Datelike, FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SCHEMA_VERSION: u32 = 8;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct Progress {
    pub schema_version: u32,
    pub saved_at: i64,
    pub completed: BTreeMap<String, bool>,
    pub seen_new_ids: BTreeMap<String, bool>,
    pub refresh_cycles: BTreeMap<String, String>,
    pub backup: BackupStatus,
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            schema_version: 1,
            saved_at: 0,
            completed: BTreeMap::new(),
            seen_new_ids: BTreeMap::new(),
            refresh_cycles: BTreeMap::new(),
            backup: BackupStatus::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct BackupStatus {
    pub last_at: i64,
    pub last_fingerprint: String,
}

pub fn resolve_id(id: &str) -> &str {
    if id == "src-027-05" {
        "src-027-04"
    } else {
        id
    }
}

fn migrate_key(key: &str) -> String {
    let (id, suffix) = key.split_once("::").unwrap_or((key, ""));
    if suffix.is_empty() {
        resolve_id(id).into()
    } else {
        format!("{}::{suffix}", resolve_id(id))
    }
}

pub fn china_time(now: DateTime<Utc>) -> DateTime<FixedOffset> {
    now.with_timezone(&FixedOffset::east_opt(8 * 3600).expect("UTC+8 is a valid offset"))
}

pub fn period_scope(period: &str, now: DateTime<Utc>) -> String {
    let date = china_time(now);
    match period {
        "weekly" => format!(
            "week:{}-W{:02}",
            date.iso_week().year(),
            date.iso_week().week()
        ),
        "monthly" => format!("month:{}-{:02}", date.year(), date.month()),
        "season" => "season:current".into(),
        "phase" => "phase:current".into(),
        "event" => "event:current".into(),
        _ => "once".into(),
    }
}

pub fn completion_key(item: &Item, atom_id: &str, now: DateTime<Utc>) -> String {
    format!("{atom_id}::{}", period_scope(&item.period, now))
}

pub fn occurrence(item: &Item, now: DateTime<Utc>) -> Option<String> {
    match item.refresh_rule.as_deref() {
        Some("monthly") => Some(period_scope("monthly", now)),
        // The upstream cycle ID identifies the current data revision; refreshAt is its deadline.
        Some("scheduled") if item.refresh_at.as_deref().and_then(timestamp).is_some() => item
            .refresh_cycle_id
            .as_ref()
            .filter(|id| !id.is_empty())
            .map(|id| format!("cycle:{id}")),
        _ => None,
    }
}

pub fn notice_keys(item: &Item, id: &str, is_new: bool, now: DateTime<Utc>) -> Vec<String> {
    let mut keys = Vec::new();
    if is_new {
        keys.push(id.to_owned());
    }
    if let Some(cycle) = occurrence(item, now) {
        keys.push(format!("{id}::refresh:{cycle}"));
    }
    keys
}

impl Progress {
    pub fn new(catalog: &Catalog, now: DateTime<Utc>) -> Self {
        let mut state = Self {
            schema_version: SCHEMA_VERSION,
            ..Self::default()
        };
        for item in &catalog.items {
            for atom in item.atoms() {
                for key in notice_keys(item, &atom.id, atom.is_new, now) {
                    state.seen_new_ids.insert(key, true);
                }
            }
        }
        state.refresh(catalog, now);
        state
    }

    pub fn normalize(mut self, catalog: &Catalog) -> Result<Self, String> {
        let version = self.schema_version;
        if version > SCHEMA_VERSION {
            return Err("存档来自较新版本，请升级桌面版后再打开".into());
        }
        if self.saved_at < 0
            || DateTime::from_timestamp_millis(self.saved_at).is_none()
            || self.backup.last_at < 0
            || DateTime::from_timestamp_millis(self.backup.last_at).is_none()
        {
            return Err("存档时间无效".into());
        }
        self.completed = self
            .completed
            .iter()
            .filter(|(_, v)| **v)
            .map(|(k, _)| (migrate_key(k), true))
            .collect();
        self.seen_new_ids = self
            .seen_new_ids
            .iter()
            .filter(|(_, v)| **v)
            .map(|(k, _)| (migrate_key(k), true))
            .collect();
        if version < 4 {
            for key in self
                .completed
                .clone()
                .keys()
                .filter(|k| k.starts_with("src-088::"))
            {
                self.completed
                    .insert(key.replacen("src-088::", "src-088-02::", 1), true);
            }
        }
        if version < 6 {
            self.completed.retain(|k, _| !k.starts_with("src-072-09::"));
        }
        if version < 7 {
            for item in &catalog.items {
                if item.refresh_rule.as_deref() != Some("scheduled") {
                    continue;
                }
                if let (Some(at), Some(cycle)) = (&item.refresh_at, &item.refresh_cycle_id) {
                    for atom in item.atoms() {
                        if self
                            .seen_new_ids
                            .remove(&format!("{}::refresh:at:{at}", atom.id))
                            .is_some()
                        {
                            self.seen_new_ids
                                .insert(format!("{}::refresh:cycle:{cycle}", atom.id), true);
                        }
                    }
                    if self.refresh_cycles.get(&item.id) == Some(&format!("at:{at}")) {
                        self.refresh_cycles
                            .insert(item.id.clone(), format!("cycle:{cycle}"));
                    }
                }
            }
        }
        if version < 8 {
            if self.completed.remove("src-096::season:current").is_some() {
                self.completed.insert("src-096-02::once".into(), true);
            }
            self.refresh_cycles.remove("src-096");
        }
        self.schema_version = SCHEMA_VERSION;
        Ok(self)
    }

    pub fn checked(&self, item: &Item, atom_id: &str, now: DateTime<Utc>) -> bool {
        self.completed.get(&completion_key(item, atom_id, now)) == Some(&true)
    }

    pub fn clear_item(&mut self, item: &Item) {
        let prefixes: Vec<_> = item
            .atoms()
            .iter()
            .map(|atom| format!("{}::", atom.id))
            .collect();
        self.completed
            .retain(|key, _| !prefixes.iter().any(|prefix| key.starts_with(prefix)));
    }

    pub fn refresh(&mut self, catalog: &Catalog, now: DateTime<Utc>) {
        for item in &catalog.items {
            let Some(current) = occurrence(item, now) else {
                continue;
            };
            let previous = self.refresh_cycles.get(&item.id);
            if previous == Some(&current) {
                continue;
            }
            let last = DateTime::from_timestamp_millis(self.saved_at)
                .and_then(|date| occurrence(item, date));
            if previous.is_some()
                || (self.saved_at > 0
                    && (item.refresh_rule.as_deref() == Some("scheduled")
                        || last.as_ref() != Some(&current)))
            {
                self.clear_item(item);
            }
            self.refresh_cycles.insert(item.id.clone(), current);
        }
    }
}
