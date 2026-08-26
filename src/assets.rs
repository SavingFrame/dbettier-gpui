use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};

pub(crate) struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let bytes: Option<&'static [u8]> = match path {
            "icons/database.svg" => Some(include_bytes!("../assets/icons/database.svg")),
            "icons/schema.svg" => Some(include_bytes!("../assets/icons/schema.svg")),
            "icons/table.svg" => Some(include_bytes!("../assets/icons/table.svg")),
            _ => None,
        };

        Ok(bytes.map(Cow::Borrowed))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        if path == "icons" {
            Ok(vec![
                "database.svg".into(),
                "schema.svg".into(),
                "table.svg".into(),
            ])
        } else {
            Ok(Vec::new())
        }
    }
}
