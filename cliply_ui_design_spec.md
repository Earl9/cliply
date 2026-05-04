# Cliply UI 设计文档

## 1. 文档目的

本文档用于指导 Cliply 的产品 UI、前端组件、交互状态和视觉实现。目标是让设计、前端、Rust/Tauri 桌面能力实现都围绕同一套界面规范推进，避免只凭截图临摹导致交互细节缺失。

Cliply 是一个现代化剪贴板管理器，第一版目标平台为 Windows 11，后续预留 macOS 和 Linux。UI 应优先服务于高频操作：快速打开、搜索、选择、粘贴、固定、删除、无格式粘贴。

---

## 2. UI 总体定位

### 2.1 产品气质

Cliply 的 UI 关键词：

- 现代桌面工具
- 键盘优先
- 本地优先
- 轻量但不简陋
- 专业但不冷冰冰
- 信息清晰
- 低干扰
- 快速确认与快速执行

### 2.2 参考方向

Cliply 的 UI 不是传统 Win32 工具窗口，也不是网页后台管理系统。它更接近一个现代系统级浮层工具：

- 类似 Windows 11 的轻浮层质感
- 类似命令面板的快速搜索体验
- 类似现代文件/邮件列表的双栏信息结构
- 类似 Win+V 的低学习成本
- 比 Ditto 更现代、更可视化

### 2.3 核心 UI 原则

1. **先搜索，再操作**  
   用户打开 Cliply 后，搜索框应天然处于可输入状态。

2. **主操作永远明显**  
   当前选中项的“粘贴”按钮必须是最显眼的按钮。

3. **键盘操作不低于鼠标操作**  
   所有高频操作都应支持快捷键。

4. **列表负责速度，详情负责确认**  
   左侧列表用于快速定位，右侧详情用于确认内容与执行操作。

5. **隐私状态必须可见**  
   暂停监听、本地保存、敏感内容过滤等状态应在 UI 中有清晰反馈。

---

## 3. 信息架构

Cliply 主界面分为 5 个区域：

```txt
┌──────────────────────────────────────────────┐
│ 1. 顶部标题栏                                 │
├──────────────────────────────────────────────┤
│ 2. 搜索区                                     │
├──────────────────────────────────────────────┤
│ 3. 类型筛选区                                 │
├───────────────────┬──────────────────────────┤
│ 4A. 剪贴板历史列表 │ 4B. 内容详情与操作区       │
├───────────────────┴──────────────────────────┤
│ 5. 底部快捷键与状态栏                         │
└──────────────────────────────────────────────┘
```

### 3.1 顶部标题栏

职责：

- 展示品牌名称 Cliply
- 提供窗口级操作入口
- 提供设置、更多菜单、置顶窗口等入口

包含元素：

- Logo
- App 名称：Cliply
- 置顶按钮
- 设置按钮
- 更多菜单按钮
- 关闭按钮

### 3.2 搜索区

职责：

- 全文搜索剪贴板内容
- 搜索来源应用
- 搜索标签
- 搜索链接、代码片段、文件名

搜索框 placeholder：

```txt
搜索剪贴板、标签、应用…
```

右侧快捷键提示：

```txt
Ctrl + K
```

### 3.3 类型筛选区

默认筛选项：

```txt
全部 / 文本 / 链接 / 图片 / 代码 / 固定
```

后续可扩展：

```txt
文件 / 邮箱 / 颜色 / 收藏 / 今天
```

第一版不要放太多筛选项，避免主界面显得臃肿。

### 3.4 剪贴板历史列表

职责：

- 显示最近剪贴板记录
- 支持搜索结果展示
- 支持键盘上下选择
- 支持固定、删除、右键菜单
- 显示来源应用、类型、时间、预览内容

列表默认排序：

```txt
固定项优先，其余按 copied_at 倒序
```

建议排序逻辑：

```txt
is_pinned DESC
copied_at DESC
```

### 3.5 内容详情与操作区

职责：

- 展示当前选中剪贴板项的完整预览
- 展示来源、类型、大小、复制时间等信息
- 提供粘贴、复制、无格式粘贴、固定等操作

### 3.6 底部快捷键与状态栏

职责：

- 提示高频快捷键
- 展示当前监听状态
- 展示本地保存状态
- 后续可展示同步状态，但第一版不做云同步

第一版右下角状态文案建议：

```txt
本地保存
```

不要显示：

```txt
同步已启用
```

---

## 4. 主窗口规格

### 4.1 默认尺寸

建议默认尺寸：

```txt
宽度：1080px
高度：720px
```

允许范围：

```txt
最小宽度：880px
最小高度：600px
最大宽度：1280px
最大高度：860px
```

### 4.2 窗口位置

默认显示在屏幕中央偏上位置。

建议：

```txt
水平居中
垂直位置：屏幕高度 42% 附近
```

原因：用户打开工具通常是为了快速选择粘贴，不应让窗口过低遮挡输入区。

