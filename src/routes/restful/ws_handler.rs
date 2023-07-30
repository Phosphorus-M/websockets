use std::sync::Arc;


use axum::extract::State;
use axum::response::IntoResponse;
use axum_tungstenite::{WebSocket, WebSocketUpgrade, Message};
use futures_util::StreamExt;

use headers::HeaderMap;

use futures_util::sink::SinkExt;
use crate::AppState;

use axum::debug_handler;


#[debug_handler]
pub async fn ws_handler(headers: HeaderMap, ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state, headers))
}

async fn handle_socket(stream: WebSocket, state: Arc<AppState>, headers: HeaderMap) {
    let (mut sender, mut receiver) = stream.split();
    
    // Username gets set in the receive loop, if it's valid.
    let mut username = String::new();
    
    // Loop until a text message is found.
    while let Some(Ok(message)) = receiver.next().await {
        if let Message::Text(name) = message {
            // If username that is sent by client is not taken, fill username string.
            username.push_str(name.as_str());

            // If not empty we want to quit the loop else we want to quit function.
            if !username.is_empty() {
                break;
            } else {
                // Only send our client that username is taken.
                let _ = sender
                    .send(Message::Text(String::from("Username already taken.")))
                    .await;

                return;
            }
        }
    }
    

    // We subscribe *before* sending the "joined" message, so that we will also
    // display it to our client.
    let mut rx = state.channels.write().await.get_mut("s").unwrap().subscribe();

    // Now send the "joined" message to all subscribers.
    let msg = format!("{} joined.", username);
    tracing::debug!("{}", msg);
    let _ = state.channels.write().await.get_mut("s").unwrap().send(msg);

    // Spawn the first task that will receive broadcast messages and send text
    // messages over the websocket to our client.
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // In any websocket error, break loop.
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Clone things we want to pass (move) to the receiving task.
    let tx = state.channels.write().await.get_mut("s").unwrap().clone();
    let name = username.clone();

    // Spawn a task that takes messages from the websocket, prepends the user
    // name, and sends them to all broadcast subscribers.
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            // Add username before message.
            let _ = tx.send(format!("{}: {}", name, text));
        }
    });

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