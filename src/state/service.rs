use tokio::sync::watch;

use super::models::CurrentState;

pub struct StateService {
    pub watch_send: watch::Sender<CurrentState>,
    pub watch_recv: watch::Receiver<CurrentState>,
}

impl StateService {
    pub fn new() -> Self {
        let (send, recv) = watch::channel(CurrentState::default());

        Self {
            watch_send: send,
            watch_recv: recv,
        }
    }
}

impl Default for StateService {
    fn default() -> Self {
        Self::new()
    }
}
