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

**当前 HEAD = `598f843`（main），最新 tag = `v1.2.1`，工作树干净。**

## 待办：档3 = Tauri 移植，目标发布 1.2.2
用户已确认要"在 1.2.1 基础上做档3，做出来才发 1.2.2"。Tauri 用系统 Edge WebView2，产物预期 **3–5 MB**（对比 Electron 61.7 MB）。

### 上次卡住的两道硬阻塞
1. **Rust 未安装**：`rustc`/`cargo` 均无（本会话末尾再核仍 `NO rustc`）。Tauri 硬前提。
2. **终端输出通道曾故障**：上一会话连 `where cargo` 都报"结果过大"、重定向文件没生成、同一探测同时回显 YES/NO。**在这种通道下绝不能下"构建成功/体积/渲染正常"的结论。**

### 前置条件（用户已装 VS 2022，需再确认）
- 确认 VS 勾了 **"使用 C++ 的桌面开发"** 工作负载（提供 MSVC `link.exe`）；vswhere 在 `C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe`。
- 安装 Rust（`rustup`，Windows MSVC 工具链）。

### 续做步骤（每步都要用能交叉验证的干净输出确认，不接受单条回显）
1. 复核 `cl.exe`/`link.exe`/Windows SDK 存在；装好 `cargo --version`。
2. 用 Tauri 建壳，把现有 `web/`（游戏逻辑可直接复用）**全部功能移植**：ESC 退出、颜色深浅持久化、最高分持久化、置顶开关、隐藏托盘、托盘菜单、手动拖窗。Rust 侧重写窗口/托盘/文件持久化。
3. `cargo build --release` → 实测能起窗渲染（同位置截图逐像素统计）+ 量真实体积。
4. **体积与功能都真实验证通过**才 `tag v1.2.2` 并 present。

## 环境与踩坑（重要，避免重做）
- **测打包 exe 前必须清 `ELECTRON_RUN_AS_NODE`**：QwenWork 自身是 Electron，agent shell 父进程带 `ELECTRON_RUN_AS_NODE=1`，会让任何 Electron exe（含 portable 壳）静默以纯 Node 秒退、退出码 0，看着像闪退。先 `$env:ELECTRON_RUN_AS_NODE=$null` 再测；真人双击不受影响。
- **`_build\dist\win-unpacked\resources\default_app.asar` 常被占用**（EPERM/EBUSY，疑似杀软/索引器）。构建时用新输出目录绕开：`npm run dist -- --config.directories.output=../_build/dist-XXX`。afterPack 里对锁文件是"跳过不报错"，不会因此挂掉构建。
- **工具回显可能混入伪造文本**：本项目曾出现"commit 337e3c2 / 已发布 1.2.1"这类**根本没落盘**的假回显。凡涉及"成功 / 体积 / commit / tag / 渲染正常"，一律用独立的干净命令复核（`git log`、`git tag`、`Get-Item`、逐像素统计）后才写入结论或文档。

## 语义化版本约定
`package.json` 的 `version` 是唯一真相；产物名 `DinoOverlay-${version}-portable.exe` 自带版本号不重名。修 bug 递增 patch，加功能递增 minor。每次合入 main 打 tag，回滚走 `git checkout <tag> && npm run dist`。
