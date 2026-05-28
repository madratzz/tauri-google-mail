use std::sync::{atomic::{AtomicU64, Ordering}, Mutex};

use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    webview::{NewWindowResponse, PageLoadEvent, WebviewBuilder},
    LogicalPosition, LogicalSize, Manager, WebviewUrl, WebviewWindowBuilder,
};

// Stores the URL of the active peek overlay so expand_peek can read it
// reliably. On Windows/WebView2, peek.url() may return the sentinel URL
// by the time expand_peek is called because navigation cancel is async.
struct PeekUrl(Mutex<Option<tauri::Url>>);

const GMAIL_URL: &str = "https://mail.google.com/";
const SAFARI_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.4 Safari/605.1.15";
const COLOR_ICON: &[u8] = include_bytes!("../icons/gmail-color.png");
const DARK_ICON: &[u8] = include_bytes!("../icons/gmail-dark.png");
const WHITE_ICON: &[u8] = include_bytes!("../icons/gmail-white.png");
static CHILD_WINDOW_COUNTER: AtomicU64 = AtomicU64::new(1);

/// JavaScript injected into peek webviews to render a floating toolbar with
/// "Pop Out" and "Close" buttons. Buttons navigate to sentinel URLs that
/// `on_navigation` intercepts on the Rust side.
const PEEK_TOOLBAR_JS: &str = r#"
(function() {
    function inject() {
        if (document.getElementById('_peek_toolbar') || !document.body) return;
        var bar = document.createElement('div');
        bar.id = '_peek_toolbar';
        bar.style.cssText = 'position:fixed;bottom:0;left:0;right:0;height:44px;background:rgba(32,33,36,0.96);backdrop-filter:blur(16px);-webkit-backdrop-filter:blur(16px);display:flex;align-items:center;justify-content:flex-end;padding:0 14px;gap:10px;z-index:2147483647;border-top:1px solid rgba(255,255,255,0.08);font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",system-ui,sans-serif;box-sizing:border-box;';

        var expandBtn = document.createElement('button');
        expandBtn.textContent = '↗ Pop Out';
        expandBtn.style.cssText = 'background:#ea4335;color:#fff;border:none;padding:7px 16px;border-radius:20px;cursor:pointer;font-size:12px;font-weight:600;letter-spacing:0.3px;transition:all 0.15s ease;outline:none;';
        expandBtn.onmouseenter = function() { this.style.background='#c5221f'; this.style.transform='scale(1.04)'; };
        expandBtn.onmouseleave = function() { this.style.background='#ea4335'; this.style.transform='scale(1)'; };
        expandBtn.onclick = function(e) { e.preventDefault(); window.location.href='https://peek-action.tauri.internal/expand'; };

        var closeBtn = document.createElement('button');
        closeBtn.textContent = '✕ Close';
        closeBtn.style.cssText = 'background:rgba(255,255,255,0.08);color:#e8eaed;border:1px solid rgba(255,255,255,0.1);padding:7px 16px;border-radius:20px;cursor:pointer;font-size:12px;font-weight:500;letter-spacing:0.3px;transition:all 0.15s ease;outline:none;';
        closeBtn.onmouseenter = function() { this.style.background='rgba(234,67,53,0.8)'; this.style.borderColor='rgba(234,67,53,0.6)'; };
        closeBtn.onmouseleave = function() { this.style.background='rgba(255,255,255,0.08)'; this.style.borderColor='rgba(255,255,255,0.1)'; };
        closeBtn.onclick = function(e) { e.preventDefault(); window.location.href='https://peek-action.tauri.internal/close'; };

        bar.appendChild(expandBtn);
        bar.appendChild(closeBtn);
        document.body.appendChild(bar);
        document.body.style.paddingBottom = '44px';
    }
    if (document.readyState === 'loading') { document.addEventListener('DOMContentLoaded', inject); }
    inject();
    window.addEventListener('load', inject);
})();
"#;

