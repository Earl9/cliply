

# Cliply 开发任务文档

## 0. 项目目标

开发一个现代化剪贴板管理器，项目名为 **Cliply**。

Cliply 的定位是：

> 一个本地优先、键盘优先、隐私可控、UI 现代化的跨平台剪贴板工具。

第一阶段优先实现 **Windows MVP**，但代码结构必须为后续 macOS/Linux 扩展留好平台适配层。

核心参考：

- Ditto：可靠保存剪贴板历史、可检索、可复用。
- Windows Win+V：轻量浮层、快速打开、固定常用项。
- Maccy / CopyQ 类工具：键盘优先、搜索优先、支持多类型内容。
- 当前 UI 设计稿：现代 Fluent 风格双栏浮层，左侧历史列表，右侧详情预览。

------

## 1. 技术栈要求

### 1.1 桌面框架

使用：

```txt
Tauri v2 + Rust + React + TypeScript
```

前端建议：

```txt
Vite
React
TypeScript
Tailwind CSS
Radix UI 或 shadcn/ui 风格组件
Lucide React icons
```

后端建议：

```txt
Rust
Tauri commands
SQLite
SQLx 或 rusqlite
Windows Win32 API adapter
```

### 1.2 数据库

使用 SQLite。

必须支持：

```txt
普通结构化查询
全文搜索
标签 / 固定 / 删除 / 类型筛选
```

全文搜索优先使用 SQLite FTS5。

### 1.3 项目原则

- 不要复制 Ditto 源码。
- 不要引入 GPL 代码。
- 保持 local-first。
- 第一版不要做云同步。
- 第一版不要做账号系统。
- 第一版不要做插件系统。
- 所有敏感能力默认最小权限。
- UI 先做高质量本地桌面体验。

------

## 2. 第一阶段范围：Windows MVP

### 2.1 必须实现

第一阶段必须完成：

```txt
1. 系统托盘常驻
2. 全局快捷键打开 Cliply 面板
3. 监听 Windows 剪贴板变化
4. 保存文本、链接、代码、图片四类历史
5. 本地 SQLite 持久化
6. 搜索剪贴板历史
7. 类型筛选
8. 固定 / 取消固定
9. 删除单条记录
10. 清空历史
11. 选中历史后粘贴
12. 无格式粘贴
13. 暂停监听
14. 忽略重复内容
15. 基础隐私保护
16. 现代化 UI
```

### 2.2 暂不实现

第一阶段不要实现：

```txt
1. 云同步
2. 多设备同步
3. 账号登录
4. OCR
5. AI 总结
6. 插件市场
7. 团队共享
8. 移动端
9. 浏览器扩展
10. 复杂脚本自动化
```

这些功能可以预留接口，但不要实现。

------

## 3. 核心用户体验

### 3.1 快捷键

默认快捷键：

```txt
Ctrl + Shift + V
```

打开 Cliply 浮层。

浮层内快捷键：

```txt
Enter：粘贴当前选中项
Shift + Enter：无格式粘贴
Ctrl + C：复制当前选中项到系统剪贴板，但不自动粘贴
Ctrl + P：固定 / 取消固定
Delete：删除当前选中项
Esc：关闭浮层
ArrowUp / ArrowDown：切换选中项
Tab：切换到详情面板或操作按钮区
Ctrl + K：聚焦搜索框
```

### 3.2 主流程

用户复制内容后：

```txt
1. Cliply 后台监听到剪贴板变化
2. 读取剪贴板内容
3. 判断内容类型
4. 计算 hash
5. 如果重复，更新 last_copied_at 和 used_count，不新增
6. 如果疑似敏感内容，按隐私策略处理
7. 保存到 SQLite
8. 更新前端列表
```

用户打开 Cliply 后：

```txt
1. 显示最近剪贴板历史
2. 默认选中第一条
3. 用户可直接输入搜索
4. 用户按 Enter 粘贴
5. Cliply 将该条内容写回系统剪贴板
6. 模拟 Ctrl+V 粘贴到之前的焦点窗口
7. 关闭浮层
```

------

## 4. UI 设计要求

### 4.1 总体视觉

UI 要基于当前设计稿实现。

