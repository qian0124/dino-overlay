// Dino Overlay — Electron main process.
// Transparent / frameless / always-on-top / non-resizable floating dino window.
// See HANDOFF.md §5 (window params) and §6 (todo #2).

// Only the browser (main) process should run this logic; renderer / GPU /
// utility children must not. Launched via `electron .` the main process reports
// process.type === 'browser'; a packaged run reports '' (empty).
const isBrowserProcess = !process.type || process.type === 'browser';

if (isBrowserProcess) {
  const {
    app,
    BrowserWindow,
    ipcMain,
    screen,
    Menu,
    Tray,
    nativeImage,
    globalShortcut,
  } = require('electron');
  const path = require('path');
  const fs = require('fs');

  // Windows transparent + frameless: disabling hardware acceleration greatly
  // reduces the chance of black flicker on layered windows.
  app.disableHardwareAcceleration();

  let win = null;
  let tray = null;

  // Content size: 600x150 canvas + transparent margins used as the drag strip.
  const CANVAS_W = 600;
  const CANVAS_H = 150;
  const MARGIN_X = 20;
  const MARGIN_TOP = 40; // keep in sync with overlay.css .runner-container top
  const MARGIN_BOTTOM = 16;
  const WIN_W = CANVAS_W + MARGIN_X * 2;             // 640
  const WIN_H = CANVAS_H + MARGIN_TOP + MARGIN_BOTTOM; // 206

  const CONFIG_FILE = () => path.join(app.getPath('userData'), 'overlay-config.json');

  function loadBounds() {
    try {
      const b = JSON.parse(fs.readFileSync(CONFIG_FILE(), 'utf8'));
      if (typeof b.x === 'number' && typeof b.y === 'number') return b;
    } catch (_) { /* first run */ }
    return null;
  }

  function saveBounds() {
    if (!win) return;
    try { fs.writeFileSync(CONFIG_FILE(), JSON.stringify(win.getBounds())); } catch (_) {}
  }

  function defaultBounds() {
    const wa = screen.getPrimaryDisplay().workArea;
    return {
      x: Math.round(wa.x + (wa.width - WIN_W) / 2),
      y: Math.round(wa.y + wa.height - WIN_H - 60),
      width: WIN_W,
      height: WIN_H,
    };
  }

  function createWindow() {
    const bounds = loadBounds() || defaultBounds();
    win = new BrowserWindow({
      ...bounds,
      show: true,
      transparent: true,
      frame: false,
      alwaysOnTop: true,
      resizable: false,
      movable: true,
      minimizable: false,
      maximizable: false,
      fullscreenable: false,
      hasShadow: false,
      skipTaskbar: true,
      backgroundColor: '#00000000',
      webPreferences: {
        preload: path.join(__dirname, 'preload.js'),
        contextIsolation: true,
        nodeIntegration: false,
        sandbox: false,
      },
    });

    // screen-saver level floats above normal windows and most top-most windows.
    win.setAlwaysOnTop(true, 'screen-saver');
    win.loadFile(path.join(__dirname, 'web', 'index.html'));

    win.on('closed', () => { win = null; });
    win.on('move', saveBounds); // remember position after dragging
  }

  function pushState() {
    if (win) win.webContents.send('dino:state', { onTop: win.isAlwaysOnTop() });
  }

  function buildMenu() {
    const onTop = !!(win && win.isAlwaysOnTop());
    return Menu.buildFromTemplate([
      {
        label: onTop ? '取消置顶' : '保持置顶',
        click: () => { if (win) { win.setAlwaysOnTop(!onTop, 'screen-saver'); pushState(); } },
      },
      { label: '隐藏到托盘', click: () => { if (win) win.hide(); } },
      { type: 'separator' },
      { label: '退出', click: () => app.quit() },
    ]);
  }

  function createTray() {
    const svg =
      '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16">' +
      '<rect width="16" height="16" rx="3" fill="#535353"/>' +
      '<rect x="3" y="4" width="9" height="7" fill="#fff"/>' +
      '<rect x="4" y="6" width="2" height="2" fill="#535353"/>' +
      '<rect x="9" y="9" width="3" height="1" fill="#535353"/></svg>';
    const img = nativeImage.createFromDataURL(
      'data:image/svg+xml;base64,' + Buffer.from(svg).toString('base64'));
    tray = new Tray(img);
    tray.setToolTip('Dino Overlay');
    tray.setContextMenu(buildMenu());
    tray.on('click', () => { if (win) { win.isVisible() ? win.focus() : win.show(); } });
  }

  // ---- IPC from preload ----
  ipcMain.on('dino:quit', () => app.quit());
  ipcMain.on('dino:hide', () => { if (win) win.hide(); });
  ipcMain.on('dino:toggle-ontop', () => {
    if (win) { win.setAlwaysOnTop(!win.isAlwaysOnTop(), 'screen-saver'); pushState(); }
  });
  ipcMain.on('dino:context-menu', () => { if (win) buildMenu().popup({ window: win }); });

  // ---- single instance ----
  const gotLock = app.requestSingleInstanceLock();
  if (!gotLock) {
    app.quit();
  } else {
    app.on('second-instance', () => { if (win) { win.show(); win.focus(); } });

    app.whenReady().then(() => {
      createWindow();
      createTray();
      // App-level shortcuts (do not clash with the game's space / up / enter).
      globalShortcut.register('CommandOrControl+Q', () => app.quit());
      globalShortcut.register('CommandOrControl+Shift+P', () => {
        if (win) { win.setAlwaysOnTop(!win.isAlwaysOnTop(), 'screen-saver'); pushState(); }
      });
      app.on('activate', () => {
        if (BrowserWindow.getAllWindows().length === 0) createWindow();
      });
    });
  }

  app.on('window-all-closed', () => app.quit());
  app.on('will-quit', () => globalShortcut.unregisterAll());
}
