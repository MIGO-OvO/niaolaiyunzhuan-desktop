use chrono::{DateTime, Duration, Utc};
use image::{GrayImage, ImageFormat, Luma};
use niaolaiyunzhuan_lib::{
    backup::{self, Backup},
    catalog::{timestamp, Catalog},
    state::{completion_key, occurrence, period_scope, Progress},
    storage::{atomic_write, Store},
    Tracker,
};
use serde_json::{json, Value};
use std::{fs, io::Cursor};

fn now() -> DateTime<Utc> {
    timestamp("2026-10-24T12:00:00+08:00").unwrap()
}
fn fixtures() -> Value {
    serde_json::from_str(include_str!("fixtures/upstream.json")).unwrap()
}
fn payload(ids: &[&str]) -> String {
    backup::encode(&Backup {
        schema_version: 2,
        created_at: now().timestamp_millis(),
        completed_ids: ids.iter().map(|id| (*id).into()).collect(),
        seen_new_ids: vec![],
    })
    .unwrap()
}

#[test]
fn bundled_catalog_and_visible_statistics_match_upstream() {
    let catalog = Catalog::bundled().unwrap();
    assert_eq!(catalog.items.len(), 128);
    assert_eq!(
        catalog.items.iter().map(|i| i.atoms().len()).sum::<usize>(),
        207
    );
    let fixture = fixtures();
    let visible: Vec<_> = catalog
        .items
        .iter()
        .filter(|item| item.visible(now()))
        .map(|item| item.id.clone())
        .collect();
    assert_eq!(json!(visible), fixture["visibleIds"]);
    let dir = tempfile::tempdir().unwrap();
    let tracker = Tracker::open(dir.path(), now()).unwrap();
    let snapshot = tracker.snapshot(now()).unwrap();
    let mut stats = serde_json::to_value(snapshot.overall).unwrap();
    stats.as_object_mut().unwrap().remove("newCount");
    assert_eq!(stats, fixture["overall"]);
    assert!(snapshot.modules.values().all(|stats| stats.new_count == 0));
}

#[test]
fn group_check_is_atomic_and_survives_reopening() {
    let dir = tempfile::tempdir().unwrap();
    let mut tracker = Tracker::open(dir.path(), now()).unwrap();
    let original = tracker.progress().clone();
    assert!(tracker
        .set_checked_many(&["src-005-01", "missing"], true, now())
        .is_err());
    assert_eq!(tracker.progress(), &original);
    let ids: Vec<String> = Catalog::bundled()
        .unwrap()
        .items
        .iter()
        .find(|item| item.id == "src-005")
        .unwrap()
        .atoms()
        .into_iter()
        .map(|atom| atom.id)
        .collect();
    let refs: Vec<_> = ids.iter().map(String::as_str).collect();
    tracker.set_checked_many(&refs, true, now()).unwrap();
    let tracker = Tracker::open(dir.path(), now()).unwrap();
    let snapshot = tracker.snapshot(now()).unwrap();
    let group = snapshot
        .items
        .iter()
        .find(|item| item.item.id == "src-005")
        .unwrap();
    assert!(group.atoms.iter().all(|atom| atom.checked));
    assert_eq!(snapshot.overall.done_tasks, 8);
}

#[test]
fn state_migration_matches_original_javascript() {
    let fixture = fixtures();
    let old: Progress = serde_json::from_value(fixture["oldState"].clone()).unwrap();
    let migrated = old.normalize(&Catalog::bundled().unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(migrated).unwrap(),
        fixture["migratedState"]
    );
}

#[test]
fn backup_payload_matches_original_javascript_byte_for_byte() {
    for case in fixtures()["cases"].as_array().unwrap() {
        let input = &case["input"];
        let backup = Backup {
            schema_version: 2,
            created_at: input["createdAt"].as_i64().unwrap(),
            completed_ids: serde_json::from_value(input["completedIds"].clone()).unwrap(),
            seen_new_ids: serde_json::from_value(input["seenNewIds"].clone()).unwrap(),
        };
        let encoded = backup::encode(&backup).unwrap();
        assert_eq!(encoded, case["payload"].as_str().unwrap());
        let decoded = backup::decode(&encoded).unwrap();
        assert_eq!(decoded.completed_ids.len(), backup.completed_ids.len());
        assert_eq!(decoded.created_at, backup.created_at);
    }
}

