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

impl MonitorCommand {
    pub fn get_user_id(&self) -> Option<i64> {
        match self {
            MonitorCommand::Subscribe { user_id, .. }
            | MonitorCommand::Unsubscribe { user_id, .. }
            | MonitorCommand::ListSubscriptions { user_id, .. }
            | MonitorCommand::Summarize { user_id, .. } => Some(*user_id),
            MonitorCommand::Shutdown => None,
        }
    }

    pub fn respond_with_error(self, message: String) {
        match self {
            MonitorCommand::Subscribe { response, .. } => {
                response.send(Err(message)).expect("broken channel")
            }
            MonitorCommand::Unsubscribe { response, .. } => {
                response.send(Err(message)).expect("broken channel")
            }
            MonitorCommand::ListSubscriptions { response, .. } => {
                response.send(Err(message)).expect("broken channel")
            }
            MonitorCommand::Summarize { response, .. } => {
                response.send(Err(message)).expect("broken channel")
            }
            MonitorCommand::Shutdown => (),
        }
    }
}