### 4.3 窗口行为

- `Ctrl + Shift + V` 打开窗口
- 再次按快捷键可隐藏窗口
- `Esc` 隐藏窗口
- 点击窗口外部可隐藏窗口，后续可配置
- 点击关闭按钮默认隐藏到托盘，不退出应用
- 托盘菜单中提供真正的退出入口

### 4.4 窗口样式

```txt
圆角：18px
背景：浅色半透明白 / 毛玻璃感
边框：1px 浅灰透明边框
阴影：柔和大阴影
```

视觉目标：像系统原生浮层，而不是网页弹窗。

---

## 5. 视觉设计规范

### 5.1 色彩系统

#### 主色

建议使用蓝紫色作为主色。

```txt
Primary 50:  #F4F1FF
Primary 100: #E9E2FF
Primary 200: #D4C6FF
Primary 300: #B8A2FF
Primary 400: #9577FF
Primary 500: #7357F6
Primary 600: #5B3FD7
Primary 700: #4932AF
Primary 800: #39298B
Primary 900: #2D236D
```

主色用途：

- 当前 tab 高亮
- 当前选中列表项边框
- 主按钮
- 固定状态 icon
- focus ring

#### 中性色

```txt
Gray 0:   #FFFFFF
Gray 50:  #FAFAFB
Gray 100: #F4F5F7
Gray 200: #E7E9EE
Gray 300: #D2D6DF
Gray 400: #A8AFBD
Gray 500: #747C8C
Gray 600: #525A6A
Gray 700: #353C4A
Gray 800: #202634
Gray 900: #111827
```

#### 语义色

```txt
Success: #16A34A
Warning: #F59E0B
Danger:  #EF4444
Info:    #2563EB
```

### 5.2 背景

主背景建议：

```css
background: radial-gradient(circle at 20% 10%, rgba(115, 87, 246, 0.08), transparent 32%),
            radial-gradient(circle at 80% 80%, rgba(37, 99, 235, 0.08), transparent 30%),
            #EEF2F8;
```

窗口背景建议：

```css
background: rgba(255, 255, 255, 0.82);
backdrop-filter: blur(24px);
border: 1px solid rgba(255, 255, 255, 0.66);
box-shadow: 0 24px 80px rgba(15, 23, 42, 0.18);
```

如果 Tauri/Windows 透明窗口效果不稳定，优先使用纯白半透明近似，不要为毛玻璃牺牲稳定性。

### 5.3 字体

Windows 推荐字体栈：

```css
font-family: "Segoe UI", "Microsoft YaHei UI", "Microsoft YaHei", system-ui, sans-serif;
```

代码字体：

```css
font-family: "Cascadia Code", "JetBrains Mono", "Consolas", monospace;
```

### 5.4 字号

```txt
App 标题：20px / 600
搜索框：15px / 400
Tab：14px / 500
列表主文本：15px / 500
列表元信息：13px / 400
详情标题：15px / 600
详情正文：14px / 400
按钮主文本：14px / 600
按钮快捷键：12px / 400
底部快捷键：13px / 500
```

### 5.5 圆角

```txt
主窗口：18px
搜索框：12px
列表卡片：12px
按钮：12px
Tab pill：10px
图标容器：10px
小 badge：8px
```

### 5.6 阴影

```txt
主窗口：大阴影，柔和扩散
卡片：轻阴影
选中卡片：主色边框 + 微弱主色阴影
按钮 hover：轻微提升
```

示例：

```css
--shadow-window: 0 24px 80px rgba(15, 23, 42, 0.18);
--shadow-card: 0 8px 24px rgba(15, 23, 42, 0.06);
--shadow-selected: 0 0 0 1px rgba(115, 87, 246, 0.9), 0 12px 30px rgba(115, 87, 246, 0.16);
```

---

## 6. 顶部标题栏设计

### 6.1 布局

```txt
高度：64px
左右 padding：28px
Logo 尺寸：32px
Logo 与文字间距：12px
右侧按钮间距：8px
```

结构：

```txt
[Logo] Cliply                                      [置顶] [设置] [更多] [关闭]
```

### 6.2 Logo

第一版可以使用简洁 clipboard 图标。

建议：

- 线性图标
- 蓝紫渐变
- 不要过度复杂
- 尺寸在 28-32px

### 6.3 窗口按钮

按钮类型：

```txt
置顶：Pin icon
设置：Settings icon
更多：MoreHorizontal icon
关闭：X icon
```

按钮规格：

```txt
尺寸：36px × 36px
圆角：10px
默认背景：透明
hover 背景：rgba(15, 23, 42, 0.06)
active 背景：rgba(15, 23, 42, 0.1)
```

关闭按钮 hover 可以轻微红色提示，但不要过度警告，因为关闭只是隐藏窗口。

---

## 7. 搜索区设计

### 7.1 搜索框尺寸

