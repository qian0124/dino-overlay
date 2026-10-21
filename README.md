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
- **打包瘦身（v1.2.1）**：`afterPack.js` 在打包前裁剪 Electron 运行时——locales 只留 `zh-CN`/`en-US`（删其余 ~53 个 .pak）、移除 `LICENSES.chromium.html`、`vk_swiftshader*`、多余的 `default_app.asar`，体积明显下降。故意保留 `ffmpeg.dll`（恐龙音效是 base64 mp3，靠它解码）与 GPU 相关 dll（渲染稳定性）。若某次更新后 Electron 升级导致裁剪报错，删掉 `package.json` 里的 `"afterPack"` 一行即可回到全量打包。

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


## 目录
```
main.js  preload.js  package.json  .npmrc
web/  index.html  index.css  index.js  overlay.css  assets/*.png
```
