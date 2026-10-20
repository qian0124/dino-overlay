# Dino Overlay — 透明悬浮小恐龙（Electron 封装）

把 Chrome 断网小恐龙 (T-Rex Runner) 封装成**不依赖浏览器、透明背景、悬浮在其他窗口上层**的可执行程序。按 `HANDOFF.md` §6 落地。

## 运行（开发）
```powershell
# 本仓库已安装依赖；如缺：
npm install            # 走 .npmrc 里的 npmmirror 镜像
npm start             # = electron .
```
> 若在已设置 `ELECTRON_RUN_AS_NODE=1` 的终端里跑 `node_modules\electron\dist\electron.exe`，会退化成纯 Node 而报 `app` undefined；用 `npm start` 或干净终端即可。

## 打包（分发 exe）
```powershell
npm run dist          # = electron-builder --win portable
```
产物：`../\_build\dist\DinoOverlay-<version>-portable.exe`（单文件免安装，双击即用）。
- `package.json` 里 `"electronDist": "node_modules/electron/dist"` 让 electron-builder 复用本机已缓存的 Electron，避免再次从 GitHub 下载。
- Electron 版本固定为 **33.4.11**（win32-x64）。

## 操作方式
- **开始 / 跳 / 重开**：点窗口任意处，或按 空格 / ↑（重开用 Enter）。
- **移动窗口**：拖动恐龙画布**四周的透明边距**（`-webkit-app-region: drag`）。
- **右键**：弹出菜单（置顶开关 / 隐藏到托盘 / 退出）。
- **快捷键**：`Ctrl+Q` 退出，`Ctrl+Shift+H` 隐藏到托盘，`Ctrl+Shift+P` 切换置顶。
- 系统托盘常驻，失焦时游戏自动暂停（引擎自带行为）。

## 关键实现点（踩坑记录）
1. **主进程判别**：`electron.exe .` 直启时主进程 `process.type === 'browser'`（非空串）；打包运行则是 `''`。用 `!process.type || process.type === 'browser'` 守卫，避免 GPU/工具子进程加载 main.js 时因 `app` undefined 崩溃。
2. **透明窗**：`transparent + frame:false + hasShadow:false + skipTaskbar + backgroundColor:'#00000000'` + `setAlwaysOnTop(true,'screen-saver')`，并 `app.disableHardwareAcceleration()` 降低黑块概率。
3. **画布尺寸**：`overlay.css` 把 `.interstitial-wrapper` 设为 600×150、`padding:0`，引擎 `adjustDimensions()` 据此测得 canvas=600×150；四周留透明边距作拖拽条。
4. **游戏初始化空指针**：离线页缺少 `.icon` 元素，`init()` 里 `document.querySelector('.icon').style` 会抛 `null`；已在 `web/index.js` 加空值保护。
5. `web/index.html` 顶部原为 `/* */` 注释在 `<!doctype>` 之前，会让 CSP `<meta>` 落到 head 外；已改为规范结构。

## 目录
```
main.js  preload.js  package.json  .npmrc
web/  index.html  index.css  index.js  overlay.css  assets/*.png
```
