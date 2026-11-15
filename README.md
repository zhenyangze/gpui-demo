# gpui-demo — GPUI 五类应用示例合集

基于 [GPUI](https://gpui.rs/)(Zed 编辑器背后的 GPU 加速 Rust UI 框架)构建的 **5 个功能完备的桌面应用示例**,覆盖客户端、智能体、游戏、小工具、开发框架五类场景。每个示例都可以作为新项目的起点框架。

![gpui](https://img.shields.io/badge/GPUI-0.2.2-blue) ![rust](https://img.shields.io/badge/Rust-1.94-orange) ![tests](https://img.shields.io/badge/tests-112%20passing-brightgreen)

## 项目总览

| # | Demo | 类别 | 亮点 | 截图 |
|---|------|------|------|------|
| 1 | **FlowMail** | 客户端 | 三栏邮件客户端:文件夹树、虚拟化列表、Markdown 正文、写信/草稿/回收站、模拟收信 | [截图](shots/flowmail_main.png) |
| 2 | **AgentConsole** | 智能体 | 会话管理、流式输出、工具调用可视化、任务面板、斜杠命令、可插拔引擎 | [截图](shots/agent_tasks.png) |
| 3 | **GPUI-2048** | 游戏 | 完整规则、撤销 20 步、暂停、本地 Top5 排行、深浅主题、Game Over/胜利覆盖层 | [截图](shots/game_main.png) |
| 4 | **DevToolbox** | 小工具 | 12 种开发工具:JSON/正则/编解码/哈希/时间戳/UUID/进制/颜色/Diff/文本/计算器 | [截图](shots/toolbox_json.png) |
| 5 | **gpui-kit + Gallery** | 开发框架 | 30+ 组件的设计系统(主题 token/图标/命令面板/通知),画廊即 Showcase | [截图](shots/gallery_basics.png) |

## 快速开始

```bash
# 环境要求:macOS + Xcode(命令行工具)、Rust stable 1.94+
git clone <本仓库> && cd gpui-demo
cargo build --release

# 运行任一 demo
cargo run -p demo-flowmail        # 邮件客户端
cargo run -p demo-agentconsole    # 智能体工作台(试:输入 (12+8)*3 或 "帮我计划发布")
cargo run -p demo-game2048        # 2048(方向键/WASD 移动,U 撤销)
cargo run -p demo-devtoolbox      # 工具箱(⌘K 切换工具)
cargo run -p demo-gallery         # 组件画廊(⌘K 命令面板)

# gpui-kit 最小示例
cargo run -p gpui-kit --example hello_kit
```

> **注意**:本机 Xcode 26 缺少 Metal Toolchain 组件时,workspace 已启用 gpui 的 `runtime_shaders`
> feature(运行时经系统 Metal 框架编译着色器),无需额外下载组件。

## 数据目录

所有本地数据(JSON 持久化)位于 `~/.gpui-demo/<app>/`,删除对应目录即可重置:

- `~/.gpui-demo/flowmail/state.json` — 邮件、文件夹、设置
- `~/.gpui-demo/agentconsole/` — 会话与设置
- `~/.gpui-demo/game2048/save.json` — 存档、排行榜
- `~/.gpui-demo/devtoolbox/state.json` — 工具状态、收藏

## 各 demo 启动参数(演示/验收辅助)

| App | 参数 | 效果 |
|-----|------|------|
| demo-flowmail | `<邮件ID>` / `compose` | 启动时打开该邮件 / 打开写信窗 |
| demo-agentconsole | `tasks` | 预置任务面板演示 |
| demo-game2048 | `over` / `win` | 直达 Game Over / 胜利覆盖层 |
| demo-devtoolbox | `light` | 强制浅色主题 |
| demo-gallery | `overview/basics/forms/navigation/feedback/data/about` | 直达指定组件页 |

## gpui-kit 使用方式(框架接入)

```rust
use gpui::{prelude::*, Application};

fn main() {
    Application::new()
        .with_assets(gpui_kit::icons::asset_source()) // 内嵌 40+ 图标
        .run(|cx: &mut App| {
            gpui_kit::init_kit(cx); // 主题/快捷键/通知系统
            cx.open_window(WindowOptions::default(), |_, cx| {
                cx.new(|_| MyRoot)   // 直接使用 Button / InputState / Modal / Toast …
            }).unwrap();
        });
}
```

组件清单(30):Button/IconButton/Icon/Badge/Tag/Avatar/Divider/Kbd/Spinner/ProgressBar/
Skeleton/Card/EmptyState/Stat/ColorSwatch/Tooltip/TextInput/TextArea/Switch/Checkbox/Radio/
Slider/Select/Tabs/Segmented/Breadcrumb/Toolbar/NavItem/Pagination/Table/TreeView/
Menu/Modal/Toast/CommandPalette/Markdown 渲染器。

## 目录结构

```
gpui-demo/
├── crates/
│   ├── gpui-kit/           # 开发框架:设计系统 + 组件库(含 examples/hello_kit.rs 与输入管线测试)
│   ├── demo-flowmail/      # 客户端
│   ├── demo-agentconsole/  # 智能体(Engine trait + MockEngine)
│   ├── demo-game2048/      # 游戏(board.rs 纯逻辑 19 测试)
│   ├── demo-devtoolbox/    # 小工具(tools.rs 纯逻辑 20 测试)
│   └── demo-gallery/       # 框架 Showcase
├── docs/                   # PRD、3 轮评审记录、验收报告
├── shots/                  # 全部截图
└── scripts/                # winid.swift / shot.sh(截图脚本)
```

## 架构(v0.2 分层设计)

```
L4 应用层   demo-*(业务状态 + 页面编排)
L3 组件层   components/(30+ 受控组件,ValueBinding 对接 L1)
L2 服务层   Store 响应式状态 · History 撤销框架 · Persist 版本化迁移
            Commands 命令中心 · Router 栈式导航 · Highlight 高亮引擎
            Sortable 拖拽排序 · perf 渲染观测
L1 基础层   theme tokens / icons(AssetSource)/ 自绘元素 / util
L0 平台     GPUI 0.2.2(runtime_shaders,Metal)
```

详见 [ARCHITECTURE.md](docs/ARCHITECTURE.md)。核心子系统全部单测(41 个纯逻辑用例),
并在 demo 中实证:2048 撤销由 History 驱动、存档走真实迁移链(v1→v2)、
AgentConsole 工具注册制 + 任务面板拖拽排序、DevToolbox 页面路由、
Gallery「架构演示」页提供 Store/高亮/拖拽的 live demo。

## 文档

- [总体方案](docs/00-总体方案.md)
- [PRD-01 FlowMail](docs/demos/01-flowmail.md) · [PRD-02 AgentConsole](docs/demos/02-agentconsole.md) · [PRD-03 2048](docs/demos/03-game2048.md) · [PRD-04 DevToolbox](docs/demos/04-devtoolbox.md) · [PRD-05 gpui-kit](docs/demos/05-gpui-kit.md)
- 评审:[第 1 轮](docs/eval/round-1.md) · [第 2 轮](docs/eval/round-2.md) · [第 3 轮](docs/eval/round-3.md)
- [验收报告(逐功能点)](docs/验收报告.md)

## 接入真实 LLM(AgentConsole)

实现 `AgentEngine` trait 即可接入任何 OpenAI 兼容 API:

```rust
pub trait AgentEngine: 'static {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn respond(&self, input: &str, model: &str, temperature: f64) -> Vec<StreamEvent>;
}
```

返回 `Vec<StreamEvent>`(`TextDelta / ToolCall / Tasks / Done`),UI 层负责按速度设置逐帧流式播放。
在 `AgentApp::new` 中把 `MockEngine` 替换为你的实现即可(网络请求建议放后台 executor)。

## 测试与质量

```bash
cargo test --workspace   # 112 个用例:L2 核心(Store/History/Persist/Command/Router/Highlight/Sortable)+ 棋盘规则/意图路由/流式管线/输入管线/工具逻辑
cargo build --workspace --release
```

- 输入框无头测试:通过 gpui `test-support` 模拟与真实打字**同管线**的按键分发(中文输入、grapheme 退格、⌘A 选区、IME 插入)。
- 三轮文档评审:26 项问题闭环(见 docs/eval/)。

## 许可

MIT(示例代码)。GPUI 归属 Zed Industries,Apache-2.0。