```txt
高度：48px
左右 margin：28px
圆角：12px
边框：1px solid Gray 200
背景：rgba(255,255,255,0.72)
```

### 7.2 内容结构

```txt
[Search icon] [placeholder / input text]                         [Ctrl + K]
```

### 7.3 状态

#### 默认状态

- 边框浅灰
- icon 灰色
- placeholder 灰色

#### 聚焦状态

- 边框主色
- 外圈 focus ring
- 背景更白

#### 输入状态

- 显示清除按钮，可选
- 搜索结果实时更新

#### 搜索中状态

如果搜索耗时超过 150ms，可显示轻量 loading spinner。

#### 无结果状态

左侧列表显示空状态，不要让右侧残留旧详情。

### 7.4 搜索行为

- 打开窗口时搜索框默认聚焦
- 直接输入即搜索
- 防抖 150ms
- `Esc` 优先清空搜索
- 搜索为空时 `Esc` 关闭窗口
- 搜索词高亮可在后续版本实现，MVP 可不做

---

## 8. 类型筛选 Tabs

### 8.1 布局

```txt
高度：40px
顶部间距：16px
左右 margin：28px
tab 间距：12px
```

### 8.2 Tab 规格

```txt
高度：36px
padding：0 20px
圆角：10px
字号：14px
```

### 8.3 Tab 状态

#### 默认

```txt
背景：rgba(255,255,255,0.52)
边框：Gray 200
文字：Gray 600
```

#### Hover

```txt
背景：White
文字：Gray 800
```

#### Active

```txt
背景：Primary 50
边框：Primary 200
文字：Primary 600
```

### 8.4 筛选项定义

```ts
const filters = [
  { key: "all", label: "全部" },
  { key: "text", label: "文本" },
  { key: "link", label: "链接" },
  { key: "image", label: "图片" },
  { key: "code", label: "代码" },
  { key: "pinned", label: "固定" },
];
```

---

## 9. 主内容区布局

### 9.1 总体布局

```txt
顶部间距：20px
左右 margin：28px
底部与 footer 间距：16px
左栏宽度：46%
右栏宽度：54%
左右栏间距：24px
```

建议 CSS：

```css
.main-content {
  display: grid;
  grid-template-columns: minmax(360px, 0.92fr) minmax(420px, 1.08fr);
  gap: 24px;
  min-height: 0;
}
```

### 9.2 响应式行为

第一版主要适配桌面窗口，不需要移动端。

但在窗口较窄时：

```txt
宽度 < 920px：右侧详情可缩小，列表仍保留
宽度 < 820px：可进入单栏模式，点击列表项进入详情
```

MVP 可以先不做单栏模式，但组件结构要支持。

---

## 10. 剪贴板历史列表设计

### 10.1 列表容器

```txt
高度：自适应主内容区
overflow-y：auto
padding-right：8px
```

滚动条：

- 细滚动条
- 默认半透明
- hover 时明显

### 10.2 列表项尺寸

```txt
最小高度：92px
padding：16px
圆角：12px
间距：10px
```

列表项结构：

```txt
┌────────────────────────────────────────┐
│ [App Icon]  类型 · 来源应用        [Pin]│
│            主预览内容                  │
│            时间 · 相对时间             │
└────────────────────────────────────────┘
```

### 10.3 App Icon 区

```txt
尺寸：52px × 52px
圆角：12px
背景：White
边框：Gray 200
```

如果无法获取真实应用图标，使用类型图标 fallback：

```txt
文本：FileText
链接：Link
图片：Image
代码：Code2
文件：File
```

### 10.4 文本区域

#### 第一行

显示：

```txt
类型 · 来源应用
```

示例：

```txt
代码 · VS Code
链接 · Chrome
文本 · 微信
图片 · 截图工具
```

样式：

```txt
字号：13px
颜色：Gray 500
```

#### 第二行

显示主预览。

样式：

```txt
字号：15px
颜色：Gray 900
字重：500
单行省略
```

对于长文本：单行省略。

对于代码：保留 monospace 或局部代码风格，MVP 可统一普通字体。

#### 第三行

显示：

```txt
10:42:18 · 刚刚
10:41:03 · 1 分钟前
昨天 18:24
2025/05/24
```

样式：

```txt
字号：13px
颜色：Gray 500
```

### 10.5 Pin 图标

状态：

```txt
未固定：灰色描边 pin
已固定：主色填充 pin
hover：主色
```

点击行为：

- 切换固定状态
- 不改变当前选中项
- 不触发粘贴

### 10.6 列表项状态

#### 默认

```txt
背景：rgba(255,255,255,0.74)
边框：1px solid rgba(226,232,240,0.9)
```

#### Hover

```txt
背景：White
阴影：轻微
```

#### Selected

```txt
背景：rgba(244,241,255,0.82)
边框：Primary 500
阴影：selected shadow
```

#### Active Pressed

```txt
scale: 0.995，可选
```

#### Disabled / Sensitive Hidden

如果内容被隐私规则隐藏：