风格关键词：

```txt
现代
轻量
Fluent-like
半透明
圆角
卡片式
键盘优先
信息密度适中
专业生产力工具
```

不要做成传统 Win32 工具窗口。

### 4.2 主窗口布局

窗口尺寸建议：

```txt
宽度：980px - 1160px
高度：680px - 760px
圆角：16px - 20px
```

结构：

```txt
顶部标题栏
搜索栏
类型筛选区
主内容双栏
底部快捷键提示栏
```

布局示意：

```txt
┌──────────────────────────────────────────────┐
│ Cliply                               ☆ ⚙ ⋯ × │
├──────────────────────────────────────────────┤
│ 🔍 搜索剪贴板、标签、应用…             Ctrl+K │
├──────────────────────────────────────────────┤
│ 全部  文本  链接  图片  代码  固定           │
├───────────────────┬──────────────────────────┤
│ 历史列表           │ 详情预览                  │
│                   │                          │
│ [selected item]   │ 代码 / 图片 / 链接预览     │
│ [item]            │ 元数据                    │
│ [item]            │ 操作按钮                  │
│ [item]            │                          │
├───────────────────┴──────────────────────────┤
│ Enter 粘贴  Shift+Enter 无格式  Ctrl+P 固定 │
└──────────────────────────────────────────────┘
```

------

## 5. UI 组件拆分

前端组件建议拆成以下文件。

```txt
src/
  app/
    App.tsx
    routes.tsx

  components/
    shell/
      AppWindow.tsx
      TitleBar.tsx
      FooterShortcuts.tsx

    clipboard/
      ClipboardSearchBar.tsx
      ClipboardFilterTabs.tsx
      ClipboardList.tsx
      ClipboardListItem.tsx
      ClipboardDetailPane.tsx
      ClipboardPreview.tsx
      ClipboardMetadata.tsx
      ClipboardActions.tsx
      EmptyState.tsx

    common/
      IconButton.tsx
      ShortcutKey.tsx
      PillTabs.tsx
      Card.tsx
      Badge.tsx
      ScrollArea.tsx

  stores/
    clipboardStore.ts
    settingsStore.ts
    uiStore.ts

  lib/
    clipboardTypes.ts
    formatTime.ts
    detectContentType.ts
    keyboard.ts
```

------

## 6. 前端任务拆解

### 6.1 初始化 UI 项目

任务：

```txt
1. 创建 React + TypeScript + Vite 项目
2. 接入 Tailwind CSS
3. 配置路径别名 @/*
4. 创建基础 layout
5. 创建全局 CSS 变量
6. 支持 light theme
7. 预留 dark theme token
```

验收标准：

```txt
1. npm run dev 可启动
2. 页面显示 Cliply 主窗口
3. 样式接近设计稿
4. 无 TypeScript 错误
```

------

### 6.2 实现主窗口 AppWindow

任务：

```txt
1. 实现居中的圆角浮层窗口
2. 背景使用柔和浅色渐变
3. 窗口使用白色 / 半透明效果
4. 添加阴影
5. 添加顶部标题栏
6. 添加窗口右上角按钮：固定、设置、更多、关闭
```

验收标准：

```txt
1. UI 有现代桌面应用质感
2. 标题栏左侧显示 Cliply logo 和名称
3. 右侧图标按钮 hover 状态正常
4. 关闭按钮调用 Tauri window hide，而不是退出应用
```

------

### 6.3 实现搜索栏

任务：

```txt
1. 创建 ClipboardSearchBar
2. 显示搜索 icon
3. placeholder 使用：搜索剪贴板、标签、应用…
4. 右侧显示 Ctrl + K 快捷键提示
5. 输入时更新 query state
6. 防抖 150ms
```

验收标准：

```txt
1. 输入关键词后列表过滤
2. Ctrl+K 可聚焦搜索框
3. Esc 在搜索框为空时关闭窗口
4. Esc 在搜索框不为空时先清空搜索
```

------

### 6.4 实现类型筛选 Tabs

任务：

```txt
1. 创建 ClipboardFilterTabs
2. tabs 包含：全部、文本、链接、图片、代码、固定
3. 当前 tab 高亮
4. 点击 tab 更新 filter state
```