#[test]
fn reads_original_qr_encoder_in_original_size_backup_frame() {
    let fixture = fixtures();
    let matrix = fixture["matrix"].as_array().unwrap();
    let size = matrix.len() as u32;
    let cell = 648 / size;
    let offset_x = 126 + (648 - size * cell) / 2;
    let offset_y = 274 + (648 - size * cell) / 2;
    let mut image = GrayImage::from_pixel(900, 1180, Luma([255]));
    for (y, row) in matrix.iter().enumerate() {
        for (x, bit) in row.as_str().unwrap().bytes().enumerate() {
            if bit == b'1' {
                for dy in 0..cell {
                    for dx in 0..cell {
                        image.put_pixel(
                            offset_x + x as u32 * cell + dx,
                            offset_y + y as u32 * cell + dy,
                            Luma([35]),
                        );
                    }
                }
            }
        }
    }
    let mut bytes = Cursor::new(Vec::new());
    image.write_to(&mut bytes, ImageFormat::Png).unwrap();
    let (decoded, _) = backup::from_image(bytes.get_ref()).unwrap();
    assert_eq!(decoded, fixture["cases"][1]["payload"].as_str().unwrap());
}

#[test]
fn png_round_trip_handles_full_catalog() {
    let fixture = fixtures();
    for case in fixture["cases"].as_array().unwrap() {
        let payload = case["payload"].as_str().unwrap();
        let png = backup::to_png(payload).unwrap();
        let (decoded, _) = backup::from_image(&png).unwrap();
        assert_eq!(decoded, payload);
    }
}

#[test]
fn rejects_corrupt_payloads_and_non_images() {
    assert!(backup::decode("NYNZ2|0|||00000000").is_err());
    assert!(backup::decode("other|0|||c0ffee").is_err());
    assert!(backup::from_image(b"not an image").is_err());
    let body = "NYNZ2|0|!|";
    assert!(backup::decode(&format!("{body}|{}", backup::checksum(body))).is_err());
    assert!(backup::decode(&"x".repeat(8193)).is_err());
}

#[test]
fn legacy_backup_aliases_merge_without_erasing_current_records() {
    let dir = tempfile::tempdir().unwrap();
    let mut tracker = Tracker::open(dir.path(), now()).unwrap();
    tracker.set_checked("src-001", true, now()).unwrap();
    let new_payload = payload(&["src-027-05", "src-088", "src-999"]);
    let body = new_payload
        .rsplit_once('|')
        .unwrap()
        .0
        .replacen("NYNZ2", "NYNZ1", 1);
    let legacy = format!("{body}|{}", backup::checksum(&body));
    let preview = tracker
        .preview_restore(&backup::to_png(&legacy).unwrap(), now())
        .unwrap();
    assert_eq!(preview.completed_count, 3);
    assert_eq!(preview.added_count, 3);
    assert_eq!(preview.unknown_ids, vec!["src-999"]);
    tracker.restore_payload(&legacy, now()).unwrap();
    let completed = &tracker.progress().completed;
    assert!(completed.contains_key("src-001::once"));
    assert!(completed.contains_key("src-027-04::once"));
    assert!(completed.contains_key("src-088::event:current"));
    assert!(completed.contains_key("src-088-02::event:current"));
    assert!(!completed.keys().any(|key| key.contains("999")));
}

#[test]
fn china_month_and_iso_week_boundaries() {
    let before = timestamp("2026-10-31T23:59:59+08:00").unwrap();
    let after = before + Duration::seconds(1);
    assert_eq!(period_scope("monthly", before), "month:2026-10");
    assert_eq!(period_scope("monthly", after), "month:2026-11");
    let sunday = timestamp("2027-01-03T23:59:59+08:00").unwrap();
    assert_eq!(period_scope("weekly", sunday), "week:2026-W53");
    assert_eq!(
        period_scope("weekly", sunday + Duration::seconds(1)),
        "week:2027-W01"
    );
}

