---
name: 袅来运转桌面版
description: 玄墨江湖·宣纸暖金
colors:
  ink: "#192321"
  paper: "#F6F1E8"
  panel: "#FFFCF6"
  gold: "#A98549"
  gold-text: "#75582C"
  jade: "#357763"
  text: "#25231F"
  muted: "#706C65"
  line: "#D9D0C1"
  danger: "#A13F32"
typography:
  display:
    fontFamily: "'Ma Shan Zheng', 'Noto Serif SC', 'Source Han Serif SC', 'SimSun', 'Songti SC', serif"
    fontSize: "48px"
    fontWeight: 400
    lineHeight: 1.12
    letterSpacing: "0.01em"
  headline:
    fontFamily: "'Noto Serif SC', 'Source Han Serif SC', 'SimSun', 'Songti SC', serif"
    fontSize: "24px"
    fontWeight: 600
    lineHeight: 1.5
  title:
    fontFamily: "'Noto Serif SC', 'Source Han Serif SC', 'SimSun', 'Songti SC', serif"
    fontSize: "17px"
    fontWeight: 600
    lineHeight: 1.6
  body:
    fontFamily: "'Microsoft YaHei UI', 'Microsoft YaHei', 'PingFang SC', sans-serif"
    fontSize: "14px"
  label:
    fontFamily: "'Microsoft YaHei UI', 'Microsoft YaHei', 'PingFang SC', sans-serif"
    fontSize: "11px"
rounded:
  check: "3px"
  small: "4px"
  tab: "5px"
  control: "6px"
  navigation: "7px"
  dialog: "10px"
spacing:
  small: "6px"
  compact: "10px"
  control: "16px"
  panel: "22px"
  section: "26px"
  heading: "30px"
components:
  button-primary:
    backgroundColor: "{colors.gold-text}"
    textColor: "{colors.panel}"
    rounded: "{rounded.control}"
    padding: "9px 16px"
    height: "40px"
  button-primary-hover:
    backgroundColor: "#634823"
  button-outline:
    backgroundColor: "#FFFCF6B3"
    textColor: "{colors.text}"
    rounded: "{rounded.control}"
    padding: "9px 16px"
    height: "40px"
  button-danger:
    backgroundColor: "{colors.danger}"
    textColor: "{colors.panel}"
    rounded: "{rounded.control}"
    padding: "9px 16px"
    height: "40px"
  button-text:
    textColor: "{colors.gold-text}"
    rounded: "{rounded.check}"
    padding: "5px 1px"
    height: "30px"
  button-icon:
    textColor: "#696356"
    rounded: "{rounded.small}"
    size: "32px"
  search-field:
    backgroundColor: "#FFFCF6B3"
    textColor: "{colors.text}"
    rounded: "{rounded.control}"
    padding: "0 13px"
    height: "42px"
  navigation-module:
    textColor: "{colors.panel}"
    rounded: "{rounded.navigation}"
    padding: "12px 18px"
    height: "54px"
  badge-new:
    backgroundColor: "#EAD9B3"
    textColor: "#785A2A"
    rounded: "{rounded.check}"
    padding: "0 3px"
  dialog:
    backgroundColor: "{colors.panel}"
    textColor: "{colors.text}"
    rounded: "{rounded.dialog}"
    padding: "26px 29px 28px"
    width: "min(480px, calc(100vw - 36px))"
  source-row-selected:
    backgroundColor: "#E9D5AE70"
    height: "65px"
  checkbox-complete:
    backgroundColor: "{colors.jade}"
    textColor: "{colors.panel}"
    rounded: "{rounded.check}"
    size: "21px"
---

# Design System: 袅来运转桌面版

## Overview

**Creative North Star: "玄墨江湖·宣纸暖金"**

以用户确认的配色和稿图为视觉依据。深墨导航承载分类与区域，宣纸色工作区同时呈现来源列表和详情。淡墨山水铺在工作面下层，暖金标出当前选择，青玉标出完成与保存状态。