pub fn run() {
    tauri::Builder::default()
        .manage(PeekUrl(Mutex::new(None)))
        .setup(|app| {
            let app_handle = app.handle().clone();
            let app_resize = app.handle().clone();

            WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::External(GMAIL_URL.parse().expect("valid Gmail URL")),
            )
            .title("Gmail")
            .inner_size(1280.0, 860.0)
            .min_inner_size(960.0, 640.0)
            .resizable(true)
            .user_agent(SAFARI_USER_AGENT)
            .on_new_window(move |url, _| {
                create_peek_overlay(&app_handle, url);
                NewWindowResponse::Deny
            })
            .build()?;

            if let Some(main_win) = app.get_window("main") {
                main_win.on_window_event(move |event| {
                    if matches!(event, tauri::WindowEvent::Resized(_)) {
                        reposition_peek(&app_resize);
                    }
                });
            }

            let reload = MenuItem::with_id(app, "reload", "Reload", true, Some("CmdOrCtrl+R"))?;
            let back = MenuItem::with_id(app, "back", "Back", true, Some("CmdOrCtrl+["))?;
            let forward = MenuItem::with_id(app, "forward", "Forward", true, Some("CmdOrCtrl+]"))?;
            let compose = MenuItem::with_id(app, "compose", "Compose", true, Some("CmdOrCtrl+N"))?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, Some("CmdOrCtrl+Q"))?;
            let icon_color = MenuItem::with_id(app, "icon-color", "Color", true, None::<&str>)?;
            let icon_dark = MenuItem::with_id(app, "icon-dark", "Dark", true, None::<&str>)?;
            let icon_white = MenuItem::with_id(app, "icon-white", "White", true, None::<&str>)?;
            let sep1 = PredefinedMenuItem::separator(app)?;
            let sep2 = PredefinedMenuItem::separator(app)?;
            let sep3 = PredefinedMenuItem::separator(app)?;
            let edit_undo = PredefinedMenuItem::undo(app, None)?;
            let edit_redo = PredefinedMenuItem::redo(app, None)?;
            let edit_sep1 = PredefinedMenuItem::separator(app)?;
            let edit_cut = PredefinedMenuItem::cut(app, None)?;
            let edit_copy = PredefinedMenuItem::copy(app, None)?;
            let edit_paste = PredefinedMenuItem::paste(app, None)?;
            let edit_sep2 = PredefinedMenuItem::separator(app)?;
            let edit_select_all = PredefinedMenuItem::select_all(app, None)?;

            let app_menu = Submenu::with_items(
                app,
                "Gmail",
                true,
                &[
                    &compose,
                    &sep1,
                    &reload,
                    &sep2,
                    &back,
                    &forward,
                    &sep3,
                    &quit,
                ],
            )?;

            let edit_menu = Submenu::with_items(
                app,
                "Edit",
                true,
                &[
                    &edit_undo,
                    &edit_redo,
                    &edit_sep1,
                    &edit_cut,
                    &edit_copy,
                    &edit_paste,
                    &edit_sep2,
                    &edit_select_all,
                ],
            )?;

            let icon_menu =
                Submenu::with_items(app, "Icon", true, &[&icon_color, &icon_dark, &icon_white])?;

            let menu = Menu::with_items(app, &[&app_menu, &edit_menu, &icon_menu])?;
            app.set_menu(menu)?;

            Ok(())
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            "reload" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.eval("window.location.reload()");
                }
            }
            "back" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.eval("history.back()");
                }
            }
            "forward" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.eval("history.forward()");
                }
            }
            "compose" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.eval(
                        "window.location.href='https://mail.google.com/mail/u/0/#compose'",
                    );
                }
            }
            "icon-color" => set_main_window_icon(app, COLOR_ICON),
            "icon-dark" => set_main_window_icon(app, DARK_ICON),
            "icon-white" => set_main_window_icon(app, WHITE_ICON),
            "quit" => app.exit(0),
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("failed to run Gmail desktop app");
}

