use crate::osx_utils::title_bar_height;
use crate::MAIN_WEBVIEW;
use tauri::{LogicalSize, Manager, Runtime, Window};

const DEVTOOLS_HEIGHT: f64 = 500.;
pub trait EnhancedWindow<R: Runtime> {
    fn size(&self) -> tauri::Result<LogicalSize<f64>>;
    fn available_size(&self) -> tauri::Result<LogicalSize<f64>>;
    fn title_bar_height(&self) -> f64;
}

impl<R: Runtime> EnhancedWindow<R> for Window<R> {
    fn size(&self) -> tauri::Result<LogicalSize<f64>> {
        let scale = self.scale_factor()?;

        Ok(self.inner_size()?.to_logical(scale))
    }

    fn available_size(&self) -> tauri::Result<LogicalSize<f64>> {
        let mut window_size = self.size()?;
        if let Some(webview) = self.get_webview(MAIN_WEBVIEW)
            && webview.is_devtools_open()
        {
            window_size.height = window_size.height - DEVTOOLS_HEIGHT;
        }

        Ok(window_size)
    }

    fn title_bar_height(&self) -> f64 {
        title_bar_height(self)
    }
}
