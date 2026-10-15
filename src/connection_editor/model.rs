#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConnectionInputMode {
    Fields,
    Uri,
}

// These are UI values, not a persisted profile or a parsed database configuration.
#[derive(Clone, PartialEq, Eq)]
pub(crate) enum ConnectionDetails {
    Fields {
        host: String,
        port: String,
        database: String,
        username: String,
        password: String,
        ssl_mode: String,
    },
    Uri(String),
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct ConnectionDraft {
    pub(crate) profile_id: Option<String>,
    pub(crate) name: String,
    pub(crate) details: ConnectionDetails,
}