验收标准：

```txt
1. tab 切换后列表立即过滤
2. 固定 tab 只显示 pinned items
3. 全部 tab 显示所有未删除 items
```

------

### 6.5 实现历史列表

任务：

```txt
1. 创建 ClipboardList
2. 创建 ClipboardListItem
3. 支持 selected 状态
4. 支持 hover 状态
5. 支持 source app icon
6. 支持 content type label
7. 支持 preview text
8. 支持 copied time
9. 支持 pin icon
10. 支持虚拟滚动预留
```

列表项示例数据：

```ts
[
  {
    type: "code",
    sourceApp: "Visual Studio Code",
    preview: "const user = await getProfile()",
    createdAt: "10:42:18",
    relativeTime: "刚刚",
    pinned: true
  },
  {
    type: "link",
    sourceApp: "Chrome",
    preview: "https://github.com/sabrogden/Ditto",
    relativeTime: "1 分钟前",
    pinned: false
  }
]
```

验收标准：

```txt
1. 选中项有明显紫蓝色描边
2. 上下键可以改变选中项
3. 鼠标点击可以选中项
4. pin icon 点击可固定 / 取消固定
5. 列表底部显示：共 N 条记录
```

------

### 6.6 实现详情预览面板

任务：

```txt
1. 创建 ClipboardDetailPane
2. 根据选中项类型渲染不同预览
3. text 类型显示纯文本
4. code 类型显示代码块
5. link 类型显示 URL 卡片
6. image 类型显示图片缩略图
7. 显示 metadata
8. 显示操作按钮
```

metadata 字段：

```txt
来源应用
复制时间
类型
大小
是否固定
```

验收标准：

```txt
1. 选中不同 item 时右侧详情变化
2. 代码预览有行号
3. 图片预览不拉伸变形
4. URL 显示域名和完整链接
5. metadata 信息布局整齐
```

------

### 6.7 实现操作按钮

任务：

```txt
1. 粘贴按钮
2. 复制按钮
3. 无格式粘贴按钮
4. 固定按钮
```

按钮行为：

```txt
粘贴：调用 paste_clipboard_item
复制：调用 copy_clipboard_item
无格式粘贴：调用 paste_plain_text
固定：调用 toggle_pin
```

验收标准：

```txt
1. 主按钮使用紫蓝色强调
2. 每个按钮显示快捷键
3. 点击后调用对应 Tauri command
4. 操作成功后有轻量 toast 或状态反馈
```

------

### 6.8 实现底部快捷键提示栏

任务：

```txt
1. 创建 FooterShortcuts
2. 显示 Enter 粘贴
3. 显示 Shift+Enter 无格式
4. 显示 Ctrl+P 固定
5. 显示 Esc 关闭
6. 右侧显示 本地保存 / 同步未启用 状态
```

注意：

第一版不要真的做同步，所以 UI 文案应为：

```txt
本地保存
```

不要显示：

```txt
同步已启用
```

验收标准：

```txt
1. 底部快捷键样式清晰
2. 与设计稿类似但不误导用户
3. 窗口尺寸变小时不溢出
```

------

## 7. 后端任务拆解

### 7.1 初始化 Tauri

任务：

```txt
1. 接入 Tauri v2
2. 配置窗口为隐藏启动或托盘启动
3. 配置主窗口尺寸
4. 配置窗口透明 / 阴影 / 圆角能力
5. 配置 dev 和 build 脚本
```

验收标准：

```txt
1. npm run tauri dev 可启动
2. 桌面窗口显示 Cliply UI
3. 关闭窗口时隐藏到托盘，不退出进程
4. 托盘菜单包含：打开 Cliply、暂停监听、退出
```

------

### 7.2 配置 Tauri 权限

任务：

```txt
1. 添加 clipboard-manager plugin
2. 添加 global-shortcut plugin
3. 在 capabilities 中开启必要权限
4. 不要开启不必要权限
```

需要的权限方向：

```txt
clipboard read text
clipboard write text
clipboard read image
clipboard write image
global shortcut register
global shortcut unregister
```

验收标准：