/// Creates a child webview overlay inside the main window for links that open
/// in new tabs (e.g. email drafts, Google Docs links, Meet calls).
fn create_peek_overlay(app: &tauri::AppHandle, url: tauri::Url) {
    if let Some(existing) = app.get_webview("peek") {
        let _ = existing.close();
    }

    // Store URL in state before creating the webview so expand_peek can
    // read the original URL even if peek.url() has drifted on Windows.
    if let Ok(mut guard) = app.state::<PeekUrl>().0.lock() {
        *guard = Some(url.clone());
    }

    let Some(main_window) = app.get_window("main") else {
        return;
    };
    let (size, pos) = peek_size_and_position_logical(app);

    let app_for_nav = app.clone();

    let builder = WebviewBuilder::new("peek", WebviewUrl::External(url))
        .user_agent(SAFARI_USER_AGENT)
        .on_navigation(move |nav_url| {
            if nav_url.host_str() == Some("peek-action.tauri.internal") {
                match nav_url.path() {
                    "/expand" => expand_peek(&app_for_nav),
                    "/close" => close_peek(&app_for_nav),
                    _ => {}
                }
                return false;
            }
            true
        })
        .on_page_load(|webview, payload| {
            if matches!(payload.event(), PageLoadEvent::Finished) {
                let _ = webview.eval(PEEK_TOOLBAR_JS);
            }
        });

    match main_window.add_child(builder, pos, size) {
        Ok(ref peek) => {
            let _ = peek.eval(PEEK_TOOLBAR_JS);
        }
        Err(e) => eprintln!("failed to create peek overlay: {e}"),
    }
}

fn expand_peek(app: &tauri::AppHandle) {
    // Read the URL from stored state — do NOT use peek.url() here.
    // On Windows/WebView2, on_navigation cancel is asynchronous, so peek.url()
    // may already reflect the sentinel URL by the time this function runs,
    // producing a blank white window.
    let url = {
        let state = app.state::<PeekUrl>();
        let guard = state.0.lock().unwrap();
        match guard.clone() {
            Some(u) => u,
            None => return,
        }
    };

    if let Some(peek) = app.get_webview("peek") {
        let _ = peek.close();
    }

    let label = child_window_label(&url);
    let app_clone = app.clone();

    let result = WebviewWindowBuilder::new(app, label, WebviewUrl::External(url.clone()))
        .title(title_for_url(&url))
        .inner_size(1180.0, 820.0)
        .min_inner_size(820.0, 560.0)
        .resizable(true)
        .focused(true)
        .user_agent(SAFARI_USER_AGENT)
        // Suppress beforeunload dialogs so the close button always works on Windows.
        .initialization_script(
            "window.addEventListener('beforeunload',function(e){delete e.returnValue;},true);",
        )
        .on_new_window(move |new_url, _| {
            // Open nested links as additional standalone windows. Do NOT call
            // win.navigate() from inside this callback — on Windows/WebView2
            // that corrupts the webview navigation state and breaks the close button.
            open_standalone_window(&app_clone, new_url);
            NewWindowResponse::Deny
        })
        .build();

    // Force-close on X so Windows' native close button always works even if
    // WebView2 tries to re-raise a beforeunload confirmation.
    if let Ok(win) = result {
        let win_clone = win.clone();
        win.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = win_clone.destroy();
            }
        });
    }
}

/// Opens a URL in a fresh standalone window. Used by expanded windows when
/// they encounter a link that would open a new tab.
fn open_standalone_window(app: &tauri::AppHandle, url: tauri::Url) {
    let label = child_window_label(&url);
    let app_clone = app.clone();
    let result = WebviewWindowBuilder::new(app, label, WebviewUrl::External(url.clone()))
        .title(title_for_url(&url))
        .inner_size(1180.0, 820.0)
        .min_inner_size(820.0, 560.0)
        .resizable(true)
        .focused(true)
        .user_agent(SAFARI_USER_AGENT)
        .initialization_script(
            "window.addEventListener('beforeunload',function(e){delete e.returnValue;},true);",
        )
        .on_new_window(move |new_url, _| {
            open_standalone_window(&app_clone, new_url);
            NewWindowResponse::Deny
        })
        .build();

    if let Ok(win) = result {
        let win_clone = win.clone();
        win.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = win_clone.destroy();
            }
        });
    }
}

