use gpui::Task;

use crate::database::connection::ConnectionDetails;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConnectionInputMode {
    Fields,
    Uri,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct ConnectionDraft {
    pub(crate) profile_id: Option<String>,
    pub(crate) name: String,
    pub(crate) details: ConnectionDetails,
}

pub(super) enum TestConnectionStatus {
    Idle,
    Running(Option<Task<()>>),
    Finished(String),
    Failed(String),
}

pub(super) enum SaveConnectionStatus {
    Idle,
    Running(Option<Task<()>>),
    Finished,
    Failed(String),
}

impl TestConnectionStatus {
    pub(super) fn busy(&self) -> bool {
        matches!(self, Self::Running(_))
    }
}
