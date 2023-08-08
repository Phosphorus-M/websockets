use std::{
    borrow::BorrowMut,
    collections::{hash_map::RandomState, HashMap},
    sync::Arc,
};

use axum::extract::State;
use axum::response::IntoResponse;
use axum_tungstenite::{Message, WebSocket, WebSocketUpgrade};
use futures_util::{
    stream::{SplitSink, SplitStream},
    StreamExt,
};

use headers::HeaderMap;

use crate::{
    helpers::utils::authorize,
    models::user::{Roles, User},
    AppState,
};
use futures_util::sink::SinkExt;
use tokio::{
    sync::{broadcast::Receiver, Mutex, MutexGuard},
    task::JoinHandle,
};

use crate::helpers::utils::authorize::authorize;
use crate::models::command::Message as MessageStruct;
use axum::debug_handler;

#[debug_handler]
pub async fn ws_handler(
    headers: HeaderMap,
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state, headers))
}

async fn handle_socket(stream: WebSocket, state: Arc<AppState>, headers: HeaderMap) {
    let (sender, mut receiver) = stream.split();

    // Username gets set in the receive loop, if it's valid.
    let mut username = String::new();
    let sender_arc = Arc::new(Mutex::new(sender));

    check_first_messages(
        receiver.borrow_mut(),
        sender_arc.lock().await,
        &mut username,
        state.clone(),
        headers.clone(),
    )
    .await;

    let rx = state
        .channels
        .write()
        .await
        .get_mut("s")
        .unwrap()
        .subscribe();

    let send_task = sender_from_channel_to_client(rx, sender_arc.clone()).await;

    println!("Y vio que salio!");

    let recv_task = receiver_from_client_to_channel(
        state.clone(),
        receiver,
        sender_arc.clone(),
        username.clone(),
        headers,
    )
    .await;

    println!("Y vio que salio2222!");
    handle_user_left(recv_task, send_task, username, state).await
}

async fn check_first_messages(
    receiver: &mut SplitStream<WebSocket>,
    mut sender: MutexGuard<'_, SplitSink<WebSocket, Message>>,
    username: &mut String,
    state: Arc<AppState>,
    headers: HeaderMap,
) {
    // Loop until a text message is found.
    if let Some(jwt) = headers.get("Authorization") {
        if let Ok(user_id) = authorize((Roles::User, headers)).await {
            let users = state.users.lock().await;
            let user = users.values().find(|user| user.id == user_id);
            if let Some(user) = user {
                *username = user.nick.clone();
                let _ = sender
                    .send(Message::Text(format!("Welcome back, {}!", user.nick)))
                    .await;
            } else {
                let _ = sender
                    .send(Message::Text("Invalid token".to_string()))
                    .await;
            }
        } else {
            let _ = sender
                .send(Message::Text("Invalid token".to_string()))
                .await;
        };
    } else {
        while let Some(Ok(message)) = receiver.next().await {
            if let Message::Text(msg) = message {
                let Ok(command): Result<MessageStruct, _> = serde_json::from_str(&msg) else {
                    tracing::debug!("Received non-text message");
                    continue;
                };

                let users = state.users.lock().await;

                let jwt = if let Some(token) = headers.get("Authorization") {
                    token.to_str().unwrap_or_default().to_string()
                } else {
                    String::new()
                };

                let Some(response) = command.execute(users, &jwt).await else {
                    let error = Message::Text("Invalid command".to_string());
                    let _ = sender
                            .send(error)
                            .await;
                    continue;
                };

                *username = command.arguments.unwrap_or_default()[0].clone();

                let _ = sender.send(Message::Text(response)).await;

                break;
            }
        }
        println!("SALIOO!");

        // We subscribe *before* sending the "joined" message, so that we will also
        // display it to our client.
        let rx = state
            .channels
            .write()
            .await
            .get_mut("s")
            .unwrap()
            .subscribe();

        // Now send the "joined" message to all subscribers.
        let msg = format!("{} joined.", username);
        tracing::debug!("{}", msg);
        let _ = state.channels.write().await.get_mut("s").unwrap().send(msg);
    }
}
async fn sender_from_channel_to_client(
    mut rx: Receiver<String>,
    concurrent_sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
) -> JoinHandle<()> {
    // Spawn the first task that will receive broadcast messages and send text
    // messages over the websocket to our client.
    let send_task = tokio::spawn(async move {
        let mut sender = concurrent_sender.lock().await;
        while let Ok(msg) = rx.recv().await {
            // In any websocket error, break loop.
            let Ok(todo_bien) = sender.send(Message::Text(msg.clone())).await else {
                println!("Error sending message to client: {}", msg);
                break;
            };  
        }
    });

    send_task
}

async fn receiver_from_client_to_channel(
    state: Arc<AppState>,
    mut receiver: SplitStream<WebSocket>,
    concurrent_sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    username: String,
    headers: HeaderMap,
) -> JoinHandle<()> {
    // Clone things we want to pass (move) to the receiving task.
    let tx = state.channels.write().await.get_mut("s").unwrap().clone();
    let name = username.clone();

    // Spawn a task that takes messages from the websocket, prepends the user
    // name, and sends them to all broadcast subscribers.
    tokio::spawn(async move {
        let mut sender = concurrent_sender.lock().await;
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            let Ok(command): Result<MessageStruct, _> = serde_json::from_str(&text) else {
                tracing::debug!("Received non-text message");
                continue;
            };

            tracing::debug!("Received message: {}", text);
            let users = state.users.lock().await;
            let jwt = if let Some(token) = headers.get("Authorization") {
                token.to_str().unwrap_or_default().to_string()
            } else {
                String::new()
            };
            println!("Y vio que sal4444444444!");
            let Some(response) = command.execute(users, &jwt).await else {
                let error = Message::Text("Invalid command".to_string());
                let _ = sender.send(error).await;
                continue;
            };

            let algo = format!("{}: {}", name, text);

            sender.send(Message::Text(response.clone())).await.unwrap();
            let _ = tx.send(response);
        }
    })
}

async fn handle_user_left(
    mut recv_task: JoinHandle<()>,
    mut send_task: JoinHandle<()>,
    username: String,
    state: Arc<AppState>,
) {
    // If any one of the tasks run to completion, we abort the other.
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    // Send "user left" message (similar to "joined" above).
    let msg = format!("{} left.", username);
    tracing::debug!("{}", msg);
    let _ = state.channels.write().await.get_mut("s").unwrap().send(msg);

    // Remove username from map so new clients can take it again.
    // state.user_set.lock().unwrap().remove(&username);
}

// while let Some(msg) = socket.recv().await {

//     let msg = if let Ok(msg) = msg {
//         println!("{:?}", msg);
//         let Message::Text(msg) = msg else {
//             tracing::debug!("Received non-text message");
//             continue;
//         };

//         let Ok(command): Result<MessageStruct, _> = serde_json::from_str(&msg) else {
//             tracing::debug!("Received non-text message");
//             continue;
//         };

//         tracing::debug!("Received message: {}", msg);
//         let users = state.user_set.lock().await;
//         let Some(response) = command.execute(users).await else {
//             let error = Message::Text("Invalid command".to_string());
//             let _ = socket.send(error).await;
//             continue;
//         };
//         Message::Text(response)
//     } else {
//         // client disconnected
//         return;
//     };

//     if socket.send(msg).await.is_err() {
//         println!("Error al enviar");
//         // client disconnected
//         return;
//     }
// }
