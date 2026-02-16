use tauri::{Runtime, Webview, Window};

#[cfg(target_os = "macos")]
pub fn title_bar_height<R: Runtime>(window: &Window<R>) -> f64 {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;
    use objc2_foundation::NSRect;

    unsafe {
        let ns_window = window.ns_window().unwrap() as *mut AnyObject;
        let frame: NSRect = msg_send![ns_window, frame];
        let content_rect: NSRect = msg_send![ns_window, contentLayoutRect];

        frame.size.height - content_rect.size.height
    }
}

#[cfg(not(target_os = "macos"))]
pub fn title_bar_height<R: Runtime>(_window: &Window<R>) -> f64 {
    0.0
}

#[cfg(target_os = "macos")]
pub fn enable_swipe_navigation<R: Runtime>(webview: &Webview<R>) {
    let _ = webview.with_webview(|platform_wv| unsafe {
        let wkwebview: *mut objc2::runtime::AnyObject = platform_wv.inner().cast();
        let _: () = objc2::msg_send![wkwebview, setAllowsBackForwardNavigationGestures: true];
    });
}

#[cfg(not(target_os = "macos"))]
pub fn enable_swipe_navigation(_webview: &Webview) {}
