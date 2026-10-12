use gpui::{
    AnyWindowHandle, App, AppContext as _, Bounds, Entity, TitlebarOptions, WindowBounds,
    WindowOptions, px, size,
};

use super::{ConnectionEditor, ConnectionEditorEvent};
use crate::database::DatabaseConnectionProfile;

/// The caller can subscribe to the returned editor entity to handle Save and Test requests.
/// The window owns the editor independently of the workspace.
pub(crate) fn open(
    profile: Option<&DatabaseConnectionProfile>,
    cx: &mut App,
) -> gpui::Result<(AnyWindowHandle, Entity<ConnectionEditor>)> {
    let title = if profile.is_some() {
        "Edit database connection"
    } else {
        "Add database connection"
    };
    let bounds = Bounds::centered(None, size(px(600.), px(760.)), cx);
    gpui_kit::open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            window_min_size: Some(size(px(500.), px(440.))),
            titlebar: Some(TitlebarOptions {
                title: Some(title.into()),
                ..Default::default()
            }),
            ..Default::default()
        },
        cx,
        |window, cx| {
            let editor = cx.new(|cx| ConnectionEditor::new(profile, window, cx));
            let close_editor = editor.downgrade();
            window.on_window_should_close(cx, move |_, cx| {
                if let Some(editor) = close_editor.upgrade() {
                    editor.update(cx, |_, cx| cx.emit(ConnectionEditorEvent::Cancelled));
                }
                true
            });
            let name = editor.read(cx).name.clone();
            name.update(cx, |name, cx| name.focus(window, cx));
            editor
        },
    )
}
