# 跨会话交接 — Dino Overlay

> 给**同一项目的新会话**：本文件是当前真实进度与续做指南。下面所有"已完成"都已用可交叉验证的干净 git/文件输出核实过，可信。

## 项目是什么
Chrome 断网小恐龙（T-Rex Runner）封装成 **Electron 透明无边框置顶悬浮桌宠**，不依赖浏览器。

- **整个 `/dino` 文件夹走 git，不走 Qwen 通用会话程序**。本仓库根目录：
  `...\Qwen\dino\qwenwork\canvas\mvgdcpofe0a3149a\dino-overlay`（已 git 化）。
- 产物惯例：打包出的 `DinoOverlay-<ver>-portable.exe` 复制到 `...\canvas\mvgdcpofe0a3149a\outputs\`。
- 窗口/IPC/持久化等实现细节见仓库内 `README.md` 的「关键实现点（踩坑记录）」。

## 版本线（全部真实，已打 tag）
| tag | commit | 内容 | 验证 |
|---|---|---|---|
| v1.1.1 | `675e15a` | ESC 一键退出（窗口聚焦时，走 `dino:quit→app.quit`，引擎不占 ESC 无冲突） | 真机冒烟：聚焦按 ESC 全进程干净退出 |
| v1.2.0 | `074a9be` | 颜色深浅=画面不透明度，右键菜单+托盘菜单子项（20/40/60/80/100%，默认80%），持久化 `userData/opacity.json` | 默认(0.8)与强制(0.2)同位置 mean-luma 差异明显 |
| v1.2.1 | `598f843` | afterPack 裁剪 Electron 运行时瘦身 | 69.2→61.7 MB；逐像素 A/B 裁剪前后 dark/mid/bright 计数完全相同、4 进程稳定 |
| v1.2.2 | 见下 | **档3 = Tauri v2 壳**（`src-tauri/`），复用 `web/` 零改动，Rust 重写窗口/托盘/持久化 | 单文件 exe **3.24 MB**（≈Electron 1/19）；独立目录无 web/ 无 dll 可运行；枚举到 `vis=True 640×206` TOPMOST 窗；真人屏见恐龙运行 |

**最新 tag = `v1.2.2`（Tauri 档3）。Electron 线仍停在 `v1.2.1`，源码完整保留可回滚。**

## 已完成：档3 = Tauri 移植（v1.2.2）
两道硬阻塞都已解除，并已发布：
1. **终端通道**：本会话用"写哨兵文件 + Read 工具独立回读"证明可信；上轮"值被吞"只是 PowerShell 隐式输出被格式吞掉的假象。
2. **Rust/MSVC**：根因纠正——本机原本连 MSVC `link.exe` + Windows SDK 都没有（不是只缺 Rust）。已 `winget` 装 VS 2022 Build Tools 的 `VCTools` 工作负载（MSVC 14.44 + SDK 10.0.26100），再 `rustup` 装 `stable-x86_64-pc-windows-msvc`（rustup+crates 均走 `rsproxy.cn` 镜像）。`cargo build --release` 端到端链接+运行 hello.exe 通过。

实现：`src-tauri/` 是 Tauri v2 壳，`frontendDist=../web` **复用游戏零改动**；Rust 侧 `WebviewWindowBuilder` 建透明/无边框/置顶/640×206 窗，`initialization_script` 注入 `window.dinoApi`+`window.__DINO_BOOT__`（同步 getter），托盘/右键共用 `build_menu`，拖拽映射 `start_dragging`，全局 `Ctrl+Q`/`Ctrl+Shift+P`，单实例，持久化在 `%APPDATA%\com.qwenwork.dino-overlay\{overlay-config,opacity,highscore}.json`。产物单文件 `dino-overlay.exe` **3.24 MB**（前端编译期内嵌，无 web/ 无 dll 也能跑）。

### 复现构建（新会话若要重打）
```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
cd src-tauri
cargo build --release         # 首次会编译整棵依赖树，约 2 分钟（已配 rsproxy 镜像）
# 产物：target\release\dino-overlay.exe
```
关键 API 对齐的是 **tauri 2.12.4 / global-shortcut 2.4.1 / single-instance 2.5.4**；改版本时注意 `Menu` 用 `window.popup_menu(&menu)`（非 `menu.popup`）、`Submenu::with_items(manager,text,enabled,&[&dyn IsMenuItem])`、`is_always_on_top()` 返回 `Result<bool>`、`Shortcut` 的 `key`/`mods` 是字段、`tauri-build` 需 `icons/icon.ico`。

### 待办（下一步可选，非阻塞）
- 置顶用 TOPMOST，非 Electron 的 `screen-saver` 级；若要完全等同可研究 Tauri 的窗口层级/`HWND_TOPMOST` 之上策略。
- 双轨维护成本：Electron（功能全量基准）与 Tauri（体积优先）并存在同一仓库，`src-tauri/` 与根目录 Electron 源互不影响。是否长期双轨或择一，留给用户决策。

## 环境与踩坑（重要，避免重做）
- **测打包 exe 前必须清 `ELECTRON_RUN_AS_NODE`**：QwenWork 自身是 Electron，agent shell 父进程带 `ELECTRON_RUN_AS_NODE=1`，会让任何 Electron exe（含 portable 壳）静默以纯 Node 秒退、退出码 0，看着像闪退。先 `$env:ELECTRON_RUN_AS_NODE=$null` 再测；真人双击不受影响。
- **`_build\dist\win-unpacked\resources\default_app.asar` 常被占用**（EPERM/EBUSY，疑似杀软/索引器）。构建时用新输出目录绕开：`npm run dist -- --config.directories.output=../_build/dist-XXX`。afterPack 里对锁文件是"跳过不报错"，不会因此挂掉构建。
- **工具回显可能混入伪造文本**：本项目曾出现"commit 337e3c2 / 已发布 1.2.1"这类**根本没落盘**的假回显。凡涉及"成功 / 体积 / commit / tag / 渲染正常"，一律用独立的干净命令复核（`git log`、`git tag`、`Get-Item`、逐像素统计）后才写入结论或文档。

## 语义化版本约定
`package.json` 的 `version` 是唯一真相；产物名 `DinoOverlay-${version}-portable.exe` 自带版本号不重名。修 bug 递增 patch，加功能递增 minor。每次合入 main 打 tag，回滚走 `git checkout <tag> && npm run dist`。