界面面向 Windows 桌面查漏操作，保留连续列表、同列数字和清晰的勾选框。书法用于品牌与当前区域标题，操作文字保持紧凑、易读。

**Key Characteristics:**

- 深墨导航、暖白工作面、低对比山水底纹。
- 暖金选择与青玉完成各有固定语义。
- 书法标题、宋体任务名、雅黑操作文字。
- 三栏连续工作区，搜索、备份和窗口控制共用顶部栏。

## Colors

主色为沉稳的暖金，青玉提供完成反馈；中性色分别承担导航、纸面、正文和分隔线。

### Primary

- 暖金（`gold`）：导航边框、选择提示与焦点边框。
- 深金（`gold-text`）：浅底操作文字、主按钮、部分完成短横与键盘焦点。

### Secondary

- 青玉（`jade`）：已完成勾选、保存状态和当前备份标记。

### Tertiary

- 赭红（`danger`）：重置操作、错误状态与失败提示。

### Neutral

- 玄墨（`ink`）：左侧导航和深色底纹的底色。
- 宣纸（`paper`）：主工作区底色。
- 暖白（`panel`）：浮层、控件和深底文字。
- 墨字（`text`）：正文和任务名称。
- 灰墨（`muted`）：说明、表头和辅助计数。
- 纸线（`line`）：常驻区域分隔与列表细线。

**The 金选青成 Rule.** 当前分类、区域、筛选与来源行使用金色选择态；完成勾选和保存反馈使用青玉。分组部分完成使用深金短横，全部完成使用青玉对勾。

## Typography

标题字体 Ma Shan Zheng 随应用自托管。正文使用 Microsoft YaHei UI 字族，任务与区域文字使用宋体字族；具体字栈和字阶以 frontmatter 为准。

| 层级 | 用途 | 桌面缩放 |
| --- | --- | --- |
| Display | 当前区域或页面标题 | 常规字阶（48px），窗口宽度不超过 1300px 时收至 44px |
| Headline | 右侧来源标题 | 常规字阶（24px），不超过 1300px 时为 22px |
| Title | 任务与子目标名称 | 常规字阶（17px），不超过 1300px 时为 16px；列表在 1100px 内为 15px |
| Body | 操作控件与正文基础 | 基础字号（14px），说明段落按用途使用 12px 或 13px |
| Label | 表头、状态和辅助信息 | 紧凑字号（11px），状态栏为 10px |

品牌字样使用 Ma Shan Zheng（32px）。数量与进度采用等宽数字；任务名称允许换行，说明文字与名称分列。

**The 书法有位 Rule.** Ma Shan Zheng 用于品牌和当前页面标题，宋体承担任务内容，雅黑承担搜索、按钮、表头和状态。

## Layout

桌面默认窗口为 1440×960，最小为 1080×700。外壳由左侧导航、顶部工具栏、工作区和底部状态栏构成；工作区继续分为来源列表与右侧详情，形成三栏。

- 侧栏宽度为 `clamp(232px, 19vw, 288px)`，详情栏为 `clamp(304px, 26vw, 390px)`。窗口宽度不超过 1300px 时，两栏分别固定为 232px 和 306px。
- 顶部栏高（76px），底部状态栏高（42px）。来源列表、区域导航和详情各自纵向滚动。
- 列表行最小高度（65px），详情目标行最小高度（59px）。纸面以细线连续排列，列间保留稳定的对齐关系。
- 顶部搜索框、备份按钮和存档反馈沿一行排列。Windows 窗口按钮位于右上角，按钮区域与拖动区域分开。
- 宽度不超过 1100px 时隐藏存档状态文字和列表分组计数，保留状态图标与完整操作入口。
- 浏览器预览在 1000px 内切换为列表与详情单面显示，在 700px 内收为图标侧栏。Windows 桌面受最小窗口尺寸约束，使用三栏布局。

标题区横向留白约（30px），详情区留白（26px），控件之间通常相隔（6px 至 16px）。密度随窗口宽度收紧，交互目标和文字保持独立。

