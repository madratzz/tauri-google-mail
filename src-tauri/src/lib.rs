use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Mutex,
};

use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    webview::{DownloadEvent, NewWindowResponse, PageLoadEvent, WebviewBuilder},
    window::Color,
    AppHandle, LogicalPosition, LogicalSize, Manager, RunEvent, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder, WindowEvent,
};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_window_state::StateFlags;

// Stores the URL of the active peek overlay so expand_peek can read it
// reliably. On Windows/WebView2, peek.url() may return the sentinel URL
// by the time expand_peek is called because navigation cancel is async.
struct PeekUrl(Mutex<Option<tauri::Url>>);

/// Runtime UI state for the main window.
struct AppState {
    /// Last unread count parsed from Gmail's document title.
    unread: Mutex<Option<u32>>,
    /// Current zoom factor of the main webview.
    zoom: Mutex<f64>,
    /// Set once the main window has been shown after the first page load.
    shown: AtomicBool,
    /// Set once Gmail has sent its own (richer) notification; after that the
    /// count-based fallback notification is skipped to avoid duplicates.
    web_notifications: AtomicBool,
    /// Set when the user chose Quit, so close-to-tray does not intercept it.
    quitting: AtomicBool,
}

const MAIN: &str = "main";
const PEEK: &str = "peek";
const GMAIL_URL: &str = "https://mail.google.com/";
const SAFARI_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.4 Safari/605.1.15";
const COLOR_ICON: &[u8] = include_bytes!("../icons/gmail-color.png");
const DARK_ICON: &[u8] = include_bytes!("../icons/gmail-dark.png");
const WHITE_ICON: &[u8] = include_bytes!("../icons/gmail-white.png");
/// Monochrome menu-bar icon; macOS tints template images white or black to
/// match the menu bar.
#[cfg(target_os = "macos")]
const TRAY_TEMPLATE_ICON: &[u8] = include_bytes!("../icons/tray-template.png");
/// Gmail's light background; shown instead of a white flash while loading.
const BACKGROUND: Color = Color(246, 248, 252, 255);
const ZOOM_STEP: f64 = 0.1;
const ZOOM_MIN: f64 = 0.5;
const ZOOM_MAX: f64 = 3.0;
static CHILD_WINDOW_COUNTER: AtomicU64 = AtomicU64::new(1);

/// JavaScript injected into peek webviews to render a floating toolbar with
/// "Pop Out" and "Close" buttons. Buttons (and the Escape key) navigate to
/// sentinel URLs that `on_navigation` intercepts on the Rust side.
const PEEK_TOOLBAR_JS: &str = r#"
(function() {
    function go(action) { window.location.href = 'https://peek-action.tauri.internal/' + action; }
    if (!window._peekKeys) {
        window._peekKeys = true;
        document.addEventListener('keydown', function(e) {
            if (e.key === 'Escape' && !e.defaultPrevented) { go('close'); }
        });
    }
    function inject() {
        if (document.getElementById('_peek_toolbar') || !document.body) return;
        var style = document.createElement('style');
        style.textContent =
            '#_peek_toolbar{position:fixed;bottom:0;left:0;right:0;height:52px;display:flex;align-items:center;justify-content:flex-end;gap:10px;padding:0 16px;z-index:2147483647;box-sizing:border-box;font:500 13px/1 -apple-system,BlinkMacSystemFont,"Segoe UI",system-ui,sans-serif;background:rgba(255,255,255,0.94);border-top:1px solid rgba(0,0,0,0.12);backdrop-filter:blur(16px);-webkit-backdrop-filter:blur(16px);}' +
            '#_peek_toolbar span{margin-right:auto;color:#5f6368;font-size:12px;}' +
            '#_peek_toolbar button{min-height:36px;padding:0 18px;border-radius:18px;cursor:pointer;font:inherit;transition:background .15s ease;}' +
            '#_peek_toolbar button:focus-visible{outline:2px solid #1a73e8;outline-offset:2px;}' +
            '#_peek_pop{background:#0b57d0;color:#fff;border:none;}#_peek_pop:hover{background:#0842a0;}' +
            '#_peek_close{background:transparent;color:#1f1f1f;border:1px solid rgba(0,0,0,0.2);}#_peek_close:hover{background:rgba(0,0,0,0.06);}' +
            '@media (prefers-color-scheme: dark){#_peek_toolbar{background:rgba(32,33,36,0.96);border-top-color:rgba(255,255,255,0.1);}#_peek_toolbar span{color:#9aa0a6;}#_peek_pop{background:#a8c7fa;color:#062e6f;}#_peek_pop:hover{background:#d3e3fd;}#_peek_close{color:#e8eaed;border-color:rgba(255,255,255,0.25);}#_peek_close:hover{background:rgba(255,255,255,0.08);}}' +
            '@media (prefers-reduced-motion: reduce){#_peek_toolbar button{transition:none;}}';
        document.head && document.head.appendChild(style);
        var bar = document.createElement('div');
        bar.id = '_peek_toolbar';
        bar.setAttribute('role', 'toolbar');
        var hint = document.createElement('span');
        hint.textContent = 'Esc to close';
        var pop = document.createElement('button');
        pop.id = '_peek_pop';
        pop.textContent = '↗ Pop out';
        pop.onclick = function(e) { e.preventDefault(); go('expand'); };
        var close = document.createElement('button');
        close.id = '_peek_close';
        close.textContent = '✕ Close';
        close.onclick = function(e) { e.preventDefault(); go('close'); };
        bar.appendChild(hint);
        bar.appendChild(pop);
        bar.appendChild(close);
        document.body.appendChild(bar);
        document.body.style.paddingBottom = '52px';
    }
    if (document.readyState === 'loading') { document.addEventListener('DOMContentLoaded', inject); }
    inject();
    window.addEventListener('load', inject);
})();
"#;

