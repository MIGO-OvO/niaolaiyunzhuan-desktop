//! Browser development uses the production tracker with a separate local save.
use chrono::Utc;
use niaolaiyunzhuan_lib::Tracker;
use serde_json::{json, Value};
use std::{
    io::{self, Read},
    path::Path,
};

fn handle(directory: &Path, input: &Value) -> Result<Value, String> {
    let mut tracker = Tracker::open(directory, Utc::now())?;
    let now = Utc::now();
    let args = &input["args"];
    match input["command"].as_str().ok_or("缺少命令")? {
        "get_snapshot" => tracker.refresh(now)?,
        "set_checked" => {
            let ids: Vec<String> =
                serde_json::from_value(args["atomIds"].clone()).map_err(|_| "目标编号无效")?;
            let ids: Vec<_> = ids.iter().map(String::as_str).collect();
            tracker.set_checked_many(
                &ids,
                args["checked"].as_bool().ok_or("缺少勾选状态")?,
                now,
            )?;
        }
        "mark_seen" => tracker.mark_seen(args["module"].as_str().ok_or("缺少分类")?, now)?,
        "reset_progress" => {
            let module: Option<String> =
                serde_json::from_value(args["module"].clone()).map_err(|_| "分类无效")?;
            tracker.reset(module.as_deref(), now)?;
        }
        "restore_backup" => {
            tracker.restore_payload(args["payload"].as_str().ok_or("缺少备份内容")?, now)?
        }
        "preview_restore" => {
            let bytes: Vec<u8> =
                serde_json::from_value(args["bytes"].clone()).map_err(|_| "图片数据无效")?;
            return serde_json::to_value(tracker.preview_restore(&bytes, now)?)
                .map_err(|e| e.to_string());
        }
        "export_backup" => {
            let path = directory.join("袅来运转-预览备份.png");
            tracker.export_backup(&path, now)?;
            let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
            return Ok(
                json!({ "snapshot": tracker.snapshot(now)?, "path": "袅来运转-预览备份.png", "bytes": bytes }),
            );
        }
        _ => return Err("未知命令".into()),
    }
    serde_json::to_value(tracker.snapshot(now)?).map_err(|e| e.to_string())
}

fn main() {
    let result = (|| {
        let directory = std::env::args().nth(1).ok_or("缺少预览存档目录")?;
        let mut text = String::new();
        io::stdin()
            .take(24 * 1024 * 1024 + 1)
            .read_to_string(&mut text)
            .map_err(|e| e.to_string())?;
        if text.len() > 24 * 1024 * 1024 {
            return Err("请求超过大小限制".into());
        }
        let input: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        handle(Path::new(&directory), &input)
    })();
    println!(
        "{}",
        match result {
            Ok(value) => json!({ "ok": true, "value": value }),
            Err(error) => json!({ "ok": false, "error": error }),
        }
    );
}
