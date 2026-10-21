// Dino Overlay — Tauri v2 shell (path-3).
// Reuses the existing web/ (game logic untouched). This Rust side reimplements:
//   transparent/frameless/always-on-top/skip-taskbar window, native drag,
//   right-click + tray native menu (top / color depth / hide / quit),
//   per-machine persistence (bounds, high score, opacity), global shortcuts,
//   single instance. Sync getters (opacity/high score) are injected at boot so
//   the unmodified renderer glue in web/index.html keeps working.

use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use tauri::menu::{CheckMenuItem, IsMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

const WIN_W: f64 = 640.0;
const WIN_H: f64 = 206.0;
const OPACITY_STEPS: [f64; 5] = [0.2, 0.4, 0.6, 0.8, 1.0];
const OPACITY_DEFAULT: f64 = 0.8;

// ---------- persistence ----------

fn data_dir(app: &AppHandle) -> PathBuf {
    let d = app.path().app_data_dir().expect("no app data dir");
    let _ = fs::create_dir_all(&d);
    d
}
fn config_file(app: &AppHandle) -> PathBuf {
    data_dir(app).join("overlay-config.json")
}
fn highscore_file(app: &AppHandle) -> PathBuf {
    data_dir(app).join("highscore.json")
}
fn opacity_file(app: &AppHandle) -> PathBuf {
    data_dir(app).join("opacity.json")
}

#[derive(Deserialize)]
struct PosJson {
    x: f64,
    y: f64,
}
#[derive(Deserialize)]
struct HiJson {
    hi: i64,
}
#[derive(Deserialize)]
struct OpJson {
    opacity: f64,
}

fn clamp_opacity(v: f64) -> f64 {
    v.clamp(0.1, 1.0)
}

fn read_opacity(app: &AppHandle) -> f64 {
    fs::read_to_string(opacity_file(app))
        .ok()
        .and_then(|s| serde_json::from_str::<OpJson>(&s).ok())
        .map(|o| clamp_opacity(o.opacity))
        .unwrap_or(OPACITY_DEFAULT)
}
fn write_opacity(app: &AppHandle, v: f64) {
    let v = clamp_opacity(v);
    let _ = fs::write(opacity_file(app), format!("{{\"opacity\":{}}}", v));
}

fn read_high_score(app: &AppHandle) -> i64 {
    fs::read_to_string(highscore_file(app))
        .ok()
        .and_then(|s| serde_json::from_str::<HiJson>(&s).ok())
        .map(|h| h.hi.max(0))
        .unwrap_or(0)
}
fn write_high_score(app: &AppHandle, v: i64) {
    let cur = read_high_score(app);
    let next = cur.max(v.max(0));
    if next == cur {
        return; // never downgrade, never redundant write
    }
    let _ = fs::write(highscore_file(app), format!("{{\"hi\":{}}}", next));
}

fn read_pos(app: &AppHandle) -> Option<(f64, f64)> {
    fs::read_to_string(config_file(app))
        .ok()
        .and_then(|s| serde_json::from_str::<PosJson>(&s).ok())
        .map(|p| (p.x, p.y))
}

fn save_bounds(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        if let (Ok(p), Ok(sf)) = (w.outer_position(), w.scale_factor()) {
            let (lx, ly) = (p.x as f64 / sf, p.y as f64 / sf);
            let _ = fs::write(config_file(app), format!("{{\"x\":{},\"y\":{}}}", lx, ly));
        }
    }
}

fn default_pos(app: &AppHandle) -> (f64, f64) {
    if let Some((x, y)) = read_pos(app) {
        return (x, y);
    }
    let (mx, my, mw, mh) = app
        .primary_monitor()
        .ok()
        .flatten()
        .map(|m| {
            let sf = m.scale_factor();
            let wa = m.work_area();
            (
                wa.position.x as f64 / sf,
                wa.position.y as f64 / sf,
                wa.size.width as f64 / sf,
                wa.size.height as f64 / sf,
            )
        })
        .unwrap_or((0.0, 0.0, 1920.0, 1080.0));
    (
        (mx + (mw - WIN_W) / 2.0).round(),
        (my + mh - WIN_H - 60.0).round(),
    )
}

