use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Catalog {
    pub meta: serde_json::Value,
    pub items: Vec<Item>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: String,
    pub source_serial: u32,
    pub module: String,
    pub region: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub region_type_rank: u64,
    pub quantity: Option<u32>,
    pub condition: String,
    pub display_title: String,
    pub note: String,
    pub countdown_end: Option<String>,
    pub refresh_rule: Option<String>,
    pub refresh_at: Option<String>,
    pub refresh_cycle_id: Option<String>,
    pub expires_at: Option<String>,
    pub period: String,
    pub children: Vec<Atom>,
    pub is_new: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Atom {
    pub id: String,
    pub label: String,
    pub quantity: Option<u32>,
    pub is_new: bool,
}

pub const MODULES: [&str; 4] = ["region", "gameplay", "limited", "chance"];

impl Catalog {
    pub fn bundled() -> Result<Self, String> {
        let catalog: Self = serde_json::from_str(include_str!("../../data/catalog.json"))
            .map_err(|e| format!("数据目录无法读取：{e}"))?;
        let mut ids = BTreeSet::new();
        for item in &catalog.items {
            if !MODULES.contains(&item.module.as_str()) {
                return Err(format!("未知分类：{}", item.module));
            }
            for atom in item.atoms() {
                if !ids.insert(atom.id.clone()) {
                    return Err(format!("重复目标：{}", atom.id));
                }
            }
        }
        Ok(catalog)
    }

    pub fn find_atom(&self, id: &str) -> Option<(&Item, Atom)> {
        self.items.iter().find_map(|item| {
            item.atoms()
                .into_iter()
                .find(|a| a.id == id)
                .map(|atom| (item, atom))
        })
    }
}

impl Item {
    pub fn atoms(&self) -> Vec<Atom> {
        if self.children.is_empty() {
            vec![Atom {
                id: self.id.clone(),
                label: self.condition.clone(),
                quantity: self.quantity,
                is_new: self.is_new,
            }]
        } else {
            self.children.clone()
        }
    }

    pub fn visible(&self, now: DateTime<Utc>) -> bool {
        if self
            .expires_at
            .as_deref()
            .and_then(timestamp)
            .is_some_and(|end| now >= end)
        {
            return false;
        }
        !self
            .countdown_end
            .as_deref()
            .and_then(timestamp)
            .is_some_and(|end| now >= end + chrono::Duration::days(1))
    }
}

pub fn timestamp(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|date| date.with_timezone(&Utc))
}