/// Injected into the main window: shows a small banner while offline and
/// reloads once the connection comes back.
const OFFLINE_JS: &str = r#"
(function() {
    var id = '_gd_offline';
    function show() {
        if (document.getElementById(id) || !document.body) return;
        var b = document.createElement('div');
        b.id = id;
        b.setAttribute('role', 'status');
        b.textContent = 'You are offline. Gmail will reload when the connection returns.';
        b.style.cssText = 'position:fixed;top:0;left:0;right:0;z-index:2147483647;padding:8px 16px;text-align:center;font:500 13px system-ui,sans-serif;background:#fce8b2;color:#3c2a00;';
        document.body.appendChild(b);
    }
    window.addEventListener('offline', show);
    window.addEventListener('online', function() { window.location.reload(); });
    if (!navigator.onLine) { document.addEventListener('DOMContentLoaded', show); }
})();
"#;

/// Gmail asks for web-notification permission, but WKWebView has no
/// Notification API, so the "OK" button does nothing. This shim reports the
/// permission as granted and forwards each notification Gmail creates to the
/// native `notify` command. Only `https://mail.google.com` may call it (see
/// capabilities/gmail-notifications.json).
const NOTIFICATION_SHIM_JS: &str = r#"
(function() {
    if (window.location.hostname !== 'mail.google.com') return;
    function send(title, body) {
        var ipc = window.__TAURI_INTERNALS__;
        if (!ipc || typeof ipc.invoke !== 'function') return;
        ipc.invoke('notify', { title: String(title || 'Gmail'), body: String(body || '') }).catch(function() {});
    }
    function GmailNotification(title, options) {
        options = options || {};
        this.title = title;
        this.body = options.body || '';
        this.tag = options.tag || '';
        this.onclick = null; this.onclose = null; this.onerror = null; this.onshow = null;
        send(title, this.body);
    }
    GmailNotification.prototype.close = function() {};
    GmailNotification.prototype.addEventListener = function() {};
    GmailNotification.prototype.removeEventListener = function() {};
    Object.defineProperty(GmailNotification, 'permission', { get: function() { return 'granted'; } });
    GmailNotification.requestPermission = function(callback) {
        if (typeof callback === 'function') callback('granted');
        return Promise.resolve('granted');
    };
    window.Notification = GmailNotification;
    if (navigator.permissions && navigator.permissions.query) {
        var query = navigator.permissions.query.bind(navigator.permissions);
        navigator.permissions.query = function(desc) {
            if (desc && desc.name === 'notifications') {
                return Promise.resolve({ state: 'granted', status: 'granted', onchange: null,
                    addEventListener: function() {}, removeEventListener: function() {} });
            }
            return query(desc);
        };
    }
})();
"#;