```txt
预览：已隐藏敏感内容
icon：Shield
右侧操作受限
```

### 10.7 列表底部

显示总数：

```txt
共 128 条记录
```

如果搜索状态：

```txt
找到 8 条匹配结果
```

如果筛选状态：

```txt
共 12 条固定记录
```

---

## 11. 详情预览面板设计

### 11.1 面板整体

右侧详情面板使用大卡片结构。

```txt
背景：rgba(255,255,255,0.78)
边框：Gray 200
圆角：14px
padding：20px
```

结构：

```txt
┌──────────────────────────────────────┐
│ 详情标题                         ⋯   │
├──────────────────────────────────────┤
│ 内容预览区域                         │
├──────────────────────────────────────┤
│ 元信息区域                           │
├──────────────────────────────────────┤
│ 操作按钮区域                         │
└──────────────────────────────────────┘
```

### 11.2 详情头部

显示：

```txt
类型 · 来源应用
```

右侧显示：

- 已固定 badge
- 更多菜单

示例：

```txt
代码 · VS Code                         [已固定] [⋯]
```

### 11.3 文本预览

文本内容用可滚动区域展示。

```txt
最大高度：220px
padding：18px
圆角：12px
背景：Gray 50
边框：Gray 200
字体：14px
行高：1.6
```

长文本支持：

- 垂直滚动
- 保留换行
- 不自动编辑

### 11.4 代码预览

代码预览需要更像代码块。

```txt
背景：#FBFBFD
边框：Gray 200
圆角：12px
padding：18px
字体：Cascadia Code / Consolas
字号：14px
行高：1.65
```

支持行号：

```txt
1  const user = await getProfile()
2  if (user) {
3    return user.name
4  }
```

语法高亮第一版可以简化：

- 关键词蓝紫色
- 字符串绿色
- 注释灰色
- 普通文本深灰

如果实现成本高，MVP 可先不做完整语法高亮，但行号应保留。

### 11.5 链接预览

链接详情应显示：

```txt
URL
域名
来源应用
复制时间
```

卡片结构：

```txt
┌──────────────────────────────────────┐
│ Link icon  github.com                │
│ https://github.com/sabrogden/Ditto    │
└──────────────────────────────────────┘
```

MVP 不要默认联网抓网页标题，避免隐私风险。

后续可提供设置：

```txt
自动获取链接标题：关闭 / 开启
```

### 11.6 图片预览

图片详情显示：

- 居中缩略图
- 文件尺寸
- 图片尺寸，如果可获取
- 大小
- 来源应用

样式：

```txt
背景：棋盘格或浅灰背景
圆角：12px
最大高度：260px
object-fit：contain
```

如果图片文件丢失：

```txt
图片文件不可用
```

不要让 UI 崩溃。

### 11.7 敏感内容预览

如果内容被标记为敏感：

```txt
┌──────────────────────────────────────┐
│ Shield icon                           │
│ 此内容可能包含敏感信息，已隐藏。        │
│ 你可以在设置中调整敏感内容保存策略。    │
└──────────────────────────────────────┘
```

操作按钮限制：

- 粘贴：可用或禁用，取决于是否保存原文
- 复制：如果未保存原文，禁用
- 删除：可用

---

## 12. 元信息区域设计

### 12.1 字段

必选字段：

```txt
来源应用
复制时间
类型
大小
```

可选字段：

```txt
来源窗口
使用次数
最后使用时间
标签
hash 状态
```

### 12.2 布局

```txt
两列布局：左侧 label，右侧 value
行高：34px
icon：16px
label 颜色：Gray 600
value 颜色：Gray 700
```

示例：

```txt
来源应用    Visual Studio Code
复制时间    2025/05/24 10:42:18
类型        代码 (JavaScript)
大小        86 字符
```

### 12.3 类型文案

```txt
text   -> 文本
link   -> 链接
image  -> 图片
code   -> 代码
html   -> 富文本
file   -> 文件
mixed  -> 混合内容
```

### 12.4 大小文案

```txt
文本：86 字符
HTML：12.4 KB
图片：248 KB
文件：1.2 MB
```

---

## 13. 操作按钮设计

### 13.1 操作按钮列表

第一版显示 4 个按钮：

```txt
粘贴
复制
无格式粘贴
固定 / 取消固定
```

后续可放入更多菜单：

```txt
删除
添加标签
在历史中定位
复制为 Markdown
复制为 JSON
打开链接
在文件夹中显示
```

### 13.2 主按钮：粘贴

主按钮必须明显。

规格：

```txt
高度：72px
宽度：112px - 128px
背景：Primary 600
文字：White
圆角：12px
```

内容：

```txt
[Clipboard icon]
粘贴
Enter
```

### 13.3 次级按钮

规格：

```txt
高度：72px
背景：White
边框：Gray 200
文字：Gray 700
圆角：12px
```

示例：

```txt
[Copy icon]
复制
Ctrl+C
```