## Elevation & Depth

常驻三栏以底色、透明叠层和细线建立层次。山水作为背景进入侧栏和工作面，不随列表选择移动；浅色底纹的不透明度为（0.26）。

阴影只用于临时浮层：提示横条为 `0 4px 12px #25231F14`，通知为 `0 6px 20px #19232126`，对话框为 `0 18px 70px #19232140`。对话框遮罩使用 `#19232170`。

| 本地素材 | 实际尺寸 | 位置 |
| --- | --- | --- |
| `public/assets/ink-light.png` | 1672×941 | 浅色工作面山水 |
| `public/assets/ink-dark.png` | 887×1774 | 深色侧栏山水 |
| `public/assets/swallow-mark.png` | 1254×1254 | 品牌燕形标识 |

三张图片由 imagegen 生成，PNG 元数据约为 300 DPI。原始提示词保存在 `docs/asset-prompts/`。

## Shapes

控件采用小圆角矩形。勾选框与新增标记为（3px），图标按钮为（4px），筛选标签为（5px），搜索框和操作按钮为（6px），分类导航为（7px），对话框为（10px）。

当前分类保留内外双层细边框。进度条采用分段槽，填充保持在细框内部。状态点使用圆形；来源列表保持连续行。

## Components

### Buttons

主按钮使用深金底和暖白文字，描边按钮使用半透明暖白底，文字按钮以深金表达操作。悬停通过底色或边框变化反馈，基础过渡为（160ms ease-out）。禁用按钮降低不透明度，并以等待指针表示操作尚未完成。

### Inputs / Fields

搜索框由搜索图标、输入区和快捷键提示组成。聚焦时边框变为暖金，外侧增加淡金细环；清除按钮在输入有内容时出现。Ctrl K 将焦点移入搜索框。

### Navigation

分类使用 Lucide 线性图标和文字；当前分类以金色底、边框与内层细线标记。区域项同时显示名称、已获数量和分段进度。导航栏内的内容独立滚动，底部帮助与设置保持可见。

### Chips

新增目标使用小型“新”标记，暖金浅底配细边框。状态筛选使用带计数的标签按钮，当前筛选为金底暖白字。

### Cards / Containers

常驻列表与详情沿纸面连续排布。对话框使用暖白底、小圆角和柔和阴影；恢复预览的数量单独成组，底部操作靠右。错误信息在当前操作所在的横条或对话框内显示。

### Source rows / Checkboxes

来源行把完成框和详情入口分开。当前行以浅金横带与上下细线标记，鼠标悬停或选中时显示右侧箭头。完整目标使用青玉对勾，部分完成的分组使用深金短横。详情目标沿用同一套勾选样式。

### Window chrome / Feedback

标题栏与搜索、备份工具栏融合，Tauri 窗口关闭原生装饰。最小化、最大化或还原、关闭按钮调用真实窗口操作。失焦时顶部栏和品牌文字减弱；存档反馈位于顶部与底部，通知使用青玉深底。

勾选对勾以（180ms）轻微收定，读取骨架以（1.5s）呼吸，保存图标以（1s）旋转。系统启用减少动态效果时，关闭动画与过渡。

## Do's and Don'ts

### Do:

- Do 按“金选青成”区分选择、部分完成和全部完成。
- Do 让书法标题、宋体任务名和雅黑控件各守其位。
- Do 保留连续列表、细分隔线、数值对齐和独立滚动。
- Do 使用本地山水素材与 Lucide 线性图标，保持文字位于可读的浅色工作面。
- Do 为键盘焦点保留深金描边，为操作结果保留可读反馈。

### Don't:

- Don't 用暖金对勾表示全部完成。
- Don't 将 Ma Shan Zheng 扩展为搜索、按钮或长段正文的字体。
- Don't 用独立浮卡或投影拆开连续来源列表。
- Don't 将山水底纹作为任务文字、勾选框或数量信息的一部分。

