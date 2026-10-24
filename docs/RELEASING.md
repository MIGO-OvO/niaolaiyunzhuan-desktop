# 发布 Windows 版本

正式版本由 `.github/workflows/release.yml` 构建。工作流读取版本标签，运行测试，构建安装包和更新签名，校验产物，再公开 GitHub Release。更新端点固定为：

```text
https://github.com/MIGO-OvO/niaolaiyunzhuan-desktop/releases/latest/download/latest.json
```

## 发布新版本

1. 同步 `package.json`、`package-lock.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json` 的版本号。`npm version 1.5.1 --no-git-tag-version` 可以同步两个 npm 文件；另两个文件同步改为 `1.5.1`。
2. 新建 `docs/releases/v1.5.1.md`，记录该版本的变化。
3. 运行验证并提交代码，然后推送版本标签：

```powershell
npm ci
npm test
npm run build
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
node scripts/release-manifest.mjs --check
git add .
git commit -m "Release 1.5.1"
git push origin main
git tag v1.5.1
git push origin v1.5.1
```

工作流生成 `*_x64-setup.exe`、`.exe.sig`、`latest.json` 和 `SHA256SUMS`。安装包使用 ASCII 附件名，更新清单中的下载地址与之对应。只有验证通过后才将 Release 从草稿改为正式版本。

## 签名密钥

仓库的 Actions Secrets 使用以下名称：

| Secret | 内容 |
| --- | --- |
| `TAURI_SIGNING_PRIVATE_KEY` | Tauri 生成的加密私钥内容 |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | 私钥密码 |

公钥位于 `src-tauri/tauri.conf.json`。更新器启用 `requireSignedVersion`，验证安装包签名与签名所绑定的版本号；使用 Tauri CLI 2.12.4 以上、updater 2.14 以上构建。

当前维护电脑上的密钥保存在 `%USERPROFILE%\.tauri\keys\niaolaiyunzhuan-desktop\release.key`，对应的 `password.dpapi.xml` 使用 Windows DPAPI 加密，只能在原 Windows 用户上下文解密。私钥及密码文件保存在仓库外。

请另行备份私钥和密码到维护者的密码管理器。GitHub Secrets 无法取回明文；丢失签名私钥会影响已安装版本接收后续更新。更换密钥前，需要先用旧密钥发布能够信任新公钥的过渡版本。

Tauri / Minisign 签名用于应用更新校验。Windows Authenticode 需要独立的代码签名证书，当前未配置。

## 在维护电脑上构建

```powershell
$keyDirectory = Join-Path $env:USERPROFILE '.tauri\keys\niaolaiyunzhuan-desktop'
$securePassword = Import-Clixml -LiteralPath (Join-Path $keyDirectory 'password.dpapi.xml')
$releaseCredential = [PSCredential]::new('release', $securePassword)
$env:TAURI_SIGNING_PRIVATE_KEY = Join-Path $keyDirectory 'release.key'
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = $releaseCredential.GetNetworkCredential().Password
try {
    node scripts/collect-licenses.mjs
    npm run package -- --ci
    if ($LASTEXITCODE -ne 0) { throw 'Build failed' }
    node scripts/release-manifest.mjs
    cargo run --locked --no-default-features --example verify-update -- .local/release
} finally {
    Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY
    Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD
    $releaseCredential = $null
}
```

产物保存在 `.local/release`。验证工具使用与 updater 相同的 Minisign 校验库，检查真实安装包、公钥、签名和版本号，并确认篡改后的文件、错误的版本号均被拒绝。

## 发布后检查

- Release 为正式发布，标签与程序版本一致。
- `releases/latest/download/latest.json` 可以公开访问，平台键为 `windows-x86_64`。
- 从 Release 下载实际附件，重新验证签名与 SHA-256。
- 在已安装版本中检查更新、下载并安装，确认更新后继续读取原存档。

`cargo run --locked --example check-updater` 使用官方插件访问公开端点，确认当前版本无需更新，再下载当前发布包验证签名，并检查篡改文件和错误版本号被拒绝。此工具仅下载和校验。发布工作流会在公开 Release 后执行这一检查。

安装和更新时使用同一应用标识 `com.niaolaiyunzhuan.desktop`。存档保存在 `%APPDATA%`，发布时保持该标识稳定。
