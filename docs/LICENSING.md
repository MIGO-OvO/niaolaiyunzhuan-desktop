# 来源与许可

原项目为 [MiaTheamateur/niaolaiyunzhuan-web](https://github.com/MiaTheamateur/niaolaiyunzhuan-web)，原作者为薄荷小鹿（燕云十六声）。桌面版保留其名称、作者署名和数据来源。

桌面版采用 [PolyForm Noncommercial 1.0.0](../LICENSE)，SPDX 标识为 `PolyForm-Noncommercial-1.0.0`。许可全文来自 [PolyForm 官方文本](https://polyformproject.org/licenses/noncommercial/1.0.0.txt)。

该协议允许其规定的非商业使用、修改和分发。商业用途需另行获得权利人的授权。分发副本时，须附上许可全文或官方链接，并保留随软件提供的 `Required Notice:` 声明。

截至桌面版 1.5.0 发布时，上游没有声明覆盖整个项目的许可证。本仓库的许可声明适用于桌面版；上游仓库的许可由原作者声明。

上游的 `THIRD_PARTY_LICENSES.js` 已原样保留，其中的 MIT 和 Apache-2.0 文本分别适用于 qrcode-generator、jsQR 等第三方组件。桌面版的二维码编解码由 Rust 依赖实现；这些上游 JavaScript 组件未作为桌面版运行库使用。

字体为马善政，遵循 SIL Open Font License 1.1，完整许可见 `public/fonts/OFL-MaShanZheng.txt`。

桌面版使用的 React、Tauri、Lucide 及 Rust 依赖分别遵循各自许可。构建所用包的许可声明与可用许可文本汇总于 `public/licenses/THIRD_PARTY_NOTICES.txt`，同样嵌入安装包。`scripts/collect-licenses.mjs` 根据锁定依赖重新生成汇总。

底纹和飞燕标识由 imagegen 生成，提示词保存在 `docs/asset-prompts`，图像密度元数据约为 300 DPI。

数据快照来自上游提交 `8a88ea6c8532f1f67d51660bbb1dd46850af9f18`，对应 v1.5.0 修订 73。