/// Opens compose in the account currently shown instead of always `u/0`.
const COMPOSE_JS: &str = r#"
(function() {
    var m = window.location.pathname.match(/^\/mail\/u\/\d+\//);
    if (window.location.hostname === 'mail.google.com' && m) { window.location.hash = 'compose'; }
    else { window.location.href = 'https://mail.google.com/mail/u/0/#compose'; }
})();
"#;

/// Initialization script for pop-out windows on Windows only: WebView2 can
/// get stuck re-raising `beforeunload` prompts, which breaks the close button
/// (see docs/windows-popup-bugs.md). Other platforms keep Gmail's
/// unsaved-changes guard.
#[cfg(windows)]
const POPOUT_INIT_JS: &str =
    "window.addEventListener('beforeunload',function(e){delete e.returnValue;},true);";

pub fn run() {
    let app = tauri::Builder::default()
        // Must be registered first so a second launch (e.g. a mailto: click)
        // is forwarded to the running instance.
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            for arg in args.iter().skip(1) {
                if let Ok(url) = arg.parse::<tauri::Url>() {
                    handle_open_url(app, &url);
                }
            }
            show_main(app);
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(StateFlags::all() & !StateFlags::VISIBLE)
                .with_filter(|label| label == MAIN)
                .build(),
        )
        .manage(PeekUrl(Mutex::new(None)))
        .manage(AppState {
            unread: Mutex::new(None),
            zoom: Mutex::new(1.0),
            shown: AtomicBool::new(false),
            web_notifications: AtomicBool::new(false),
            quitting: AtomicBool::new(false),
        })
        .setup(|app| {
            let handle = app.handle().clone();
            build_main_window(&handle)?;
            build_menu(&handle)?;
            build_tray(&handle)?;

            let dl = handle.clone();
            app.deep_link().on_open_url(move |event| {
                for url in event.urls() {
                    handle_open_url(&dl, &url);
                }
                show_main(&dl);
            });
            if let Ok(Some(urls)) = app.deep_link().get_current() {
                for url in urls {
                    handle_open_url(&handle, &url);
                }
            }

            restore_icon_choice(&handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![notify])
        .on_menu_event(|app, event| handle_menu(app, event.id().as_ref()))
        .build(tauri::generate_context!())
        .expect("failed to build Gmail desktop app");

    app.run(|app, event| {
        #[cfg(target_os = "macos")]
        if let RunEvent::Reopen { .. } = event {
            show_main(app);
        }
        if let RunEvent::ExitRequested { .. } = event {
            app.state::<AppState>()
                .quitting
                .store(true, Ordering::SeqCst);
        }
        let _ = app;
    });
}

fn build_main_window(app: &AppHandle) -> tauri::Result<()> {
    let new_window_app = app.clone();
    let nav_app = app.clone();
    let title_app = app.clone();

    let window = WebviewWindowBuilder::new(
        app,
        MAIN,
        WebviewUrl::External(GMAIL_URL.parse().expect("valid Gmail URL")),
    )
    .title("Gmail")
    .inner_size(1280.0, 860.0)
    .min_inner_size(720.0, 500.0)
    .center()
    .resizable(true)
    // Stay hidden until Gmail has painted to avoid a blank flash.
    .visible(false)
    .background_color(BACKGROUND)
    .user_agent(SAFARI_USER_AGENT)
    .initialization_script(OFFLINE_JS)
    .initialization_script(NOTIFICATION_SHIM_JS)
    .on_navigation(move |url| {
        if url.scheme() == "mailto" {
            let app = nav_app.clone();
            let url = url.clone();
            // Never touch windows directly inside on_navigation (WebView2 deadlock).
            tauri::async_runtime::spawn(async move { handle_open_url(&app, &url) });
            return false;
        }
        true
    })
    .on_new_window(move |url, _| {
        route_new_window(&new_window_app, url);
        NewWindowResponse::Deny
    })
    .on_document_title_changed(move |_, title| update_unread(&title_app, &title))
    .on_download(download_handler)
    .on_page_load(|webview, payload| {
        if matches!(payload.event(), PageLoadEvent::Finished) {
            let app = webview.app_handle();
            if !app.state::<AppState>().shown.swap(true, Ordering::SeqCst) {
                show_main(app);
            }
        }
    })
    .build()?;

    // Fallback: show the window even if the first load never finishes
    // (e.g. no network), so the user is never left without a window.
    let fallback = app.clone();
    tauri::async_runtime::spawn(async move {
        sleep_async(std::time::Duration::from_secs(6)).await;
        if !fallback
            .state::<AppState>()
            .shown
            .swap(true, Ordering::SeqCst)
        {
            show_main(&fallback);
        }
    });

    let event_app = app.clone();
    window.on_window_event(move |event| match event {
        WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
            reposition_peek(&event_app)
        }
        WindowEvent::CloseRequested { api, .. }
            // Close to tray; Quit (menu, tray or Cmd+Q) exits for real.
            if !event_app
                .state::<AppState>()
                .quitting
                .load(Ordering::SeqCst)
            => {
                api.prevent_close();
                if let Some(w) = event_app.get_webview_window(MAIN) {
                    let _ = w.hide();
                }
            }
        _ => {}
    });
    Ok(())
}

/// Async sleep on a blocking thread, avoiding a direct tokio dependency.
async fn sleep_async(duration: std::time::Duration) {
    let _ = tauri::async_runtime::spawn_blocking(move || std::thread::sleep(duration)).await;
}

fn build_menu(app: &AppHandle) -> tauri::Result<()> {
    let item =
        |id: &str, text: &str, accel: Option<&str>| MenuItem::with_id(app, id, text, true, accel);
    let sep = || PredefinedMenuItem::separator(app);

    let app_menu = Submenu::with_items(
        app,
        "Gmail",
        true,
        &[
            &item("compose", "Compose", Some("CmdOrCtrl+N"))?,
            &sep()?,
            &item("reload", "Reload", Some("CmdOrCtrl+R"))?,
            &sep()?,
            &item("back", "Back", Some("CmdOrCtrl+["))?,
            &item("forward", "Forward", Some("CmdOrCtrl+]"))?,
            &sep()?,
            &item("quit", "Quit", Some("CmdOrCtrl+Q"))?,
        ],
    )?;
    let edit_menu = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(app, None)?,
            &PredefinedMenuItem::redo(app, None)?,
            &sep()?,
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &sep()?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;
    let view_menu = Submenu::with_items(
        app,
        "View",
        true,
        &[
            &item("zoom-in", "Zoom In", Some("CmdOrCtrl+="))?,
            &item("zoom-out", "Zoom Out", Some("CmdOrCtrl+-"))?,
            &item("zoom-reset", "Actual Size", Some("CmdOrCtrl+0"))?,
        ],
    )?;
    let window_menu = Submenu::with_items(
        app,
        "Window",
        true,
        &[
            &PredefinedMenuItem::minimize(app, None)?,
            &PredefinedMenuItem::close_window(app, None)?,
            &PredefinedMenuItem::fullscreen(app, None)?,
        ],
    )?;
    let icon_menu = Submenu::with_items(
        app,
        "Icon",
        true,
        &[
            &item("icon-color", "Color", None)?,
            &item("icon-dark", "Dark", None)?,
            &item("icon-white", "White", None)?,
        ],
    )?;

    let menu = Menu::with_items(
        app,
        &[&app_menu, &edit_menu, &view_menu, &window_menu, &icon_menu],
    )?;
    app.set_menu(menu)?;
    Ok(())
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, "show", "Show Gmail", true, None::<&str>)?,
            &MenuItem::with_id(app, "compose", "Compose", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?,
        ],
    )?;
    #[cfg(target_os = "macos")]
    let builder = TrayIconBuilder::with_id("tray")
        .icon(Image::from_bytes(TRAY_TEMPLATE_ICON)?)
        .icon_as_template(true);
    // Windows/Linux taskbars may be light, where a white icon would vanish.
    #[cfg(not(target_os = "macos"))]
    let builder = TrayIconBuilder::with_id("tray").icon(Image::from_bytes(COLOR_ICON)?);
    builder
        .tooltip("Gmail")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_main(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

fn handle_menu(app: &AppHandle, id: &str) {
    let main = app.get_webview_window(MAIN);
    match id {
        "reload" => eval_main(main, "window.location.reload()"),
        "back" => eval_main(main, "history.back()"),
        "forward" => eval_main(main, "history.forward()"),
        "compose" => {
            show_main(app);
            eval_main(main, COMPOSE_JS);
        }
        "zoom-in" => change_zoom(app, Some(ZOOM_STEP)),
        "zoom-out" => change_zoom(app, Some(-ZOOM_STEP)),
        "zoom-reset" => change_zoom(app, None),
        "show" => show_main(app),
        "icon-color" | "icon-dark" | "icon-white" => {
            set_main_window_icon(app, icon_bytes(id));
            save_icon_choice(app, id);
        }
        "quit" => {
            app.state::<AppState>()
                .quitting
                .store(true, Ordering::SeqCst);
            app.exit(0);
        }
        _ => {}
    }
}

fn eval_main(main: Option<WebviewWindow>, js: &str) {
    if let Some(w) = main {
        let _ = w.eval(js);
    }
}

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(MAIN) {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

fn toggle_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(MAIN) {
        if w.is_visible().unwrap_or(false) && w.is_focused().unwrap_or(false) {
            let _ = w.hide();
        } else {
            show_main(app);
        }
    }
}

fn change_zoom(app: &AppHandle, delta: Option<f64>) {
    let state = app.state::<AppState>();
    let Ok(mut zoom) = state.zoom.lock() else {
        return;
    };
    *zoom = match delta {
        Some(d) => ((*zoom + d) * 10.0).round() / 10.0,
        None => 1.0,
    }
    .clamp(ZOOM_MIN, ZOOM_MAX);
    if let Some(w) = app.get_webview_window(MAIN) {
        let _ = w.set_zoom(*zoom);
    }
}

/// Google-owned hosts stay inside the app; everything else opens in the
/// user's default browser, where they have an address bar and their own
/// security tooling.
fn is_google_host(url: &tauri::Url) -> bool {
    matches!(url.scheme(), "http" | "https")
        && url.host_str().is_some_and(|h| {
            h == "google.com"
                || h.ends_with(".google.com")
                || h == "gmail.com"
                || h.ends_with(".gmail.com")
        })
}

/// Gmail wraps outbound links as `https://www.google.com/url?q=<target>`;
/// unwrap them so the real destination decides where the link opens.
fn unwrap_redirect(url: &tauri::Url) -> tauri::Url {
    if matches!(url.host_str(), Some("www.google.com" | "google.com")) && url.path() == "/url" {
        if let Some(target) = url
            .query_pairs()
            .find(|(k, _)| k == "q" || k == "url")
            .and_then(|(_, v)| v.parse::<tauri::Url>().ok())
        {
            return target;
        }
    }
    url.clone()
}

fn route_new_window(app: &AppHandle, url: tauri::Url) {
    let target = unwrap_redirect(&url);
    if target.scheme() == "mailto" {
        handle_open_url(app, &target);
    } else if is_google_host(&target) {
        create_peek_overlay(app, url);
    } else if matches!(target.scheme(), "http" | "https") {
        if let Err(e) = app.opener().open_url(target.as_str(), None::<&str>) {
            eprintln!("failed to open link in browser: {e}");
        }
    }
}

/// Handles URLs handed to the app (mailto deep links, second-instance args).
fn handle_open_url(app: &AppHandle, url: &tauri::Url) {
    if url.scheme() != "mailto" {
        return;
    }
    let compose = mailto_compose_url(url);
    if let Some(w) = app.get_webview_window(MAIN) {
        if let Ok(parsed) = compose.parse() {
            let _ = w.navigate(parsed);
        }
    }
    show_main(app);
}

fn mailto_compose_url(mailto: &tauri::Url) -> String {
    let encoded: String =
        url::form_urlencoded::byte_serialize(mailto.as_str().as_bytes()).collect();
    format!("https://mail.google.com/mail/?extsrc=mailto&url={encoded}")
}

/// Parses the unread count from Gmail's title, e.g. "Inbox (3) - … - Gmail".
fn parse_unread(title: &str) -> Option<u32> {
    let start = title.find(" (")? + 2;
    let rest = &title[start..];
    let end = rest.find(')')?;
    rest[..end].replace([',', '.', '\u{a0}'], "").parse().ok()
}

fn update_unread(app: &AppHandle, title: &str) {
    // Gmail's title includes the account address; keep it but only surface
    // the count in the badge, tray and notification.
    let count = parse_unread(title).unwrap_or(0);
    let state = app.state::<AppState>();
    let previous = {
        let Ok(mut guard) = state.unread.lock() else {
            return;
        };
        guard.replace(count)
    };
    if previous == Some(count) {
        return;
    }

    if let Some(w) = app.get_webview_window(MAIN) {
        let _ = w.set_title(&if count > 0 {
            format!("Gmail ({count})")
        } else {
            "Gmail".into()
        });
        // Supported on macOS and some Linux desktops; ignored elsewhere.
        let _ = w.set_badge_count((count > 0).then_some(count as i64));

        let focused = w.is_focused().unwrap_or(false);
        if let Some(prev) = previous {
            let gmail_notifies = app
                .state::<AppState>()
                .web_notifications
                .load(Ordering::SeqCst);
            if count > prev && !focused && !gmail_notifies {
                let new = count - prev;
                let _ = app
                    .notification()
                    .builder()
                    .title("Gmail")
                    .body(if new == 1 {
                        "1 new message".to_string()
                    } else {
                        format!("{new} new messages")
                    })
                    .show();
            }
        }
    }
    if let Some(tray) = app.tray_by_id("tray") {
        let tip = if count > 0 {
            format!("Gmail — {count} unread")
        } else {
            "Gmail".into()
        };
        let _ = tray.set_tooltip(Some(tip));
    }
}

/// Shows a native notification for a web notification raised by Gmail.
#[tauri::command]
fn notify(app: AppHandle, title: String, body: String) {
    app.state::<AppState>()
        .web_notifications
        .store(true, Ordering::SeqCst);
    let clip = |s: String, n: usize| s.chars().take(n).collect::<String>();
    let _ = app
        .notification()
        .builder()
        .title(clip(title, 200))
        .body(clip(body, 500))
        .show();
}

/// Saves downloads (attachments) to the user's Downloads folder. Without a
/// handler, WKWebView silently drops downloads.
fn download_handler(webview: tauri::Webview, event: DownloadEvent<'_>) -> bool {
    if let DownloadEvent::Requested { url, destination } = event {
        if let Ok(dir) = webview.app_handle().path().download_dir() {
            let name = destination
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .filter(|n| !n.is_empty())
                .or_else(|| {
                    url.path_segments()
                        .and_then(|mut s| s.next_back().map(str::to_owned))
                        .filter(|n| !n.is_empty())
                })
                .unwrap_or_else(|| "download".into());
            *destination = unique_path(dir.join(name));
        }
    }
    true
}

fn unique_path(path: std::path::PathBuf) -> std::path::PathBuf {
    if !path.exists() {
        return path;
    }
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    (1..1000)
        .map(|i| path.with_file_name(format!("{stem} ({i}){ext}")))
        .find(|p| !p.exists())
        .unwrap_or(path)
}

/// Creates a child webview overlay inside the main window for Google links
/// that open in new tabs (e.g. Docs, Drive, Calendar, Meet).
fn create_peek_overlay(app: &AppHandle, url: tauri::Url) {
    if let Some(existing) = app.get_webview(PEEK) {
        let _ = existing.close();
    }

    // Store URL in state before creating the webview so expand_peek can
    // read the original URL even if peek.url() has drifted on Windows.
    if let Ok(mut guard) = app.state::<PeekUrl>().0.lock() {
        *guard = Some(url.clone());
    }

    let Some(main_window) = app.get_window(MAIN) else {
        return;
    };
    let (size, pos) = peek_size_and_position_logical(app);

    let app_for_nav = app.clone();
    let app_for_new = app.clone();

    let builder = WebviewBuilder::new(PEEK, WebviewUrl::External(url))
        .user_agent(SAFARI_USER_AGENT)
        .background_color(BACKGROUND)
        .on_navigation(move |nav_url| {
            if nav_url.host_str() == Some("peek-action.tauri.internal") {
                // Spawn onto the async runtime so this callback returns immediately.
                // On Windows, on_navigation fires on the WebView2 background thread.
                // Calling peek.close() or WebviewWindowBuilder::new() directly here
                // deadlocks: those ops need the main thread, but the main thread is
                // waiting for this callback to return. macOS is unaffected because
                // WKWebView fires navigation callbacks on the main thread already.
                let app = app_for_nav.clone();
                match nav_url.path() {
                    "/expand" => {
                        tauri::async_runtime::spawn(async move { expand_peek(&app) });
                    }
                    "/close" => {
                        tauri::async_runtime::spawn(async move { close_peek(&app) });
                    }
                    _ => {}
                }
                return false;
            }
            true
        })
        .on_new_window(move |url, _| {
            let app = app_for_new.clone();
            tauri::async_runtime::spawn(async move { route_popout_link(&app, url) });
            NewWindowResponse::Deny
        })
        .on_download(download_handler)
        .on_page_load(|webview, payload| {
            if matches!(payload.event(), PageLoadEvent::Finished) {
                let _ = webview.eval(PEEK_TOOLBAR_JS);
            }
        });

    match main_window.add_child(builder, pos, size) {
        Ok(ref peek) => {
            let _ = peek.eval(PEEK_TOOLBAR_JS);
            let _ = peek.set_focus();
        }
        Err(e) => eprintln!("failed to create peek overlay: {e}"),
    }
}

fn expand_peek(app: &AppHandle) {
    // Read the URL from stored state — do NOT use peek.url() here.
    // On Windows/WebView2, on_navigation cancel is asynchronous, so peek.url()
    // may already reflect the sentinel URL by the time this function runs,
    // producing a blank white window.
    let url = {
        let state = app.state::<PeekUrl>();
        let Ok(guard) = state.0.lock() else {
            return;
        };
        match guard.clone() {
            Some(u) => u,
            None => return,
        }
    };

    close_peek(app);
    open_standalone_window(app, url);
}

/// Opens nested links from pop-outs: Google links get another window, other
/// sites go to the default browser.
fn route_popout_link(app: &AppHandle, url: tauri::Url) {
    let target = unwrap_redirect(&url);
    if target.scheme() == "mailto" {
        handle_open_url(app, &target);
    } else if is_google_host(&target) {
        open_standalone_window(app, url);
    } else if matches!(target.scheme(), "http" | "https") {
        let _ = app.opener().open_url(target.as_str(), None::<&str>);
    }
}

/// Opens a Google URL in a standalone window sized to fit the current monitor.
fn open_standalone_window(app: &AppHandle, url: tauri::Url) {
    let label = child_window_label(&url);
    let app_clone = app.clone();
    let (w, h) = popout_size(app);

    let builder = WebviewWindowBuilder::new(app, label, WebviewUrl::External(url.clone()))
        .title(title_for_url(&url))
        .inner_size(w, h)
        .min_inner_size(w.min(600.0), h.min(420.0))
        .center()
        .resizable(true)
        .focused(true)
        .background_color(BACKGROUND)
        .user_agent(SAFARI_USER_AGENT)
        .on_download(download_handler)
        .on_new_window(move |new_url, _| {
            // Do NOT call win.navigate() from inside this callback — on
            // Windows/WebView2 that corrupts the webview navigation state and
            // breaks the close button.
            route_popout_link(&app_clone, new_url);
            NewWindowResponse::Deny
        });

    #[cfg(windows)]
    let builder = builder.initialization_script(POPOUT_INIT_JS);

    let result = builder.build();

    // Windows only: force-close on X so the native close button always works
    // even if WebView2 tries to re-raise a beforeunload confirmation.
    #[cfg(windows)]
    if let Ok(win) = &result {
        let win_clone = win.clone();
        win.on_window_event(move |event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = win_clone.destroy();
            }
        });
    }
    if let Err(e) = result {
        eprintln!("failed to open window: {e}");
    }
}

