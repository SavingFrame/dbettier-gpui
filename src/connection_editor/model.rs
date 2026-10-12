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

/// UI-only integration hooks. Drafts may contain credentials and should not be logged.
#[derive(Clone)]
#[allow(dead_code)]
pub(crate) enum ConnectionEditorEvent {
    SaveRequested(ConnectionDraft),
    TestRequested(ConnectionDraft),
    Cancelled,
}

/// Controlled by the consumer. Requests never imply a successful operation.
#[allow(dead_code)]
pub(crate) enum ConnectionEditorStatus {
    Idle,
    Testing,
    Saving,
    Success(String),
    Error(String),
    Notice(String),
}

impl ConnectionEditorStatus {
    pub(super) fn busy(&self) -> bool {
        matches!(self, Self::Testing | Self::Saving)
    }
}
