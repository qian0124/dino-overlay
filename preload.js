// Dino Overlay — preload control bridge (safe exposure via contextIsolation).
const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('dinoApi', {
  quit: () => ipcRenderer.send('dino:quit'),
  hide: () => ipcRenderer.send('dino:hide'),
  toggleAlwaysOnTop: () => ipcRenderer.send('dino:toggle-ontop'),
  showContextMenu: () => ipcRenderer.send('dino:context-menu'),
  onState: (cb) => ipcRenderer.on('dino:state', (_e, s) => cb(s)),
  // Manual window dragging (used by the top handle).
  startDrag: () => ipcRenderer.send('dino:drag-start'),
  dragMove: () => ipcRenderer.send('dino:drag-move'),
  endDrag: () => ipcRenderer.send('dino:drag-end'),
  // Per-machine high score (stored under userData, never bundled with the exe).
  getHighScore: () => ipcRenderer.sendSync('dino:hi-get') || 0,
  saveHighScore: (v) => ipcRenderer.send('dino:hi-set', v),
});