```txt
1. 应用可以读写文本剪贴板
2. 应用可以注册 Ctrl+Shift+V
3. 权限配置最小化
4. 构建无权限报错
```

------

### 7.3 实现全局快捷键

任务：

```txt
1. 注册 Ctrl+Shift+V
2. 快捷键触发时显示主窗口
3. 显示窗口前记录当前前台窗口句柄
4. 主窗口获得焦点
5. Esc 或粘贴后隐藏窗口
```

验收标准：

```txt
1. 任意应用中按 Ctrl+Shift+V 可以打开 Cliply
2. 重复按快捷键可以显示 / 隐藏
3. 不影响系统 Win+V
4. 应用退出时注销快捷键
```

------

### 7.4 实现 Windows 剪贴板监听

任务：

```txt
1. 新建 platform/windows_clipboard_listener.rs
2. 使用 AddClipboardFormatListener
3. 接收 WM_CLIPBOARDUPDATE
4. 变化后触发内部事件 clipboard_changed
5. 防抖处理，避免连续重复触发
6. 应用退出时 RemoveClipboardFormatListener
```

验收标准：

```txt
1. 复制文本后数据库新增记录
2. 复制链接后识别为 link
3. 复制图片后数据库新增 image 记录
4. Cliply 自己写回剪贴板时不要重复保存
5. 连续复制同一内容不要刷屏
```

------

### 7.5 实现剪贴板读取服务

任务：

```txt
1. 创建 clipboard_service.rs
2. 实现 read_current_clipboard()
3. 优先读取多格式内容
4. 文本内容读取 text/plain
5. HTML 内容读取 text/html，若可行
6. 图片内容读取 bitmap/png
7. 文件列表后续预留，第一版可不实现
```

返回结构：

```rust
pub struct ClipboardSnapshot {
    pub primary_type: ClipboardItemType,
    pub text: Option<String>,
    pub html: Option<String>,
    pub image: Option<ImageSnapshot>,
    pub formats: Vec<ClipboardFormatSnapshot>,
    pub source_app: Option<String>,
    pub source_window: Option<String>,
}
```

验收标准：

```txt
1. 复制普通文本可以读取 text
2. 复制 URL 可以读取 text，并识别 link
3. 复制代码可以读取 text，并识别 code
4. 复制图片可以读取 image
5. 空剪贴板或读取失败时不崩溃
```

------

### 7.6 实现内容类型识别

任务：

```txt
1. 创建 content_detector.rs
2. 实现 detect_clipboard_type()
3. 支持 text/link/code/image/html/mixed
```

识别规则：

```txt
如果有 image 且无 text：image
如果 text 是 URL：http/https/mailto/file -> link
如果 text 命中代码特征：code
如果有 html 且 text 存在：html
否则：text
```

代码识别特征：

```txt
包含 import/export/function/const/let/class/interface
包含多行缩进
包含 {} 或 => 或 ;
包含 SQL 关键词 SELECT/FROM/WHERE
包含 JSON-like 结构
```

验收标准：

```txt
1. URL 被识别为 link
2. JavaScript 片段被识别为 code
3. 普通聊天内容被识别为 text
4. 图片被识别为 image
```

------

### 7.7 实现 hash 去重

任务：

```txt
1. 创建 hash_service.rs
2. 文本使用 normalized text hash
3. 图片使用 bytes hash
4. HTML 使用 stripped fallback + raw hash
5. 保存前检查是否已存在 hash
```

重复处理规则：

```txt
如果 hash 已存在：
- 不新增记录
- 更新 copied_at
- 更新 last_seen_at
- count += 1
- 如果原记录已删除，不自动恢复
```

验收标准：

```txt
1. 连续复制同一文本只保留一条
2. 再次复制历史内容时该条移动到顶部
3. pinned 状态不被覆盖
```

------

## 8. 数据库设计

### 8.1 表：clipboard_items

```sql
CREATE TABLE clipboard_items (
  id TEXT PRIMARY KEY,
  type TEXT NOT NULL,
  title TEXT,
  preview_text TEXT,
  normalized_text TEXT,
  source_app TEXT,
  source_window TEXT,
  hash TEXT NOT NULL,
  size_bytes INTEGER DEFAULT 0,
  is_pinned INTEGER DEFAULT 0,
  is_favorite INTEGER DEFAULT 0,
  is_deleted INTEGER DEFAULT 0,
  sensitive_score INTEGER DEFAULT 0,
  copied_at TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  used_count INTEGER DEFAULT 0
);
```