/// Pop-out size: 1180×820 logical, shrunk to 90% of the main window's monitor.
fn popout_size(app: &AppHandle) -> (f64, f64) {
    let monitor = app
        .get_window(MAIN)
        .and_then(|w| w.current_monitor().ok().flatten());
    match monitor {
        Some(m) => {
            let size = m.size().to_logical::<f64>(m.scale_factor());
            (
                (size.width * 0.9).min(1180.0),
                (size.height * 0.9).min(820.0),
            )
        }
        None => (1180.0, 820.0),
    }
}

fn close_peek(app: &AppHandle) {
    if let Some(peek) = app.get_webview(PEEK) {
        let _ = peek.close();
    }
}

fn reposition_peek(app: &AppHandle) {
    let Some(peek) = app.get_webview(PEEK) else {
        return;
    };
    let (size, pos) = peek_size_and_position_logical(app);
    let _ = peek.set_size(size);
    let _ = peek.set_position(pos);
}

fn peek_size_and_position_logical(app: &AppHandle) -> (LogicalSize<f64>, LogicalPosition<f64>) {
    let fallback = (
        LogicalSize::new(800.0, 600.0),
        LogicalPosition::new(100.0, 100.0),
    );
    let Some(main_window) = app.get_window(MAIN) else {
        return fallback;
    };
    let Ok(phys) = main_window.inner_size() else {
        return fallback;
    };
    let logical = phys.to_logical::<f64>(main_window.scale_factor().unwrap_or(1.0));
    peek_rect(logical.width, logical.height)
}

