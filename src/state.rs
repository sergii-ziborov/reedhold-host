//! One unlocked session for this process.

use reedhold_api::{AccountView, Session};

/// In-memory host state.
#[derive(Default)]
pub struct State {
    pub(crate) session: Option<Session>,
}

impl State {
    pub(crate) fn view(&self) -> Result<AccountView, &'static str> {
        self.session.as_ref().map(Session::view).ok_or("no unlocked session")
    }

    pub(crate) fn with<T, E: ToString>(
        &self,
        op: impl FnOnce(&Session) -> Result<T, E>,
    ) -> Result<T, String> {
        let session = self.session.as_ref().ok_or_else(|| "no unlocked session".to_owned())?;
        op(session).map_err(|error| error.to_string())
    }

    pub(crate) fn with_mut<T, E: ToString>(
        &mut self,
        op: impl FnOnce(&mut Session) -> Result<T, E>,
    ) -> Result<T, String> {
        let session = self.session.as_mut().ok_or_else(|| "no unlocked session".to_owned())?;
        op(session).map_err(|error| error.to_string())
    }
}