索引：

```sql
CREATE INDEX idx_clipboard_items_hash ON clipboard_items(hash);
CREATE INDEX idx_clipboard_items_type ON clipboard_items(type);
CREATE INDEX idx_clipboard_items_copied_at ON clipboard_items(copied_at);
CREATE INDEX idx_clipboard_items_pinned ON clipboard_items(is_pinned);
```

------

### 8.2 表：clipboard_formats

```sql
CREATE TABLE clipboard_formats (
  id TEXT PRIMARY KEY,
  item_id TEXT NOT NULL,
  format_name TEXT NOT NULL,
  mime_type TEXT,
  data_kind TEXT NOT NULL,
  data_text TEXT,
  data_path TEXT,
  size_bytes INTEGER DEFAULT 0,
  priority INTEGER DEFAULT 0,
  created_at TEXT NOT NULL,
  FOREIGN KEY (item_id) REFERENCES clipboard_items(id)
);
```

`data_kind` 可选值：

```txt
text
html
image_file
binary_file
external_ref
```

------

### 8.3 表：clipboard_tags

```sql
CREATE TABLE clipboard_tags (
  item_id TEXT NOT NULL,
  tag TEXT NOT NULL,
  created_at TEXT NOT NULL,
  PRIMARY KEY (item_id, tag),
  FOREIGN KEY (item_id) REFERENCES clipboard_items(id)
);
```

------

### 8.4 表：settings

