use tokio::sync::oneshot;

#[derive(Debug)]
pub enum MonitorCommand {
    Subscribe {
        user_id: i64,
        channel_handle: String,
        response: oneshot::Sender<Result<(), String>>,
    },

    Unsubscribe {
        user_id: i64,
        channel_handle: String,
        response: oneshot::Sender<Result<(), String>>,
    },

    ListSubscriptions {
        user_id: i64,
        response: oneshot::Sender<Result<Vec<String>, String>>,
    },

    Summarize {
        user_id: i64,
        response: oneshot::Sender<Result<String, String>>,
    },

    Shutdown,
}
