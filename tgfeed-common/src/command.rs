use tokio::sync::oneshot;

#[derive(Debug)]
pub enum MonitorCommand {
    Subscribe {
        channel_handle: String,
        response: oneshot::Sender<Result<(), String>>,
    },

    Unsubscribe {
        channel_handle: String,
        response: oneshot::Sender<Result<(), String>>,
    },

    ListSubscriptions {
        response: oneshot::Sender<Vec<String>>,
    },

    Shutdown,
}
