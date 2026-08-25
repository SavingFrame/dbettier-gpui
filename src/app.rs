use gpui::{App, AppContext, Bounds, WindowBounds, WindowOptions, px, size};
use gpui_platform::application;

use crate::{
    database::{profile_storage::FileDatabaseStorage, profile_store::DatabaseProfileStore},
    theme::AppTheme,
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
    application().run(|cx: &mut App| {
        cx.set_global(AppTheme::dark());

        let profile_store = cx.new(|cx| DatabaseProfileStore::new(database_storage(), cx));
        DatabaseProfileStore::set_global(profile_store, cx);

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