```sql
CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

默认设置：

```json
{
  "max_history_items": 1000,
  "auto_delete_days": 30,
  "pause_monitoring": false,
  "ignore_duplicate": true,
  "save_images": true,
  "save_html": true,
  "save_sensitive": false,
  "global_shortcut": "Ctrl+Shift+V",
  "theme": "light"
}
```

------

### 8.5 FTS 表

```sql
CREATE VIRTUAL TABLE clipboard_items_fts USING fts5(
  item_id UNINDEXED,
  title,
  preview_text,
  normalized_text,
  source_app
);
```

任务：

```txt
1. 新增 clipboard item 时同步写入 FTS
2. 更新 item 时同步更新 FTS
3. 删除 item 时同步删除 FTS
```

验收标准：

```txt
1. 搜索正文可命中
2. 搜索 source app 可命中
3. 搜索 URL 可命中
4. 搜索性能在 1000 条以内无明显卡顿
```

------

## 9. Tauri Commands 设计

实现以下 commands。

### 9.1 获取列表

```rust
#[tauri::command]
async fn list_clipboard_items(
    query: Option<String>,
    item_type: Option<String>,
    pinned_only: Option<bool>,
    limit: Option<i64>,
    offset: Option<i64>
) -> Result<Vec<ClipboardItemDto>, String>
```

验收标准：

```txt
1. 默认返回最近 50 条
2. 支持 query 搜索
3. 支持 type 过滤
4. 支持 pinned_only
5. 按 is_pinned desc, copied_at desc 排序
```

------

### 9.2 获取详情

```rust
#[tauri::command]
async fn get_clipboard_item_detail(
    id: String
) -> Result<ClipboardItemDetailDto, String>
```

验收标准：

```txt
1. 返回 item 基本信息
2. 返回 formats
3. 图片返回 thumbnail path
4. 文本返回完整 text
```

------

### 9.3 粘贴

```rust
#[tauri::command]
async fn paste_clipboard_item(
    id: String
) -> Result<(), String>
```

逻辑：

```txt
1. 根据 id 获取 formats
2. 写回系统剪贴板
3. 恢复之前的前台窗口
4. 模拟 Ctrl+V
5. used_count += 1
6. last_used_at 更新
7. 隐藏 Cliply 窗口
```

验收标准：

```txt
1. 在记事本中可粘贴文本
2. 在浏览器地址栏可粘贴 URL
3. 在 VS Code 中可粘贴代码
4. 粘贴后 Cliply 自动隐藏
```

------

### 9.4 无格式粘贴

```rust
#[tauri::command]
async fn paste_plain_text(
    id: String
) -> Result<(), String>
```

逻辑：

```txt
1. 获取 normalized text 或 preview text
2. 只写 text/plain 到系统剪贴板
3. 模拟 Ctrl+V
```

验收标准：

```txt
1. HTML 内容以纯文本粘贴
2. 代码内容不带格式粘贴
3. 图片类型如果无 text fallback，按钮 disabled
```

------

### 9.5 复制到剪贴板

```rust
#[tauri::command]
async fn copy_clipboard_item(
    id: String
) -> Result<(), String>
```

验收标准：

```txt
1. 将 item 写回系统剪贴板
2. 不模拟 Ctrl+V
3. 不关闭窗口
4. 不重复新增历史
```

------

### 9.6 固定

```rust
#[tauri::command]
async fn toggle_pin_clipboard_item(
    id: String
) -> Result<ClipboardItemDto, String>
```

验收标准：

```txt
1. pinned 状态切换
2. UI 立即更新
3. 固定项排序靠前
4. 固定项不被自动清理
```

------

### 9.7 删除

```rust
#[tauri::command]
async fn delete_clipboard_item(
    id: String
) -> Result<(), String>
```

采用软删除：

```txt
is_deleted = 1
```

验收标准：

```txt
1. 删除后列表消失
2. FTS 不再命中
3. 文件资源可延迟清理
```

------

### 9.8 清空历史

```rust
#[tauri::command]
async fn clear_clipboard_history(
    include_pinned: bool
) -> Result<(), String>
```

逻辑：

```txt
include_pinned = false 时保留 pinned items
include_pinned = true 时全部清空
```

验收标准：

```txt
1. 默认清空不删除固定项
2. 二次确认后可清空固定项
```

------

### 9.9 暂停监听

```rust
#[tauri::command]
async fn set_monitoring_paused(
    paused: bool
) -> Result<(), String>
```

验收标准：

```txt
1. 暂停后复制内容不入库
2. 恢复后继续监听
3. 托盘状态同步显示
```

------

## 10. 隐私保护任务

### 10.1 敏感内容识别

创建 `sensitive_detector.rs`。

规则：

```txt
疑似密码
疑似验证码
疑似 API key
疑似 token
疑似私钥
疑似银行卡号
疑似身份证号
疑似 seed phrase
```

基础正则：

```txt
长度 6 位纯数字，且来源不是普通文本时 -> 可能验证码
包含 sk- / token / api_key / secret / password -> 高风险
包含 -----BEGIN PRIVATE KEY----- -> 高风险
连续 12-24 个助记词 -> 高风险
```

处理策略：

```txt
默认不保存高风险内容
中风险内容保存 preview，但隐藏详情
用户可在设置中修改
```

验收标准：

```txt
1. 复制 private key 不入库
2. 复制 sk- 开头 token 不入库
3. 普通 URL 不误判
4. 普通聊天文本不误判
```

------

### 10.2 忽略应用

第一版实现配置结构，UI 可以晚一点做。

默认忽略应用：

```txt
1Password
Bitwarden
KeePass
KeePassXC
Windows Credential Manager
银行类应用，先预留
```

任务：

```txt
1. 获取 source_app
2. 如果 source_app 在 ignore list，不保存
3. 设置表中保存 ignore_apps
```

验收标准：

```txt
1. 命中 ignore app 时不保存
2. 日志记录为 ignored，但不记录内容本身
```

------

## 11. 文件与图片存储

### 11.1 本地目录

使用 app data 目录：

```txt
Cliply/
  cliply.db
  blobs/
    images/
    thumbnails/
  logs/
