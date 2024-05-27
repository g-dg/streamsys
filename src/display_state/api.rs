use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{app::AppState, auth::db::UserPermission};

use super::service::DisplayState;

pub fn route() -> Router<Arc<AppState>> {
    Router::new().route("/", get(handler))
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum DisplayStateRequest {
    Get,
    Set { token: String, state: DisplayState },
}

pub async fn handler(State(state): State<Arc<AppState>>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|socket| websocket_handler(socket, state))
}

pub async fn websocket_handler(socket: WebSocket, state: Arc<AppState>) {
    let (mut ws_send, mut ws_recv) = socket.split();

    // send a message to this queue to send it to the client
    let (queue_send, mut queue_recv) = mpsc::channel::<String>(1);

    // sends messages to the client from the message queue
    let mut send_task = tokio::spawn(async move {
        while let Some(state) = queue_recv.recv().await {
            ws_send.send(Message::Text(state)).await.unwrap();
        }
    });

    let recv_state = state.clone();
    let recv_queue_send = queue_send.clone();

    // handles incoming requests from the client
    let mut recv_task = tokio::spawn(async move {
        let mut watch_recv = recv_state.display_state_service.watch_recv.clone();
        let watch_send = recv_state.display_state_service.watch_send.clone();

        while let Some(Ok(msg)) = ws_recv.next().await {
            match msg {
                Message::Text(msg) => {
                    let request: DisplayStateRequest =
                        serde_json::from_str(&msg).expect("Failed to parse display state request");

                    match request {
                        DisplayStateRequest::Get => {
                            // respond with current state
                            let state = watch_recv.borrow_and_update().clone();
                            let state_json = serde_json::to_string(&state).unwrap();
                            if recv_queue_send.send(state_json).await.is_err() {
                                return;
                            }
                        }
                        DisplayStateRequest::Set { token, state } => {
                            // check if user has permissions
                            if recv_state
                                .auth_service
                                .authorize(&token, UserPermission::OPERATION)
                                .is_some()
                            {
                                // set state (will trigger response)
                                if watch_send.send(state).is_err() {
                                    return;
                                }
                            } else {
                                // respond with current state
                                let state = watch_recv.borrow_and_update().clone();
                                let state_json = serde_json::to_string(&state).unwrap();
                                if recv_queue_send.send(state_json).await.is_err() {
                                    return;
                                }
                            }
                        }
                    }
                }
                Message::Close(_) => return,
                _ => {}
            }
        }
    });

    let watch_state = state.clone();
    let watch_queue_send = queue_send.clone();

    // watch for changed state
    let watch_task = tokio::spawn(async move {
        let mut watch_recv = watch_state.display_state_service.watch_recv.clone();
        while let Ok(()) = watch_recv.changed().await {
            let state = watch_recv.borrow_and_update().clone();
            let state_json = serde_json::to_string(&state).unwrap();
            if watch_queue_send.send(state_json).await.is_err() {
                return;
            }
        }
    });

    // abort tasks if send or receive task exit
    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
            watch_task.abort();
        },
        _ = (&mut recv_task) => {
            send_task.abort();
            watch_task.abort();
        }
    }
}