### 13.4 按钮状态

#### 默认

清晰可点击。

#### Hover

轻微上浮或背景变亮。

#### Active

轻微下压。

#### Disabled

```txt
opacity: 0.45
cursor: not-allowed
```

无格式粘贴在以下情况禁用：

```txt
当前项无 text fallback
当前项为纯图片且无 OCR 文本
当前项原文未保存
```

### 13.5 操作反馈

操作成功后：

- 粘贴：窗口关闭，不必 toast
- 复制：toast `已复制到剪贴板`
- 固定：图标状态立即变化
- 删除：列表项消失，并选择下一项

---

## 14. 底部快捷键与状态栏设计

### 14.1 布局

```txt
高度：56px
border-top：1px solid Gray 200
左右 padding：28px
```

左侧显示快捷键：

```txt
Enter 粘贴    Shift+Enter 无格式    Ctrl+P 固定    Esc 关闭
```

右侧显示状态：

```txt
本地保存
```

如果监听暂停：

```txt
监听已暂停
```

### 14.2 快捷键 chip

```txt
背景：Gray 100
边框：Gray 200
圆角：6px
padding：3px 8px
字号：12px
```

### 14.3 状态文案

状态优先级：

1. 监听已暂停
2. 本地保存
3. 只读模式，极少出现
4. 数据库错误，异常状态

示例：

```txt
绿色圆点 + 本地保存
黄色圆点 + 监听已暂停
红色圆点 + 保存异常
```

---

## 15. 右键菜单与更多菜单

### 15.1 列表项右键菜单

菜单项：

```txt
粘贴
复制
无格式粘贴
固定 / 取消固定
删除
添加标签
查看详情
```

MVP 可先实现：

```txt
粘贴
复制
固定 / 取消固定
删除
```

### 15.2 详情更多菜单

菜单项：

```txt
删除此记录
复制为纯文本
复制来源信息
添加标签
忽略此应用
清空同来源记录
```

危险操作要分组放底部，并使用红色文本。

### 15.3 顶部更多菜单

菜单项：

```txt
暂停监听
清空历史
导出数据
导入数据
关于 Cliply
退出应用
```

第一版：

```txt
暂停监听
清空历史
关于 Cliply
退出应用
```

---

## 16. 设置页设计

### 16.1 设置入口

点击顶部设置按钮打开设置弹窗或独立窗口。

MVP 建议使用 modal 弹窗，避免多窗口复杂度。

### 16.2 设置页布局

```txt
左侧分组导航 / 右侧内容
```

如果第一版设置项较少，可用单栏。

分组：

```txt
通用
快捷键
隐私
历史记录
外观
关于
```

### 16.3 通用设置

字段：

```txt
开机启动
关闭窗口时隐藏到托盘
打开 Cliply 后自动聚焦搜索框
粘贴后自动关闭窗口
```

### 16.4 快捷键设置

字段：

```txt
打开 Cliply：Ctrl + Shift + V
粘贴选中项：Enter
无格式粘贴：Shift + Enter
固定：Ctrl + P
```

第一版可只支持修改全局打开快捷键。

### 16.5 隐私设置

字段：

```txt
启用敏感内容过滤
保存疑似验证码
保存疑似密码
保存图片
保存 HTML 富文本
忽略应用列表
```

默认：

```txt
启用敏感内容过滤：开启
保存疑似验证码：关闭
保存疑似密码：关闭
保存图片：开启
保存 HTML 富文本：开启
```

### 16.6 历史记录设置

字段：

```txt
最大历史条数：1000
自动清理：30 天
清空历史
清空全部，包括固定项
```

危险操作需要二次确认。

### 16.7 外观设置

字段：

```txt
主题：跟随系统 / 浅色 / 深色
显示密度：舒适 / 紧凑
主色：蓝紫 / 蓝色 / 绿色 / 橙色
```

MVP 可以先只做浅色主题，保留设置结构。

---

## 17. 空状态设计

### 17.1 无剪贴板记录

出现条件：数据库没有任何未删除记录。

文案：

```txt
还没有剪贴板记录
复制一段文字、链接或图片后，它会出现在这里。
```

操作：

```txt
了解快捷键
打开设置
```

### 17.2 搜索无结果

出现条件：query 不为空且无搜索结果。

文案：

```txt
没有找到匹配内容
试试换个关键词，或者切换到“全部”。
```

操作：

```txt
清空搜索
```

### 17.3 筛选无结果

示例：固定 tab 无内容。

文案：

```txt
还没有固定内容
点击记录右侧的图钉，可以把常用内容固定在这里。
```

### 17.4 监听暂停

如果用户暂停监听，顶部或底部应显示状态。

文案：

```txt
监听已暂停
新的复制内容暂时不会被保存。
```

操作：

```txt
恢复监听
```

---

## 18. 加载状态设计

### 18.1 启动加载

启动数据库和加载历史时：

