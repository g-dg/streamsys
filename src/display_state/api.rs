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
use tokio::sync::{mpsc, watch};

use crate::{app::AppState, auth::db::UserPermission};

use super::service::DisplayState;

pub fn route() -> Router<Arc<AppState>> {
    Router::new().route("/", get(handler))
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum DisplayStateRequest {
    Get { get: bool },
    Authenticate { auth_token: String },
    Set { state: DisplayState },
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum DisplayStateResponse {
    AuthResult { auth: bool },
    State { state: DisplayState },
}

pub async fn handler(State(state): State<Arc<AppState>>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|socket| websocket_handler(socket, state))
}

pub async fn websocket_handler(socket: WebSocket, state: Arc<AppState>) {
    let mut client_auth_token = None;

    let (mut ws_send, mut ws_recv) = socket.split();

    // send a message to this queue to send it to the client
    let (queue_send, mut queue_recv) = mpsc::channel::<String>(1);

    // sends messages to the client from the message queue
    let mut send_task = tokio::spawn(async move {
        while let Some(response) = queue_recv.recv().await {
            ws_send.send(Message::Text(response)).await.unwrap();
        }
    });

    let r_state = state.clone();
    let r_queue_send = queue_send.clone();

    // handles incoming requests from the client
    let mut recv_task = tokio::spawn(async move {
        let mut watch_recv = r_state.display_state_service.watch_recv.clone();
        let watch_send = r_state.display_state_service.watch_send.clone();

        async fn send_response(
            response: &DisplayStateResponse,
            queue_send: &mpsc::Sender<String>,
        ) -> Result<(), ()> {
            let response_json = serde_json::to_string(&response).unwrap();
            if queue_send.send(response_json).await.is_err() {
                return Err(());
            }
            Ok(())
        }

        async fn send_current_state(
            watch_recv: &mut watch::Receiver<DisplayState>,
            queue_send: &mpsc::Sender<String>,
        ) -> Result<(), ()> {
            let state = watch_recv.borrow_and_update().clone();
            let response = DisplayStateResponse::State { state };
            send_response(&response, queue_send).await
        }

        while let Some(Ok(msg)) = ws_recv.next().await {
            match msg {
                Message::Text(msg) => {
                    let request: DisplayStateRequest =
                        serde_json::from_str(&msg).expect("Failed to parse display state request");

                    match request {
                        DisplayStateRequest::Get { get: _ } => {
                            // respond with current state
                            if send_current_state(&mut watch_recv, &r_queue_send)
                                .await
                                .is_err()
                            {
                                return;
                            }
                        }

                        DisplayStateRequest::Authenticate { auth_token } => {
                            // check permissions
                            let auth_result = r_state
                                .auth_service
                                .authorize(&auth_token, UserPermission::OPERATION)
                                .is_some();

                            // respond with auth result
                            let send_result = send_response(
                                &DisplayStateResponse::AuthResult { auth: auth_result },
                                &r_queue_send,
                            )
                            .await;

                            client_auth_token = Some(auth_token);

                            if send_result.is_err() {
                                return;
                            }
                        }

                        DisplayStateRequest::Set { state } => {
                            // check permissions
                            let can_set_state = if let Some(ref auth_token) = client_auth_token {
                                r_state
                                    .auth_service
                                    .authorize(auth_token, UserPermission::OPERATION)
                                    .is_some()
                            } else {
                                false
                            };

                            if can_set_state {
                                // set state (will trigger response)
                                if watch_send.send(state).is_err() {
                                    return;
                                }
                            } else {
                                // respond with current state
                                if send_response(
                                    &DisplayStateResponse::AuthResult { auth: false },
                                    &r_queue_send,
                                )
                                .await
                                .is_err()
                                {
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

    // watch for changed state
    let watch_task = tokio::spawn(async move {
        let mut watch_recv = state.display_state_service.watch_recv.clone();
        while let Ok(()) = watch_recv.changed().await {
            let state = watch_recv.borrow_and_update().clone();
            let response = DisplayStateResponse::State { state };
            let response_json = serde_json::to_string(&response).unwrap();
            if queue_send.send(response_json).await.is_err() {
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
