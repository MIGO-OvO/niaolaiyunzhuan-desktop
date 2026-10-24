use std::{error::Error, time::Duration};
use tauri_plugin_updater::UpdaterExt;

// 发布后验证真实更新端点与官方插件；仅下载和校验，不启动安装器。
fn main() -> Result<(), Box<dyn Error>> {
    let mut context = tauri::generate_context!();
    context.config_mut().app.windows.clear();
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .build(context)?;
    tauri::async_runtime::block_on(async {
        let updater = app
            .updater_builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        if let Some(update) = updater.check().await? {
            return Err(format!(
                "Expected current release; endpoint offers {}",
                update.version
            )
            .into());
        }
        // 工具允许读取当前版本以验证下载；正式应用沿用默认的“仅更新到较新版本”。
        let updater = app
            .updater_builder()
            .timeout(Duration::from_secs(120))
            .version_comparator(|_, _| true)
            .build()?;
        let mut update = updater
            .check()
            .await?
            .ok_or("Release manifest is missing")?;
        let mut bytes = update.download(|_, _| {}, || {}).await?;
        update.verify(&bytes)?;
        let version = update.version.clone();
        update.version = "9999.0.0".into();
        assert!(
            update.verify(&bytes).is_err(),
            "Wrong announced version must be rejected"
        );
        update.version = version;
        bytes[0] ^= 1;
        assert!(
            update.verify(&bytes).is_err(),
            "Tampered download must be rejected"
        );
        println!("GitHub v{}: current-version check, official download verification and rejection checks passed", update.version);
        Ok(())
    })
}