- 显示 skeleton list
- 不要长时间空白
- 加载超过 500ms 再显示明显 loading，避免闪烁

### 18.2 搜索加载

搜索一般应很快，超过 150ms 才显示小 spinner。

### 18.3 图片加载

图片缩略图：

- 加载前显示浅灰占位
- 加载失败显示图片损坏 icon
- 不影响列表滚动

---

## 19. 错误状态设计

### 19.1 剪贴板读取失败

提示：

```txt
读取剪贴板失败
请稍后重试，或检查是否有应用正在占用剪贴板。
```

### 19.2 粘贴失败

提示：

```txt
粘贴失败
目标窗口可能不允许自动粘贴。内容已复制到剪贴板，你可以手动粘贴。
```

这种情况下，应该把内容写入系统剪贴板，用户仍可手动 `Ctrl+V`。

### 19.3 数据库错误

提示：

```txt
保存历史记录失败
Cliply 暂时无法写入本地数据库。
```

底部状态栏显示红色状态。

---

## 20. 动效设计

### 20.1 窗口打开

```txt
opacity: 0 -> 1
transform: translateY(8px) scale(0.985) -> translateY(0) scale(1)
duration: 140ms - 180ms
easing: ease-out
```

### 20.2 窗口关闭

```txt
opacity: 1 -> 0
transform: translateY(0) scale(1) -> translateY(6px) scale(0.99)
duration: 100ms - 140ms
```

### 20.3 列表 hover

```txt
duration: 120ms
background / border / shadow 过渡
```

### 20.4 选中项切换

选中项边框和背景过渡应轻快，不要有拖泥带水的动画。

### 20.5 Toast

位置：窗口底部中间或右下角。

持续时间：

```txt
成功：1200ms
错误：3000ms
```

---

## 21. 键盘交互规范

### 21.1 全局快捷键

```txt
Ctrl + Shift + V：打开 / 隐藏 Cliply
```

### 21.2 窗口内快捷键

```txt
Ctrl + K：聚焦搜索框
ArrowDown：选择下一条
ArrowUp：选择上一条
Enter：粘贴当前选中项
Shift + Enter：无格式粘贴
Ctrl + C：复制当前选中项
Ctrl + P：固定 / 取消固定当前选中项
Delete：删除当前选中项
Esc：清空搜索或关闭窗口
Tab：切换焦点区域
```

### 21.3 Esc 行为

优先级：

```txt
如果有打开的菜单：关闭菜单
如果搜索框有内容：清空搜索
如果无搜索内容：关闭 Cliply 窗口
```

### 21.4 Enter 行为

```txt
默认：粘贴当前选中项
搜索框聚焦时：仍然粘贴当前选中项，不提交搜索表单
按钮聚焦时：触发按钮
```

为了避免误触，可以在设置中提供：

```txt
搜索框聚焦时 Enter 粘贴：开启 / 关闭
```

MVP 默认开启。

---

## 22. 鼠标交互规范

### 22.1 单击列表项

- 选中该项
- 更新右侧详情
- 不粘贴

### 22.2 双击列表项

- 粘贴该项
- 粘贴后关闭窗口

### 22.3 点击 Pin

- 切换固定状态
- 不触发列表项点击

### 22.4 右键列表项

- 打开上下文菜单
- 同时选中该项

### 22.5 点击窗口外部

MVP 可配置：

```txt
默认关闭窗口：开启
```

后续设置项：

```txt
点击窗口外关闭：开启 / 关闭
```

---

## 23. 内容类型 UI 规范

### 23.1 文本 Text

列表：

```txt
类型：文本
icon：FileText
预览：首行文本
```

详情：

- 显示完整文本
- 保留换行
- 支持复制、粘贴、无格式粘贴

### 23.2 链接 Link

识别：

```txt
http://
https://
mailto:
file://
```

列表：

```txt
类型：链接
icon：Link 或来源浏览器 icon
预览：完整 URL 或域名 + path
```

详情：

- 显示域名
- 显示完整 URL
- 提供复制、粘贴
- 后续可提供打开链接

### 23.3 代码 Code

列表：

```txt
类型：代码
icon：Code2 或来源编辑器 icon
预览：第一行代码
```

详情：

- 代码块
- 行号
- 语言识别结果
- 支持无格式粘贴

语言识别展示：

```txt
代码 (JavaScript)
代码 (TypeScript)
代码 (SQL)
代码 (JSON)
代码
```

### 23.4 图片 Image

列表：

```txt
类型：图片
icon / thumbnail：显示缩略图
预览：图片名称或 图片 1280×720 PNG
```

详情：

- 图片预览
- 尺寸
- 文件大小
- 来源应用

无格式粘贴一般禁用。

### 23.5 富文本 HTML

列表：

```txt
类型：富文本
预览：纯文本 fallback
```

详情：

MVP 可以显示纯文本 fallback，并标注：

```txt
包含富文本格式
```

后续版本可支持富文本预览。

---

