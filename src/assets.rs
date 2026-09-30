use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};

pub(crate) struct Assets;

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

        Ok(bytes.map(Cow::Borrowed))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        if path == "icons" {
            Ok(vec![
                "column.svg".into(),
                "database.svg".into(),
                "index.svg".into(),
                "key.svg".into(),
                "schema.svg".into(),
                "table.svg".into(),
            ])
        } else {
            Ok(Vec::new())
        }
    }
}