```

### 11.2 图片处理

任务：

```txt
1. 图片原图保存到 blobs/images
2. 生成缩略图到 blobs/thumbnails
3. DB 保存路径
4. UI 使用缩略图
```

验收标准：

```txt
1. 大图片不直接塞进 SQLite
2. 缩略图加载快
3. 原图丢失时 UI 不崩溃
```

------

## 12. 状态管理

前端 store 建议：

```ts
type ClipboardState = {
  items: ClipboardItem[]
  selectedId: string | null
  query: string
  filter: "all" | "text" | "link" | "image" | "code" | "pinned"
  loading: boolean
  detail: ClipboardItemDetail | null
}
```

actions：

```ts
loadItems()
selectItem(id)
setQuery(query)
setFilter(filter)
loadDetail(id)
pasteSelected()
copySelected()
pastePlainSelected()
togglePinSelected()
deleteSelected()
```

验收标准：

```txt
1. UI 状态和后端数据一致
2. 搜索和筛选不会互相覆盖
3. 删除当前项后自动选择下一项
4. 空列表显示 EmptyState
```

------

## 13. 空状态与异常状态

### 13.1 空剪贴板

文案：

```txt
还没有剪贴板记录
复制一段文字、链接或图片后，它会出现在这里。
```

### 13.2 搜索无结果

文案：

```txt
没有找到匹配内容
试试换个关键词，或者切换到“全部”。
```

### 13.3 暂停监听

顶部或底部显示：

```txt
监听已暂停
```

并提供：

```txt
恢复监听
```

按钮。

------

## 14. 设置页，第一版简化

第一版设置页可以简单做成弹窗。

设置项：

```txt
1. 全局快捷键
2. 最大历史条数
3. 自动清理天数
4. 是否保存图片
5. 是否保存 HTML
6. 是否启用敏感内容过滤
7. 忽略应用列表
8. 清空历史
9. 导出数据库，预留
```

验收标准：

```txt
1. 设置可保存到 SQLite
2. 重启后设置仍然生效
3. 修改快捷键后立即重新注册
```

------

## 15. 代码结构建议

```txt
cliply/
  src/
    app/
    components/
    stores/
    lib/
    styles/

  src-tauri/
    src/
      main.rs
      commands/
        mod.rs
        clipboard_commands.rs
        settings_commands.rs
      services/
        mod.rs
        clipboard_service.rs
        database_service.rs
        search_service.rs
        hash_service.rs
        sensitive_detector.rs
        content_detector.rs
        paste_service.rs
        settings_service.rs
      platform/
        mod.rs
        windows/
          mod.rs
          clipboard_listener.rs
          foreground_window.rs
          paste_simulator.rs
        macos/
          mod.rs
        linux/
          mod.rs
      models/
        mod.rs
        clipboard_item.rs
        settings.rs
      db/
        migrations/
          001_init.sql
          002_fts.sql
      tray.rs
      shortcuts.rs