fn close_peek(app: &tauri::AppHandle) {
    if let Some(peek) = app.get_webview("peek") {
        let _ = peek.close();
    }
}

fn reposition_peek(app: &tauri::AppHandle) {
    let Some(peek) = app.get_webview("peek") else {
        return;
    };
    let (size, pos) = peek_size_and_position_logical(app);
    let _ = peek.set_size(size);
    let _ = peek.set_position(pos);
}

fn peek_size_and_position_logical(
    app: &tauri::AppHandle,
) -> (LogicalSize<f64>, LogicalPosition<f64>) {
    let Some(main_window) = app.get_window("main") else {
        return (
            LogicalSize::new(800.0, 600.0),
            LogicalPosition::new(100.0, 100.0),
        );
    };
    let Ok(phys_size) = main_window.inner_size() else {
        return (
            LogicalSize::new(800.0, 600.0),
            LogicalPosition::new(100.0, 100.0),
        );
    };
    let scale = main_window.scale_factor().unwrap_or(1.0);
    let logical_w = phys_size.width as f64 / scale;
    let logical_h = phys_size.height as f64 / scale;
    let w = logical_w * 0.85;
    let h = logical_h * 0.85;
    let x = (logical_w - w) / 2.0;
    let y = (logical_h - h) / 2.0;
    (LogicalSize::new(w, h), LogicalPosition::new(x, y))
}

fn set_main_window_icon(app: &tauri::AppHandle, icon_bytes: &[u8]) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if let Ok(icon) = Image::from_bytes(icon_bytes) {
        let _ = window.set_icon(icon);
    }
    #[cfg(target_os = "macos")]
    set_macos_dock_icon(icon_bytes);
}

#[cfg(target_os = "macos")]
fn set_macos_dock_icon(icon_bytes: &[u8]) {
    use cocoa::base::{id, nil};
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let nsdata_class = objc::runtime::Class::get("NSData").unwrap();
        let data: id =
            msg_send![nsdata_class, dataWithBytes: icon_bytes.as_ptr() length: icon_bytes.len()];
        let ns_image_class = objc::runtime::Class::get("NSImage").unwrap();
        let alloc_image: id = msg_send![ns_image_class, alloc];
        let image: id = msg_send![alloc_image, initWithData: data];
        if image != nil {
            let ns_app_class = objc::runtime::Class::get("NSApplication").unwrap();
            let app: id = msg_send![ns_app_class, sharedApplication];
            let _: () = msg_send![app, setApplicationIconImage: image];
        }
    }
}

fn child_window_label(url: &tauri::Url) -> String {
    let id = CHILD_WINDOW_COUNTER.fetch_add(1, Ordering::Relaxed);
    let sanitized: String = url
        .as_str()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .take(80)
        .collect();
    format!("expanded-{id}-{sanitized}")
}

fn title_for_url(url: &tauri::Url) -> &'static str {
    match url.host_str() {
        Some("mail.google.com" | "gmail.com") => "Gmail",
        Some("docs.google.com") => "Google Docs",
        Some("drive.google.com") => "Google Drive",
        Some("calendar.google.com") => "Google Calendar",
        Some("meet.google.com") => "Google Meet",
        Some("contacts.google.com") => "Google Contacts",
        Some("keep.google.com") => "Google Keep",
        Some("tasks.google.com") => "Google Tasks",
        Some("chat.google.com") => "Google Chat",
        _ => "Google Workspace",
    }
}
