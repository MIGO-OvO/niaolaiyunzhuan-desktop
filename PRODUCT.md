# 袅来运转桌面版

<!-- impeccable:product-schema 1 -->

## Platform

web

界面运行于 Windows 的 Tauri WebView，交付物为 Windows 桌面应用。

## Stack

Rust + Tauri 2 + React + TypeScript，由用户指定并确认。

## Users

《燕云十六声》玩家，用于记录袅袅之音获取进度、查找遗漏来源。

## Product Purpose

将袅来运转网页版迁移为离线单机桌面工具，保留原有数据、查漏流程和备份兼容性。

## Operating Context

用户按区域、玩法、限时、概率来源查找目标，勾选已完成项，通过备份图片迁移记录。
进度存储在本机，首版不包含账号或云同步。

## Capabilities and Constraints

- 保留分类、搜索、完成状态筛选、分组子项、数量统计、新增提示、周期刷新与限时倒计时。
- Rust 管理记录、周期规则和备份校验；React 提供交互界面。
- 兼容原版 NYNZ1 / NYNZ2 二维码 PNG 备份，恢复采用合并语义。
- 保留旧 ID 与状态格式的迁移规则。进度写入失败必须向用户报告。
- 验收覆盖旧备份兼容、重启后记录保留、周期边界和 Windows 打包。
- 视觉采用用户确认的“玄墨江湖·宣纸暖金”配色与稿图；安装包签名与发布位置待确认。

## Brand Commitments

名称为“袅来运转”，保留“薄荷小鹿（燕云十六声）”原作者署名。
用户确认与原作者合作开发桌面版。

## Evidence on Hand

上游：https://github.com/MiaTheamateur/niaolaiyunzhuan-web

基线提交：`8a88ea6c8532f1f67d51660bbb1dd46850af9f18`，网页版 v1.5.0，数据修订 73。
该快照含 128 条来源、207 个独立勾选目标；过期条目保留在数据中，按规则隐藏。
上游包含第三方二维码组件的许可声明，未发现覆盖整个项目的独立 LICENSE 文件。
桌面版采用 [PolyForm Noncommercial 1.0.0](./LICENSE)。

## Product Principles

- 每次勾选都保存，退出和重启后可继续。
- 迁移和恢复保留已有进度。
- 完成项数与袅袅数量分别统计，概率来源不计入确定来源总量。
- 复用上游数据与兼容规则，桌面操作适配键盘和可调整大小的窗口。
