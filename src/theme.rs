use gpui::{Global, Rgba, rgb};

#[derive(Clone, Debug)]
pub(crate) struct AppTheme {
    pub(crate) background: Rgba,
    pub(crate) panel: Rgba,
    pub(crate) panel_raised: Rgba,
    pub(crate) status_bar: Rgba,
    pub(crate) border: Rgba,
    pub(crate) border_subtle: Rgba,
    pub(crate) text: Rgba,
    pub(crate) text_muted: Rgba,
    pub(crate) text_subtle: Rgba,
    pub(crate) selection: Rgba,
    pub(crate) success: Rgba,
    pub(crate) warning: Rgba,
    pub(crate) error: Rgba,
    pub(crate) editor_text: Rgba,
    pub(crate) line_number: Rgba,
}

impl AppTheme {
    pub(crate) fn dark() -> Self {
        Self {
            background: rgb(0x17191d),
            panel: rgb(0x1d2025),
            panel_raised: rgb(0x22262c),
            status_bar: rgb(0x20242a),
            border: rgb(0x343941),
            border_subtle: rgb(0x292d33),
            text: rgb(0xd5d9df),
            text_muted: rgb(0x818894),
            text_subtle: rgb(0xb4bac3),
            selection: rgb(0x28364d),
            success: rgb(0x75be88),
            warning: rgb(0xd7ba7d),
            error: rgb(0xe06c75),
            editor_text: rgb(0xc6ccd5),
            line_number: rgb(0x5f6670),
        }
    }
}

impl Global for AppTheme {}
