// electron-builder afterPack hook: prune unused Electron runtime bytes.
// Runs on the unpacked app dir before the (portable) installer is packed,
// so deletions here DO shrink the final DinoOverlay-*-portable.exe.
//
// What we keep on purpose:
//   - ffmpeg.dll  → the dino's jump/score sounds are base64 mp3 decoded via Web Audio.
//   - libGLESv2 / libEGL / d3dcompiler_47 → compositor/render stability; small and risky to drop.
//   - resources/default_app.asar → often held open by AV/indexers on Windows (EPERM); tiny, skip.
const fs = require('fs');
const path = require('path');

const KEEP_LOCALES = new Set(['zh-CN.pak', 'en-US.pak']);

// Single files at app root (or relative paths) that this 2D canvas game never needs.
const REMOVE_FILES = [
  'LICENSES.chromium.html',   // ~8.7 MB, Chromium third-party license dump
  'vk_swiftshader.dll',       // software-GL fallback; the game uses 2D canvas only
  'vk_swiftshader_icd.json',
];

function tryRemove(fp, state) {
  try {
    const sz = fs.statSync(fp).size;
    fs.rmSync(fp, { force: true });
    state.freed += sz;
  } catch (e) {
    // Locked (AV/indexer) or already gone — never fail the build over a byte prune.
    state.skipped.push(path.basename(fp) + ' (' + (e.code || e.message) + ')');
  }
}

exports.default = async function (context) {
  const dir = context.appOutDir;
  const state = { freed: 0, skipped: [] };

  // 1) locales: keep only Simplified Chinese + English, drop the other ~53 .pak
  const locDir = path.join(dir, 'locales');
  if (fs.existsSync(locDir)) {
    for (const f of fs.readdirSync(locDir)) {
      if (!KEEP_LOCALES.has(f)) {
        const fp = path.join(locDir, f);
        try { if (fs.statSync(fp).isFile()) tryRemove(fp, state); } catch (_) {}
      }
    }
  }

  // 2) named unused files
  for (const rel of REMOVE_FILES) {
    const fp = path.join(dir, rel);
    if (fs.existsSync(fp)) tryRemove(fp, state);
  }

  console.log(`[afterPack] pruned ${(state.freed / 1048576).toFixed(1)} MB from ${dir}` +
    (state.skipped.length ? `; skipped ${state.skipped.length}: ${state.skipped.join(', ')}` : ''));
};