```

------

## 16. 平台抽象

定义 trait：

```rust
pub trait ClipboardPlatform {
    fn start_listening(&self) -> Result<(), CliplyError>;
    fn stop_listening(&self) -> Result<(), CliplyError>;
    fn read_clipboard(&self) -> Result<ClipboardSnapshot, CliplyError>;
    fn write_clipboard(&self, item: ClipboardWritePayload) -> Result<(), CliplyError>;
    fn paste_to_foreground(&self) -> Result<(), CliplyError>;
    fn get_foreground_app(&self) -> Result<Option<ForegroundAppInfo>, CliplyError>;
}
```

第一版实现：

```txt
WindowsClipboardPlatform
```

预留：

```txt
MacOSClipboardPlatform
LinuxClipboardPlatform
```

验收标准：

```txt
1. commands 不直接依赖 Win32 API
2. Windows 代码集中在 platform/windows
3. 后续可替换 macOS/Linux 实现
```

------

## 17. 质量要求

### 17.1 性能

目标：

```txt
打开浮层 < 150ms
搜索响应 < 100ms
复制入库 < 300ms
1000 条记录列表不卡顿
图片缩略图异步加载
```

### 17.2 稳定性

要求：

```txt
剪贴板读取失败不能崩溃
数据库写入失败要记录错误
图片保存失败时仍可保存文本 fallback
粘贴失败要给用户提示
快捷键冲突要提示
```

### 17.3 日志

日志不要保存剪贴板正文。

可以记录：

```txt
事件类型
错误类型
item id
content type
source app
耗时
```

不能记录：

```txt
完整复制内容
token
密码
图片原始内容
```

------

## 18. 开发顺序

请按以下顺序实现，不要跳步。

### Phase 1：项目骨架

```txt
1. 初始化 Tauri + React + TypeScript
2. 接入 Tailwind
3. 创建主窗口 UI 静态版
4. 创建 SQLite 初始化逻辑
5. 创建基础 commands
```

完成标准：

```txt
可以打开 Cliply 窗口，看到完整静态 UI。
```

------

### Phase 2：数据模型与 Mock UI

```txt
1. 实现 ClipboardItem 类型
2. 前端使用 mock 数据渲染列表
3. 实现搜索、筛选、选中、详情
4. 实现键盘导航
```

完成标准：

```txt
不接系统剪贴板，也可以完整操作 UI。
```

------

### Phase 3：剪贴板入库

```txt
1. 实现 Windows 剪贴板监听
2. 实现文本读取
3. 实现链接 / 代码类型识别
4. 实现 SQLite 保存
5. 实现前端读取真实数据
```

完成标准：

```txt
复制文本、URL、代码后，Cliply 列表自动出现记录。
```

------

### Phase 4：搜索与固定

```txt
1. 实现 FTS5 搜索
2. 实现类型筛选
3. 实现固定 / 取消固定
4. 实现删除
5. 实现清空历史
```

完成标准：

```txt
剪贴板历史可以正常管理。
```

------

### Phase 5：粘贴能力

```txt
1. 实现写回剪贴板
2. 实现复制按钮
3. 实现粘贴按钮
4. 实现无格式粘贴
5. 实现粘贴后隐藏窗口
```

完成标准：

```txt
在 Notepad、VS Code、Chrome 输入框中可正常粘贴。
```

------

### Phase 6：图片支持

```txt
1. 读取图片剪贴板
2. 保存图片文件
3. 生成缩略图
4. UI 显示图片卡片
5. 详情页显示图片预览
```

完成标准：

```txt
截图后 Cliply 能显示图片记录和缩略图。
```

------

### Phase 7：托盘与设置

```txt
1. 实现系统托盘
2. 实现打开 / 暂停监听 / 退出
3. 实现设置页
4. 设置持久化
5. 快捷键可修改
```

完成标准：

```txt
Cliply 可以作为常驻工具使用。
```

------

### Phase 8：隐私与打磨

```txt
1. 实现敏感内容过滤
2. 实现忽略应用
3. 优化窗口动画
4. 优化空状态
5. 优化错误提示
6. 增加基础测试
```

完成标准：

```txt
达到可日常自用的 MVP 质量。
```

------

## 19. 测试清单

### 19.1 文本测试

```txt
复制普通中文文本
复制英文文本
复制长文本
复制空格文本
复制重复文本
复制多行文本
```

### 19.2 链接测试

```txt
复制 https URL
复制 http URL
复制 GitHub URL
复制邮箱地址
复制带 query 参数的 URL
```

### 19.3 代码测试

```txt
复制 JavaScript
复制 TypeScript
复制 JSON
复制 SQL
复制 Rust
```

### 19.4 图片测试

```txt
截图后复制
从浏览器复制图片
从画图复制图片
复制大图片
复制图片后删除本地 blob 文件，UI 不崩溃
```

### 19.5 粘贴测试

```txt
粘贴到 Notepad
粘贴到 VS Code
粘贴到 Chrome 地址栏
粘贴到微信输入框
粘贴到 Word
无格式粘贴到 Word
```

### 19.6 隐私测试

```txt
复制疑似密码
复制 6 位验证码
复制 API token
复制 private key
复制普通聊天文本，确保不误伤
```

------

## 20. 第一版验收标准

MVP 完成时，Cliply 应满足：

```txt
1. Windows 下可以常驻运行
2. Ctrl+Shift+V 可以打开主窗口
3. 复制文本、链接、代码、图片后可以保存
4. 可以搜索历史
5. 可以按类型筛选
6. 可以固定记录
7. 可以删除记录
8. 可以粘贴历史记录
9. 可以无格式粘贴
10. UI 接近设计稿
11. 数据保存在本地 SQLite
12. 不保存明显敏感内容
13. 不复制 Ditto 源码
14. 无明显崩溃
```