/// Peek geometry: 85% of the window, but on narrow/short windows use more of
/// the space (down to a 12px margin) so the overlay stays usable.
fn peek_rect(win_w: f64, win_h: f64) -> (LogicalSize<f64>, LogicalPosition<f64>) {
    let fit = |total: f64| (total * 0.85).max(total - 24.0).min(total).max(0.0);
    let (w, h) = if win_w < 900.0 || win_h < 640.0 {
        (fit(win_w), fit(win_h))
    } else {
        (win_w * 0.85, win_h * 0.85)
    };
    (
        LogicalSize::new(w, h),
        LogicalPosition::new((win_w - w) / 2.0, (win_h - h) / 2.0),
    )
}

fn icon_bytes(id: &str) -> &'static [u8] {
    match id {
        "icon-dark" => DARK_ICON,
        "icon-white" => WHITE_ICON,
        _ => COLOR_ICON,
    }
}

fn icon_pref_path(app: &AppHandle) -> Option<std::path::PathBuf> {
    app.path().app_config_dir().ok().map(|d| d.join("icon"))
}

fn save_icon_choice(app: &AppHandle, id: &str) {
    if let Some(path) = icon_pref_path(app) {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(path, id);
    }
}

fn restore_icon_choice(app: &AppHandle) {
    let choice = icon_pref_path(app).and_then(|p| std::fs::read_to_string(p).ok());
    if let Some(id) = choice.as_deref().map(str::trim) {
        if matches!(id, "icon-dark" | "icon-white") {
            set_main_window_icon(app, icon_bytes(id));
        }
    }
}