#[test]
fn month_rollover_clears_monthly_only_and_announces_new_cycle() {
    let dir = tempfile::tempdir().unwrap();
    let mut tracker = Tracker::open(dir.path(), now()).unwrap();
    tracker.set_checked("src-001", true, now()).unwrap();
    let catalog = Catalog::bundled().unwrap();
    let monthly = catalog
        .items
        .iter()
        .find(|item| item.id == "src-084")
        .unwrap();
    let atom = &monthly.atoms()[0].id;
    tracker.set_checked(atom, true, now()).unwrap();
    let after = timestamp("2026-11-01T00:00:00+08:00").unwrap();
    tracker.refresh(after).unwrap();
    assert!(!tracker.progress().checked(monthly, atom, after));
    assert!(tracker.progress().completed.contains_key("src-001::once"));
    let snapshot = tracker.snapshot(after).unwrap();
    assert!(
        snapshot
            .items
            .iter()
            .find(|view| view.item.id == "src-084")
            .unwrap()
            .atoms[0]
            .unseen
    );
    tracker.mark_seen(&monthly.module, after).unwrap();
    assert!(
        !tracker
            .snapshot(after)
            .unwrap()
            .items
            .iter()
            .find(|view| view.item.id == "src-084")
            .unwrap()
            .atoms[0]
            .unseen
    );
}

#[test]
fn scheduled_cycle_is_id_based_and_refreshes_exactly_once() {
    let mut catalog = Catalog::bundled().unwrap();
    let mut progress = Progress::new(&catalog, now());
    let item = catalog
        .items
        .iter()
        .find(|item| item.id == "src-080")
        .unwrap();
    let atom = item.atoms()[0].id.clone();
    let key = completion_key(item, &atom, now());
    progress.completed.insert(key.clone(), true);
    progress.saved_at = now().timestamp_millis();
    assert_eq!(occurrence(item, now()).as_deref(), Some("cycle:2026-09-24"));
    progress.refresh(&catalog, now());
    assert!(progress.completed.contains_key(&key));
    catalog
        .items
        .iter_mut()
        .find(|item| item.id == "src-080")
        .unwrap()
        .refresh_cycle_id = Some("next".into());
    progress.refresh(&catalog, now());
    assert!(!progress.completed.contains_key(&key));
    progress.completed.insert(key.clone(), true);
    progress.refresh(&catalog, now());
    assert!(progress.completed.contains_key(&key));
}

#[test]
fn expiry_and_countdown_grace_period() {
    let catalog = Catalog::bundled().unwrap();
    let countdown = catalog
        .items
        .iter()
        .find(|item| item.id == "src-130")
        .unwrap();
    let deadline = timestamp(countdown.countdown_end.as_deref().unwrap()).unwrap();
    assert!(countdown.visible(deadline));
    assert!(countdown.visible(deadline + Duration::days(1) - Duration::milliseconds(1)));
    assert!(!countdown.visible(deadline + Duration::days(1)));
    let expiring = catalog
        .items
        .iter()
        .find(|item| item.id == "src-110-group")
        .unwrap();
    let expiry = timestamp(expiring.expires_at.as_deref().unwrap()).unwrap();
    assert!(expiring.visible(expiry - Duration::milliseconds(1)));
    assert!(!expiring.visible(expiry));
}

#[test]
fn saves_unchecks_and_reset_survive_restart() {
    let dir = tempfile::tempdir().unwrap();
    let mut tracker = Tracker::open(dir.path(), now()).unwrap();
    tracker.set_checked("src-001", true, now()).unwrap();
    tracker.set_checked("src-002", true, now()).unwrap();
    drop(tracker);
    let mut tracker = Tracker::open(dir.path(), now()).unwrap();
    assert_eq!(tracker.snapshot(now()).unwrap().overall.done_tasks, 2);
    tracker.set_checked("src-001", false, now()).unwrap();
    drop(tracker);
    let mut tracker = Tracker::open(dir.path(), now()).unwrap();
    assert_eq!(tracker.snapshot(now()).unwrap().overall.done_tasks, 1);
    tracker.reset(Some("region"), now()).unwrap();
    drop(tracker);
    assert_eq!(
        Tracker::open(dir.path(), now())
            .unwrap()
            .snapshot(now())
            .unwrap()
            .overall
            .done_tasks,
        0
    );
}

