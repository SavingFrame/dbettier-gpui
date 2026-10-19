mod editor;
mod model;
mod view;
mod window;

use gpui::{App, KeyBinding, actions};

pub(crate) use editor::ConnectionEditor;
pub(crate) use model::ConnectionDraft;
pub(crate) use window::open;

actions!(connection_editor, [SaveConnection, CancelConnection]);

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("escape", CancelConnection, Some("ConnectionEditor")),
        KeyBinding::new("ctrl-enter", SaveConnection, Some("ConnectionEditor")),
        KeyBinding::new("cmd-enter", SaveConnection, Some("ConnectionEditor")),
    ]);
}