// ---------- native menu ----------

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let on_top = app
        .get_webview_window("main")
        .map(|w| w.is_always_on_top().unwrap_or(true))
        .unwrap_or(true);
    let cur = read_opacity(app);

    let top = MenuItem::with_id(
        app,
        "top",
        if on_top { "取消置顶" } else { "保持置顶" },
        true,
        None::<&str>,
    )?;

    let mut opacity_items: Vec<CheckMenuItem<tauri::Wry>> = Vec::new();
    for v in OPACITY_STEPS {
        let id = format!("op:{}", v);
        let label = format!("{}%", (v * 100.0).round());
        let checked = (cur - v).abs() < 0.001;
        opacity_items.push(CheckMenuItem::with_id(app, id, label, true, checked, None::<&str>)?);
    }
    let refs: Vec<&dyn IsMenuItem<tauri::Wry>> = opacity_items
        .iter()
        .map(|i| i as &dyn IsMenuItem<tauri::Wry>)
        .collect();
    let opacity = Submenu::with_items(app, "颜色深浅", true, &refs)?;

    let hide = MenuItem::with_id(app, "hide", "隐藏到托盘", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    Menu::with_items(app, &[&top, &opacity, &hide, &sep, &quit])
}

fn apply_opacity(app: &AppHandle, v: f64) {
    let v = clamp_opacity(v);
    write_opacity(app, v);
    let _ = app.emit("dino:opacity", v);
    refresh_tray_menu(app);
}

fn toggle_on_top(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let next = !w.is_always_on_top().unwrap_or(true);
        let _ = w.set_always_on_top(next);
        refresh_tray_menu(app);
    }
}