fn set_main_window_icon(app: &AppHandle, icon_bytes: &[u8]) {
    let Some(window) = app.get_webview_window(MAIN) else {
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
        let Some(nsdata_class) = objc::runtime::Class::get("NSData") else {
            return;
        };
        let data: id =
            msg_send![nsdata_class, dataWithBytes: icon_bytes.as_ptr() length: icon_bytes.len()];
        let Some(ns_image_class) = objc::runtime::Class::get("NSImage") else {
            return;
        };
        let alloc_image: id = msg_send![ns_image_class, alloc];
        let image: id = msg_send![alloc_image, initWithData: data];
        if image != nil {
            if let Some(ns_app_class) = objc::runtime::Class::get("NSApplication") {
                let app: id = msg_send![ns_app_class, sharedApplication];
                let _: () = msg_send![app, setApplicationIconImage: image];
            }
            // NSApplication retains the image; balance our alloc/init.
            let _: () = msg_send![image, release];
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

fn title_for_url(url: &tauri::Url) -> String {
    match url.host_str() {
        Some("mail.google.com" | "gmail.com") => "Gmail".into(),
        Some("docs.google.com") => "Google Docs".into(),
        Some("drive.google.com") => "Google Drive".into(),
        Some("calendar.google.com") => "Google Calendar".into(),
        Some("meet.google.com") => "Google Meet".into(),
        Some("contacts.google.com") => "Google Contacts".into(),
        Some("keep.google.com") => "Google Keep".into(),
        Some("tasks.google.com") => "Google Tasks".into(),
        Some("chat.google.com") => "Google Chat".into(),
        // Show the real host rather than implying Google ownership.
        Some(host) => host.to_string(),
        None => "Gmail Desktop".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u(s: &str) -> tauri::Url {
        s.parse().unwrap()
    }

    #[test]
    fn unread_from_title() {
        assert_eq!(parse_unread("Inbox (3) - me@example.com - Gmail"), Some(3));
        assert_eq!(
            parse_unread("Inbox (1,234) - me@example.com - Gmail"),
            Some(1234)
        );
        assert_eq!(parse_unread("Inbox - me@example.com - Gmail"), None);
        assert_eq!(parse_unread("Gmail"), None);
    }

    #[test]
    fn google_hosts() {
        assert!(is_google_host(&u("https://docs.google.com/x")));
        assert!(is_google_host(&u("https://mail.google.com/")));
        assert!(!is_google_host(&u("https://google.com.evil.example/")));
        assert!(!is_google_host(&u("https://example.com/")));
        assert!(!is_google_host(&u("mailto:a@example.com")));
    }

    #[test]
    fn unwraps_gmail_redirects() {
        let r = unwrap_redirect(&u(
            "https://www.google.com/url?q=https://example.com/a&sa=D",
        ));
        assert_eq!(r.as_str(), "https://example.com/a");
        let d = u("https://docs.google.com/document/d/1");
        assert_eq!(unwrap_redirect(&d), d);
    }

    #[test]
    fn mailto_is_encoded() {
        let s = mailto_compose_url(&u("mailto:a@example.com?subject=Hi%20there"));
        assert!(s.starts_with("https://mail.google.com/mail/?extsrc=mailto&url=mailto%3A"));
        assert!(!s[40..].contains('?'));
    }

    #[test]
    fn peek_fits_small_windows() {
        let (size, pos) = peek_rect(720.0, 500.0);
        assert!(size.width >= 696.0 - 0.001 && pos.x >= 0.0);
        let (size, _) = peek_rect(1280.0, 860.0);
        assert!((size.width - 1088.0).abs() < 0.001);
    }

    #[test]
    fn unknown_hosts_not_branded() {
        assert_eq!(title_for_url(&u("https://example.com/")), "example.com");
    }
}
