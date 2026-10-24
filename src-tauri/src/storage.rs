use crate::{catalog::Catalog, state::Progress};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const MAX_STATE_BYTES: u64 = 4 * 1024 * 1024;

pub struct Store {
    directory: PathBuf,
}

pub struct Loaded {
    pub progress: Option<Progress>,
    pub warning: Option<String>,
}

impl Store {
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }

    pub fn load(&self, catalog: &Catalog) -> Result<Loaded, String> {
        let read = |name: &str| -> Result<Option<Progress>, String> {
            let path = self.directory.join(name);
            let file = match File::open(&path) {
                Ok(file) => file,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(e) => return Err(format!("无法读取 {}：{e}", path.display())),
            };
            let mut bytes = Vec::new();
            file.take(MAX_STATE_BYTES + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| format!("存档读取失败：{e}"))?;
            if bytes.len() as u64 > MAX_STATE_BYTES {
                return Err("存档超过 4 MB".into());
            }
            let value: serde_json::Value =
                serde_json::from_slice(&bytes).map_err(|e| format!("存档格式损坏：{e}"))?;
            if !value.get("completed").is_some_and(|v| v.is_object()) {
                return Err("存档缺少 completed 记录".into());
            }
            let progress: Progress =
                serde_json::from_value(value).map_err(|e| format!("存档格式损坏：{e}"))?;
            Ok(Some(progress.normalize(catalog)?))
        };
        let primary = read("progress.json");
        let backup = read("progress.previous.json");
        // A future schema must never fall back to an older copy and overwrite newer records.
        if [&primary, &backup]
            .iter()
            .any(|r| matches!(r, Err(e) if e.contains("较新版本")))
        {
            return Err("存档来自较新版本，请升级桌面版后再打开".into());
        }
        match (primary, backup) {
            (Ok(Some(current)), _) => Ok(Loaded {
                progress: Some(current),
                warning: None,
            }),
            (primary, Ok(Some(previous))) => Ok(Loaded {
                progress: Some(previous),
                warning: Some(format!(
                    "已从上一份存档恢复记录。{}",
                    primary.err().unwrap_or_else(|| "主存档缺失".into())
                )),
            }),
            (Ok(None), Ok(None)) => Ok(Loaded {
                progress: None,
                warning: None,
            }),
            (Err(error), _) | (_, Err(error)) => {
                Err(format!("无法打开进度，原文件已保留。{error}"))
            }
        }
    }

    pub fn save(&self, progress: &Progress, previous: &Progress) -> Result<(), String> {
        fs::create_dir_all(&self.directory).map_err(|e| format!("无法创建存档目录：{e}"))?;
        let bytes = serde_json::to_vec(progress).map_err(|e| format!("无法编码进度：{e}"))?;
        let old = serde_json::to_vec(previous).map_err(|e| format!("无法编码上一份进度：{e}"))?;
        atomic_write(&self.directory.join("progress.previous.json"), &old)?;
        atomic_write(&self.directory.join("progress.json"), &bytes)
    }
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or("文件路径缺少父目录")?;
    let mut temporary =
        tempfile::NamedTempFile::new_in(parent).map_err(|e| format!("无法创建临时文件：{e}"))?;
    temporary
        .write_all(bytes)
        .and_then(|_| temporary.as_file().sync_all())
        .map_err(|e| format!("写入失败：{e}"))?;
    temporary
        .persist(path)
        .map_err(|e| format!("无法保存 {}：{}", path.display(), e.error))?;
    Ok(())
}