fn refresh_tray_menu(app: &AppHandle) {
    if let Some(tray) = app.tray_by_id("main") {
        if let Ok(menu) = build_menu(app) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

// ---------- commands (invoked from the injected dinoApi adapter) ----------

#[tauri::command]
fn dino_quit(app: AppHandle) {
    app.exit(0);
}
#[tauri::command]
fn dino_hide(window: WebviewWindow) {
    let _ = window.hide();
}
#[tauri::command]
fn dino_toggle_ontop(app: AppHandle) {
    toggle_on_top(&app);
}
#[tauri::command]
fn dino_drag_start(window: WebviewWindow) {
    let _ = window.start_dragging();
}
#[tauri::command]
fn dino_drag_end(app: AppHandle) {
    save_bounds(&app);
}
#[tauri::command]
fn dino_save_high_score(app: AppHandle, value: i64) {
    write_high_score(&app, value);
}
#[tauri::command]
fn dino_show_context_menu(app: AppHandle, window: WebviewWindow) {
    if let Ok(menu) = build_menu(&app) {
        let _ = window.popup_menu(&menu);
    }
}

// ---------- boot injection for sync getters ----------

fn boot_script(opacity: f64, high_score: i64) -> String {
    format!(
        r#"
window.__DINO_BOOT__ = {{ opacity: {opacity}, highscore: {high_score} }};
(function () {{
    var T = window.__TAURI__;
    if (!T) return;
    function invoke(cmd, args) {{ try {{ T.core.invoke(cmd, args || {{}}); }} catch (_) {{}} }}
    window.dinoApi = {{
        quit: function () {{ invoke('dino_quit'); }},
        hide: function () {{ invoke('dino_hide'); }},
        toggleAlwaysOnTop: function () {{ invoke('dino_toggle_ontop'); }},
        showContextMenu: function () {{ invoke('dino_show_context_menu'); }},
        onState: function (cb) {{ T.event.listen('dino:state', function (e) {{ cb(e.payload); }}); }},
        getOpacity: function () {{
            var v = window.__DINO_BOOT__ && window.__DINO_BOOT__.opacity;
            return (typeof v === 'number' && isFinite(v)) ? v : 0.8;
        }},
        onOpacity: function (cb) {{ T.event.listen('dino:opacity', function (e) {{ cb(e.payload); }}); }},
        startDrag: function () {{ invoke('dino_drag_start'); }},
        dragMove: function () {{ /* native dragging */ }},
        endDrag: function () {{ invoke('dino_drag_end'); }},
        getHighScore: function () {{
            var v = (window.__DINO_BOOT__ && window.__DINO_BOOT__.highscore) || 0;
            return Number(v) || 0;
        }},
        saveHighScore: function (val) {{ invoke('dino_save_high_score', {{ value: val }}); }}
    }};
}})();
"#
    )
}

/// 16x16 RGBA tray glyph (dino-ish), built in code so no external asset is needed.
fn tray_image() -> tauri::image::Image<'static> {
    const S: usize = 16;
    let mut buf = vec![0u8; S * S * 4];
    let bg = [0x53u8, 0x53, 0x53, 0xff];
    let fg = [0xffu8, 0xff, 0xff, 0xff];
    for y in 0..S {
        for x in 0..S {
            let mut c = [0u8, 0, 0, 0];
            // rounded-ish background block
            if x >= 1 && x <= 14 && y >= 3 && y <= 12 {
                c = bg;
            }
            // body
            if x >= 3 && x <= 12 && y >= 5 && y <= 11 {
                c = fg;
            }
            // eye
            if x >= 4 && x <= 5 && y >= 6 && y <= 7 {
                c = bg;
            }
            // leg
            if x >= 9 && x <= 11 && y >= 10 && y <= 11 {
                c = bg;
            }
            let i = (y * S + x) * 4;
            buf[i] = c[0];
            buf[i + 1] = c[1];
            buf[i + 2] = c[2];
            buf[i + 3] = c[3];
        }
    }
    tauri::image::Image::new_owned(buf, S as u32, S as u32)
}

// ---------- app entry ----------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();

            let (x, y) = default_pos(&handle);
            let opacity = read_opacity(&handle);
            let high_score = read_high_score(&handle);

            let _win = WebviewWindowBuilder::new(&handle, "main", WebviewUrl::App("index.html".into()))
                .title("DinoOverlay")
                .inner_size(WIN_W, WIN_H)
                .position(x, y)
                .decorations(false)
                .transparent(true)
                .always_on_top(true)
                .skip_taskbar(true)
                .resizable(false)
                .maximizable(false)
                .minimizable(false)
                .shadow(false)
                .visible(true)
                .focused(true)
                .initialization_script(&boot_script(opacity, high_score))
                .build()?;

            let menu = build_menu(&handle)?;
            let _tray = tauri::tray::TrayIconBuilder::with_id("main")
                .icon(tray_image())
                .menu(&menu)
                .tooltip("Dino Overlay")
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            if w.is_visible().unwrap_or(false) {
                                let _ = w.set_focus();
                            } else {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            let gs = handle.global_shortcut();
            let _ = gs.on_shortcut("CommandOrControl+Q", |app, _sc, ev| {
                if ev.state == ShortcutState::Pressed {
                    app.exit(0);
                }
            });
            let _ = gs.on_shortcut("CommandOrControl+Shift+P", |app, _sc, ev| {
                if ev.state == ShortcutState::Pressed {
                    toggle_on_top(app);
                }
            });

            Ok(())
        })
        .on_menu_event(|app: &AppHandle, event: MenuEvent| {
            let id = event.id().as_ref().to_string();
            if id == "quit" {
                app.exit(0);
            } else if id == "top" {
                toggle_on_top(app);
            } else if id == "hide" {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.hide();
                }
            } else if let Some(rest) = id.strip_prefix("op:") {
                if let Ok(v) = rest.parse::<f64>() {
                    apply_opacity(app, v);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            dino_quit,
            dino_hide,
            dino_toggle_ontop,
            dino_drag_start,
            dino_drag_end,
            dino_save_high_score,
            dino_show_context_menu
        ])
        .run(tauri::generate_context!())
        .expect("error while running DinoOverlay");
}
