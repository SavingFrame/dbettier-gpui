use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};

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
        let bytes: Option<&'static [u8]> = match path {
            "icons/column.svg" => Some(include_bytes!("../assets/icons/column.svg")),
            "icons/database.svg" => Some(include_bytes!("../assets/icons/database.svg")),
            "icons/index.svg" => Some(include_bytes!("../assets/icons/index.svg")),
            "icons/key.svg" => Some(include_bytes!("../assets/icons/key.svg")),
            "icons/schema.svg" => Some(include_bytes!("../assets/icons/schema.svg")),
            "icons/table.svg" => Some(include_bytes!("../assets/icons/table.svg")),
            _ => None,
        };

        match bytes {
            Some(bytes) => Ok(Some(Cow::Borrowed(bytes))),
            None => self.kit.load(path),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = self.kit.list(path)?;
        for icon in ["column", "database", "index", "key", "schema", "table"] {
            let filename = format!("icons/{icon}.svg");
            if filename.starts_with(path) {
                paths.push(filename.into());
            }
        }
        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;

    #[test]
    fn serves_application_and_kit_icons() {
        let assets = Assets::new();
        assert!(
            assets
                .load("icons/database.svg")
                .is_ok_and(|bytes| bytes.is_some())
        );
        assert!(
            assets
                .load("icons/inbox.svg")
                .is_ok_and(|bytes| bytes.is_some())
        );
    }
}
