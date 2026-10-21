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
- **开始 / 跳**：点画布任意处，或按 空格 / ↑。
- **重开**：Enter，或死亡后再点画布。
- **趴下 / 俯冲**：↓ 或 **Ctrl**（Ctrl + ↑/空格 会触发俯冲）。
- **暂停 / 继续**：**Tab**。窗口失焦（Alt+Tab 到别的应用）会自动暂停；恢复需要主动按键或点画布，拖动窗口、右键都不会误恢复。
- **退出**：**ESC** 一键退出（窗口有焦点时），或 `Ctrl+Q`（全局快捷键）。
- **移动窗口**：拖顶部**小把手**（拖动基准块）。
- **右键**：弹出原生操作菜单（置顶开关 / 颜色深浅 / 隐藏到托盘 / 退出）。
- **颜色深浅**：右键菜单或底部托盘图标菜单 →「颜色深浅」子菜单，选 20% / 40% / 60% / 80% / 100%（默认 80%）。调节恐龙 / 障碍 / 地面的不透明度，让它更淡融入背景或更清晰；顶部拖动把手不受影响。选择记在本机、重启保留。
- **快捷键**：`ESC` 一键退出（需窗口聚焦），`Ctrl+Q` 退出（全局，任意焦点下有效），`Ctrl+Shift+H` 隐藏到托盘，`Ctrl+Shift+P` 切换置顶。
- 系统托盘常驻。

## 最高分持久化
- 存储位置：`%APPDATA%\Dino Overlay\highscore.json`（Windows 用户目录，**不进 exe**）。
- 效果：同机重启进程，最高分保留；把 exe 拷贝给朋友，他看不到你的分数（他没那个 json 文件）。
- 想清空：手动删这个 json 文件即可。
- 引擎只在 `gameOver()` 里把新记录推给主进程，写盘幂等：只有比当前记录更高才落盘，不会因异常回退。

## 关键实现点（踩坑记录）
1. **主进程判别**：`electron.exe .` 直启时主进程 `process.type === 'browser'`（非空串）；打包运行则是 `''`。用 `!process.type || process.type === 'browser'` 守卫，避免 GPU/工具子进程加载 main.js 时因 `app` undefined 崩溃。
2. **透明窗**：`transparent + frame:false + hasShadow:false + skipTaskbar + backgroundColor:'#00000000'` + `setAlwaysOnTop(true,'screen-saver')`，并 `app.disableHardwareAcceleration()` 降低黑块概率。
3. **画布尺寸**：`overlay.css` 把 `.interstitial-wrapper` 用 flex 稳定居中、`padding:0`，引擎 `adjustDimensions()` 据此测得 canvas=600×150；顶部有独立的**拖动把手**。
4. **禁用 Arcade Mode**：引擎 `startGame()` 会调用 `setArcadeMode()` 给容器加 `transform:scale()`，在 Chrome 全屏是必要的，但把画布塞进固定小窗会把恐龙推到中间、跑道被裁、右上角分数丢失。已在 `web/index.js` 里把 `setArcadeMode` / `setArcadeModeContainerScale` 改成 no-op。
5. **暂停语义**：`Runner.stop()` 是真的 `cancelAnimationFrame + paused=true`（画面完全冻结），不是视觉遮罩。原本引擎失焦自动 `stop()`、聚焦自动 `play()`；聚焦自动 play 会让拖动窗口 / 右键弹菜单误恢复游戏。改成**只暂停不自动恢复**，恢复交给 Tab / 空格 / ↑ / 点画布。
6. **改键 DUCK 加 Ctrl**：`Runner.keycodes.DUCK` 从 `{ '40': 1 }` 扩到 `{ '40': 1, '17': 1 }`；引擎的 keydown/keyup 处理已经能识别 Ctrl 复合键。
7. **游戏初始化空指针**：离线页缺少 `.icon` 元素，`init()` 里 `document.querySelector('.icon').style` 会抛 `null`；已在 `web/index.js` 加空值保护。
8. **`web/index.html` 顶部原为 `/* */` 注释在 `<!doctype>` 之前**，会让 CSP `<meta>` 落到 head 外；已改为规范结构。
9. **拖动把手用手动 IPC**：`-webkit-app-region: drag` 嵌在 `no-drag` 父容器里命中不稳定；改成把手 mousedown → 主进程记录 `cursor - window` 偏移，mousemove 用 `screen.getCursorScreenPoint()` 实时 `setPosition`，并节流写位置记忆。
10. **最高分持久化**：主进程读写 `userData/highscore.json`，preload 用 `ipcRenderer.sendSync('dino:hi-get')` 一次性拉取、`send('dino:hi-set', v)` 增量落盘；渲染进程在 `window.runner` 就绪后（`distanceMeter` 出现）给实例的 `gameOver` 挂 hook。文件在**用户 AppData**、**不在 exe** 里，所以拷贝 exe 给朋友不会带上你的记录。
11. **ESC 一键退出**：渲染层捕获阶段 `keydown` 监听里拦 `Escape`（keyCode 27），`preventDefault + stopPropagation` 后走已有 IPC 通道 `dino:quit → app.quit()`。引擎本身不使用 ESC（JUMP/DUCK/RESTART 键表里都没有 27），无冲突。注意 ESC 需**窗口聚焦**；后台退出仍走全局 `Ctrl+Q`。
12. **打包 exe 从 agent/自动化 shell 启动会“秒退”**：若父进程环境带 `ELECTRON_RUN_AS_NODE=1`，Electron 应用（含 portable 壳）会静默以纯 Node 运行、退出码 0，看起来像闪退。先清掉该变量再测；真人双击不受影响。
13. **颜色深浅（不透明度）**：菜单里用一组 `type:'radio'` 档位（20/40/60/80/100%，默认 80%）。选中 → 主进程写 `userData/opacity.json`（独立文件，避免被 `saveBounds()` 覆盖），再 `webContents.send('dino:set-opacity', v)` 通知渲染层设 `.runner-container` 的 `style.opacity`，并用 `tray.setContextMenu(buildMenu())` 刷新托盘菜单的勾选。`index.html` 启动时 `sendSync('dino:opacity-get')` 拉初始值；`.runner-container` 由引擎异步创建，故 `applyOpacityDom()` 幂等、并在高轮询里重试直到容器出现。只淡恐龙画面，不影响顶部拖动把手。右键与托盘两处菜单共用 `buildMenu()`，改一处即两处生效。

## 版本约定
- `package.json` 的 `version` 是唯一真相；产物名 `DinoOverlay-${version}-portable.exe` 自动带版本号，**不会重名覆盖**。
- 语义化：修 bug 递增 patch (1.0.x)，加功能递增 minor (1.x.0)。
- 每次合入 main 打一个 tag（`v1.0.0` / `v1.0.1` / `v1.1.0` ...），回滚走 `git checkout <tag> && npm run dist`，而不是留旧 exe。

## 目录
```
main.js  preload.js  package.json  .npmrc
web/  index.html  index.css  index.js  overlay.css  assets/*.png
```
