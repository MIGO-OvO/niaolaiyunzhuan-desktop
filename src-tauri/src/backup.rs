use image::{GrayImage, ImageFormat, Luma};
use qrcode::{Color, EcLevel, QrCode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::io::Cursor;

pub const MAX_IMAGE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Backup {
    pub schema_version: u32,
    pub created_at: i64,
    pub completed_ids: Vec<String>,
    pub seen_new_ids: Vec<String>,
}

fn base36(mut value: u64) -> String {
    if value == 0 {
        return "0".into();
    }
    let mut digits = Vec::new();
    while value > 0 {
        digits.push(char::from_digit((value % 36) as u32, 36).expect("base36 digit"));
        value /= 36;
    }
    digits.iter().rev().collect()
}

fn token(id: &str) -> Option<u64> {
    let pieces: Vec<_> = id.strip_prefix("src-")?.split('-').collect();
    if pieces.is_empty()
        || pieces.len() > 2
        || pieces[0].len() != 3
        || !pieces[0].bytes().all(|c| c.is_ascii_digit())
    {
        return None;
    }
    let serial: u64 = pieces[0].parse().ok()?;
    let child = if pieces.len() == 2 {
        if pieces[1].len() != 2 || !pieces[1].bytes().all(|c| c.is_ascii_digit()) {
            return None;
        }
        pieces[1].parse::<u64>().ok()?
    } else {
        0
    };
    Some(serial * 100 + child)
}

fn encode_ids(ids: &[String]) -> String {
    let mut tokens: Vec<_> = ids.iter().filter_map(|id| token(id)).collect();
    tokens.sort_unstable();
    tokens.into_iter().map(base36).collect::<Vec<_>>().join(".")
}

fn decode_ids(value: &str) -> Result<Vec<String>, String> {
    if value.is_empty() {
        return Ok(Vec::new());
    }
    value
        .split('.')
        .map(|part| {
            let value = u64::from_str_radix(part, 36).map_err(|_| "备份中的目标编号无效")?;
            if !(100..=99_999).contains(&value) {
                return Err("备份中的目标编号超出范围".into());
            }
            let serial = value / 100;
            let child = value % 100;
            Ok(if child == 0 {
                format!("src-{serial:03}")
            } else {
                format!("src-{serial:03}-{child:02}")
            })
        })
        .collect()
}

pub fn checksum(value: &str) -> String {
    let hash = value.encode_utf16().fold(2_166_136_261_u32, |hash, unit| {
        (hash ^ u32::from(unit)).wrapping_mul(16_777_619)
    });
    format!("{hash:08x}")
}

pub fn encode(backup: &Backup) -> Result<String, String> {
    if backup.created_at < 0 {
        return Err("备份时间无效".into());
    }
    let body = format!(
        "NYNZ2|{}|{}|{}",
        base36(backup.created_at as u64),
        encode_ids(&backup.completed_ids),
        encode_ids(&backup.seen_new_ids)
    );
    Ok(format!("{body}|{}", checksum(&body)))
}

pub fn decode(payload: &str) -> Result<Backup, String> {
    if payload.len() > 8192 {
        return Err("备份内容过长".into());
    }
    let parts: Vec<_> = payload.trim().split('|').collect();
    if parts.len() != 5 || !["NYNZ1", "NYNZ2"].contains(&parts[0]) {
        return Err("这不是袅来运转生成的备份图".into());
    }
    if checksum(&parts[..4].join("|")) != parts[4] {
        return Err("备份图校验失败，请选择清晰的原图".into());
    }
    let created_at = i64::from_str_radix(parts[1], 36).map_err(|_| "备份时间无法识别")?;
    if created_at < 0 || chrono::DateTime::from_timestamp_millis(created_at).is_none() {
        return Err("备份时间无效".into());
    }
    Ok(Backup {
        schema_version: if parts[0] == "NYNZ2" { 2 } else { 1 },
        created_at,
        completed_ids: decode_ids(parts[2])?,
        seen_new_ids: decode_ids(parts[3])?,
    })
}

pub fn to_png(payload: &str) -> Result<Vec<u8>, String> {
    decode(payload)?;
    let qr = QrCode::with_error_correction_level(payload.as_bytes(), EcLevel::M)
        .map_err(|e| format!("无法生成备份二维码：{e}"))?;
    let width = qr.width();
    let mut image = GrayImage::from_pixel(
        ((width + 8) * 8) as u32,
        ((width + 8) * 8) as u32,
        Luma([255]),
    );
    for y in 0..width {
        for x in 0..width {
            if qr[(x, y)] == Color::Dark {
                for dy in 0..8 {
                    for dx in 0..8 {
                        image.put_pixel(
                            ((x + 4) * 8 + dx) as u32,
                            ((y + 4) * 8 + dy) as u32,
                            Luma([0]),
                        );
                    }
                }
            }
        }
    }
    let mut bytes = Cursor::new(Vec::new());
    image
        .write_to(&mut bytes, ImageFormat::Png)
        .map_err(|e| format!("无法编码备份图片：{e}"))?;
    Ok(bytes.into_inner())
}

pub fn from_image(bytes: &[u8]) -> Result<(String, Backup), String> {
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err("备份图片不能超过 16 MB".into());
    }
    let mut reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| format!("无法读取备份图片：{e}"))?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(4096);
    limits.max_image_height = Some(4096);
    limits.max_alloc = Some(64 * 1024 * 1024);
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|e| format!("图片无法识别或超过 4096×4096 像素：{e}"))?
        .to_luma8();
    let mut prepared = rqrr::PreparedImage::prepare(image);
    let mut payloads = BTreeSet::new();
    for grid in prepared.detect_grids() {
        if let Ok((_, payload)) = grid.decode() {
            if decode(&payload).is_ok() {
                payloads.insert(payload);
            }
        }
    }
    if payloads.len() > 1 {
        return Err("图片中有多份备份，请选择只包含一份备份的原图".into());
    }
    let payload = payloads
        .into_iter()
        .next()
        .ok_or("没有识别到有效的备份二维码，请选择清晰的原图")?;
    let backup = decode(&payload)?;
    Ok((payload, backup))
}
