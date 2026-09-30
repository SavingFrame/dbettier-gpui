use gpui::{App, AppContext, Bounds, WindowBounds, WindowOptions, px, size};
use gpui_kit::{
    application,
    component::{Theme, ThemeMode},
    init, open_window,
};

use crate::{
    assets::Assets,
    database::{
        profile_storage::FileDatabaseStorage, profile_store::DatabaseProfileStore,
        session_store::DatabaseSessionStore,
    },
    runtime,
    workspace::Workspace,
};

fn database_storage() -> Result<FileDatabaseStorage, String> {
    let config_directory = dirs::config_dir()
        .ok_or_else(|| "the system config directory is unavailable".to_owned())?;

    Ok(FileDatabaseStorage::new(
        config_directory.join("dbettier").join("databases.json"),
    ))
}

pub(crate) fn run() {
    application()
        .with_assets(Assets::new())
        .run(|cx: &mut App| {
            if let Err(error) = runtime::init(cx) {
                eprintln!("failed to start database runtime: {error}");
                return;
            }
            init(cx);
            Theme::change(ThemeMode::Dark, None, cx);
            Theme::update(cx, |theme| {
                theme.notification.max_items = 3;
                theme.notification.width = px(420.);
                theme.notification.margins.top = px(16.);
            });

            let profile_store = cx.new(|cx| DatabaseProfileStore::new(database_storage(), cx));
            DatabaseProfileStore::set_global(profile_store, cx);

            let session_store = cx.new(|_| DatabaseSessionStore::new());
            DatabaseSessionStore::set_global(session_store, cx);

            let bounds = Bounds::centered(None, size(px(1400.), px(900.)), cx);
            let window = open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                cx,
                |window, cx| cx.new(|cx| Workspace::new(window, cx)),
            );

            if let Err(error) = window {
                eprintln!("failed to open dbettier window: {error}");
                return;
            }

            cx.activate(true);
        });
}