#[test]
fn recover_previous_file_and_report_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut tracker = Tracker::open(dir.path(), now()).unwrap();
    tracker.set_checked("src-001", true, now()).unwrap();
    tracker.set_checked("src-002", true, now()).unwrap();
    drop(tracker);
    fs::write(dir.path().join("progress.json"), b"broken").unwrap();
    let restored = Tracker::open(dir.path(), now()).unwrap();
    let snapshot = restored.snapshot(now()).unwrap();
    assert!(snapshot.warning.is_some());
    assert_eq!(snapshot.overall.done_tasks, 1);
}

#[test]
fn corrupt_or_future_files_are_not_overwritten() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("progress.json"), b"broken").unwrap();
    assert!(Tracker::open(dir.path(), now()).is_err());
    assert_eq!(
        fs::read(dir.path().join("progress.json")).unwrap(),
        b"broken"
    );
    let future = br#"{"schemaVersion":99,"completed":{"src-001::once":true}}"#;
    fs::write(dir.path().join("progress.json"), future).unwrap();
    fs::write(
        dir.path().join("progress.previous.json"),
        b"{\"completed\":{}}",
    )
    .unwrap();
    assert!(Tracker::open(dir.path(), now()).is_err());
    assert_eq!(fs::read(dir.path().join("progress.json")).unwrap(), future);
}

#[test]
fn unrelated_json_is_not_accepted_as_empty_progress() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("progress.json"), b"{\"unrelated\":true}").unwrap();
    assert!(Store::new(dir.path())
        .load(&Catalog::bundled().unwrap())
        .is_err());
}

#[test]
fn failed_save_keeps_memory_and_disk_progress() {
    let dir = tempfile::tempdir().unwrap();
    let mut tracker = Tracker::open(dir.path(), now()).unwrap();
    tracker.set_checked("src-001", true, now()).unwrap();
    let before = tracker.progress().clone();
    let bytes = fs::read(dir.path().join("progress.json")).unwrap();
    fs::rename(
        dir.path().join("progress.previous.json"),
        dir.path().join("previous.moved.json"),
    )
    .unwrap();
    fs::create_dir(dir.path().join("progress.previous.json")).unwrap();
    assert!(tracker.set_checked("src-002", true, now()).is_err());
    assert_eq!(tracker.progress(), &before);
    assert_eq!(fs::read(dir.path().join("progress.json")).unwrap(), bytes);
}

#[test]
fn export_marks_saved_only_after_success_and_restore_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let mut tracker = Tracker::open(dir.path(), now()).unwrap();
    tracker.set_checked("src-001", true, now()).unwrap();
    assert!(tracker
        .export_backup(&dir.path().join("missing/backup.png"), now())
        .is_err());
    assert_eq!(tracker.progress().backup.last_at, 0);
    let path = dir.path().join("backup.png");
    tracker.export_backup(&path, now()).unwrap();
    assert!(tracker.snapshot(now()).unwrap().backup_current);
    tracker.set_checked("src-002", true, now()).unwrap();
    assert!(!tracker.snapshot(now()).unwrap().backup_current);
    let preview = tracker
        .preview_restore(&fs::read(path).unwrap(), now())
        .unwrap();
    assert_eq!(preview.added_count, 0);
    tracker.restore_payload(&preview.payload, now()).unwrap();
    tracker.restore_payload(&preview.payload, now()).unwrap();
    assert_eq!(tracker.snapshot(now()).unwrap().overall.done_tasks, 2);
    assert!(!tracker.snapshot(now()).unwrap().backup_current);
}

#[test]
fn invalid_commands_leave_progress_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let mut tracker = Tracker::open(dir.path(), now()).unwrap();
    let before = tracker.progress().clone();
    assert!(tracker.set_checked("src-999", true, now()).is_err());
    assert!(tracker.set_checked("src-095", true, now()).is_err());
    assert!(tracker.mark_seen("nope", now()).is_err());
    assert!(tracker.reset(Some("nope"), now()).is_err());
    assert!(tracker.restore_payload("corrupt", now()).is_err());
    assert_eq!(tracker.progress(), &before);
}

#[test]
fn atomic_write_replaces_existing_files() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("save.json");
    atomic_write(&path, b"first").unwrap();
    atomic_write(&path, b"second").unwrap();
    assert_eq!(fs::read(path).unwrap(), b"second");
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}