## 24. 数据状态与 UI 映射

### 24.1 ClipboardItem DTO

```ts
export type ClipboardItemType = "text" | "link" | "image" | "code" | "html" | "file" | "mixed";

export type ClipboardItemDto = {
  id: string;
  type: ClipboardItemType;
  title?: string;
  previewText: string;
  sourceApp?: string;
  sourceWindow?: string;
  copiedAt: string;
  relativeTime: string;
  sizeBytes: number;
  isPinned: boolean;
  isFavorite?: boolean;
  sensitiveScore?: number;
  thumbnailUrl?: string;
  usedCount?: number;
};
```

### 24.2 Clipboard Detail DTO

```ts
export type ClipboardItemDetailDto = ClipboardItemDto & {
  fullText?: string;
  html?: string;
  imageUrl?: string;
  imageWidth?: number;
  imageHeight?: number;
  language?: string;
  formats: Array<{
    formatName: string;
    mimeType?: string;
    sizeBytes: number;
  }>;
};
```

### 24.3 UI 映射规则

```txt
isPinned = true -> pin icon 主色，详情显示 已固定 badge
sensitiveScore >= 80 -> 隐藏详情内容
thumbnailUrl 有值 -> 列表显示缩略图
type = image -> 禁用无格式粘贴
type = link -> 详情显示链接卡片
type = code -> 详情显示代码块
```

---

## 25. 组件拆分规范

建议前端组件结构：

```txt
src/components/shell/
  AppWindow.tsx
  TitleBar.tsx
  FooterShortcuts.tsx

src/components/clipboard/
  ClipboardSearchBar.tsx
  ClipboardFilterTabs.tsx
  ClipboardList.tsx
  ClipboardListItem.tsx
  ClipboardDetailPane.tsx
  ClipboardPreview.tsx
  ClipboardMetadata.tsx
  ClipboardActions.tsx
  EmptyState.tsx
  ClipboardContextMenu.tsx

src/components/settings/
  SettingsDialog.tsx
  GeneralSettings.tsx
  ShortcutSettings.tsx
  PrivacySettings.tsx
  HistorySettings.tsx
  AppearanceSettings.tsx

src/components/common/
  IconButton.tsx
  ShortcutKey.tsx
  PillTab.tsx
  Badge.tsx
  Card.tsx
  Toast.tsx
  ConfirmDialog.tsx
```

### 25.1 组件职责

#### AppWindow

- 负责主窗口布局
- 组合标题栏、搜索、筛选、内容区、底部栏

#### ClipboardSearchBar

- 负责输入、清空、快捷键提示
- 不直接请求后端，只触发 state action

#### ClipboardFilterTabs

- 负责筛选状态切换

#### ClipboardList

- 负责列表渲染
- 负责键盘选择逻辑的承接
- 可后续替换为虚拟列表

#### ClipboardListItem

- 负责单条卡片视觉
- pin 点击要 stopPropagation

#### ClipboardDetailPane

- 根据 selected item 展示详情

#### ClipboardPreview

- 根据 type 渲染不同预览

#### ClipboardActions

- 负责粘贴、复制、无格式粘贴、固定按钮

---

## 26. Tailwind Token 建议

可以在 `tailwind.config.ts` 中定义：

```ts
theme: {
  extend: {
    colors: {
      primary: {
        50: "#F4F1FF",
        100: "#E9E2FF",
        200: "#D4C6FF",
        300: "#B8A2FF",
        400: "#9577FF",
        500: "#7357F6",
        600: "#5B3FD7",
        700: "#4932AF",
        800: "#39298B",
        900: "#2D236D",
      }
    },
    boxShadow: {
      window: "0 24px 80px rgba(15, 23, 42, 0.18)",
      card: "0 8px 24px rgba(15, 23, 42, 0.06)",
      selected: "0 0 0 1px rgba(115, 87, 246, 0.9), 0 12px 30px rgba(115, 87, 246, 0.16)",
    },
    borderRadius: {
      window: "18px",
    }
  }
}
```

---

## 27. 深色模式预留

MVP 可只实现浅色，但 class/token 要能扩展 dark。

深色模式方向：

```txt
背景：#0F172A
窗口：rgba(15, 23, 42, 0.78)
卡片：rgba(30, 41, 59, 0.72)
边框：rgba(148, 163, 184, 0.18)
主文本：#F8FAFC
次级文本：#CBD5E1
```

注意：深色模式下主色不要过饱和，避免刺眼。

---

## 28. 可访问性要求

### 28.1 键盘可达

所有操作按钮都必须可通过键盘访问。

### 28.2 Focus 状态

所有可交互元素必须有清晰 focus ring。

建议：

```css
focus-visible:ring-2 focus-visible:ring-primary-400 focus-visible:ring-offset-2
```

### 28.3 对比度

正文文本与背景对比度需要足够，不要用过浅灰色显示主要内容。

### 28.4 图标不可单独传递信息

