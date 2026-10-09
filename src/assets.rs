use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};
use gpui_kit::assets::icon_assets;

icon_assets!(
    ExtraIcons,
    [
        Columns3,
        Database,
        ListOrdered,
        KeyRound,
        Network,
        Table2,
        SkipBack,
        SkipForward
    ]
);

pub(crate) struct Assets {
    kit: gpui_kit::assets::Assets,
}

impl Assets {
    pub(crate) fn new() -> Self {
        Self {
            kit: gpui_kit::assets::Assets::new(""),
        }
    }
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some(bytes) = ExtraIcons.load(path)? {
            return Ok(Some(bytes));
        }
        self.kit.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = self.kit.list(path)?;
        paths.extend(ExtraIcons.list(path)?);
        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}
