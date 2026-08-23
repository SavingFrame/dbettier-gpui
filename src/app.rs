use gpui::{App, AppContext, Bounds, WindowBounds, WindowOptions, px, size};
use gpui_platform::application;

use crate::{theme::AppTheme, workspace::Workspace};

pub(crate) fn run() {
    application().run(|cx: &mut App| {
        cx.set_global(AppTheme::dark());

        let bounds = Bounds::centered(None, size(px(1400.), px(900.)), cx);
        let window = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(Workspace::new),
        );

        if let Err(error) = window {
            eprintln!("failed to open dbettier window: {error}");
            return;
        }

        cx.activate(true);
    });
}