例如固定状态除了 pin icon，详情中也要显示“已固定”。

### 28.5 Reduced Motion

遵守系统减少动画设置。

```css
@media (prefers-reduced-motion: reduce) {
  * {
    animation: none !important;
    transition: none !important;
  }
}
```

---

## 29. MVP 页面清单

第一版需要实现：

```txt
1. 主浮层窗口
2. 设置弹窗
3. 清空历史确认弹窗
4. 关于弹窗
5. Toast 提示
```

可以暂不实现：

```txt
1. 独立全屏历史管理页
2. 标签管理页
3. 数据导入导出复杂页面
4. 账户页面
5. 云同步页面
```

---

## 30. 主界面 Mock 数据

用于前端 UI 阶段：

```ts
export const mockClipboardItems = [
  {
    id: "1",
    type: "code",
    title: "const user = await getProfile()",
    previewText: "const user = await getProfile()",
    sourceApp: "Visual Studio Code",
    copiedAt: "2025-05-24T10:42:18",
    relativeTime: "刚刚",
    sizeBytes: 86,
    isPinned: true,
    language: "JavaScript",
    fullText: "const user = await getProfile()\nif (user) {\n  return user.name\n}",
  },
  {
    id: "2",
    type: "link",
    title: "https://github.com/sabrogden/Ditto",
    previewText: "https://github.com/sabrogden/Ditto",
    sourceApp: "Chrome",
    copiedAt: "2025-05-24T10:41:03",
    relativeTime: "1 分钟前",
    sizeBytes: 34,
    isPinned: false,
    fullText: "https://github.com/sabrogden/Ditto",
  },
  {
    id: "3",
    type: "text",
    title: "好的，今晚 8 点前发你方案",
    previewText: "好的，今晚 8 点前发你方案",
    sourceApp: "微信",
    copiedAt: "2025-05-24T10:39:25",
    relativeTime: "3 分钟前",
    sizeBytes: 42,
    isPinned: false,
    fullText: "好的，今晚 8 点前发你方案",
  },
  {
    id: "4",
    type: "image",
    title: "产品草图.png",
    previewText: "产品草图.png",
    sourceApp: "截图工具",
    copiedAt: "2025-05-24T10:37:11",
    relativeTime: "5 分钟前",
    sizeBytes: 248000,
    isPinned: false,
    thumbnailUrl: "/mock/product-sketch.png",
  },
  {
    id: "5",
    type: "text",
    title: "jack.chen@contoso.com",
    previewText: "jack.chen@contoso.com",
    sourceApp: "Outlook",
    copiedAt: "2025-05-24T10:35:48",
    relativeTime: "7 分钟前",
    sizeBytes: 22,
    isPinned: false,
    fullText: "jack.chen@contoso.com",
  }
];
```

---

## 31. 前端验收标准

UI MVP 完成后应满足：

```txt
1. 主窗口视觉接近设计稿
2. 搜索框默认聚焦
3. mock 数据列表正常渲染
4. 当前选中项有明显高亮
5. 上下键可以切换选中项
6. 筛选 tab 可以过滤列表
7. 搜索可以过滤列表
8. 右侧详情跟随选中项变化
9. 代码详情显示代码块和行号
10. 链接详情显示链接卡片
11. 图片详情显示图片占位或缩略图
12. 操作按钮显示正确快捷键
13. 空状态可见
14. 底部快捷键提示正确
15. 窗口缩放时布局不明显破碎
```

---

## 32. 给前端实现的第一步任务

先实现以下内容，不要急着接系统剪贴板：

```txt
1. 创建主窗口布局 AppWindow
2. 创建 TitleBar
3. 创建 ClipboardSearchBar
4. 创建 ClipboardFilterTabs
5. 创建 ClipboardList 和 ClipboardListItem
6. 创建 ClipboardDetailPane
7. 创建 ClipboardPreview
8. 创建 ClipboardActions
9. 使用 mockClipboardItems 渲染完整页面
10. 实现搜索、筛选、选中、键盘上下切换
```

完成这一阶段后，再接入 Tauri commands 和 SQLite。

---

## 33. UI 不做事项

第一版 UI 不要做：

```txt
1. 账号登录入口
2. 云同步开关，除非后端已实现
3. AI 功能入口
4. 插件市场
5. 复杂团队协作
6. 过多图表或数据统计
7. 过重的动画
8. 拟物化设计
9. 大面积渐变按钮
10. 复杂侧边栏
```

Cliply 的第一印象应该是：打开就能用，几乎不用学习。

---

## 34. 最终 UI 目标

Cliply 的最终 UI 应让用户感到：

```txt
它比 Windows 自带剪贴板更强；
它比 Ditto 更现代；
它比大型剪贴板工具更轻；
它像系统能力的一部分，而不是一个额外负担。
```

第一版只要把“打开、搜索、选择、粘贴、固定、删除”做顺，Cliply 就已经具备非常好的产品基础。

