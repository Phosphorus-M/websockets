use std::sync::Arc;

use axum::extract::State;
use axum::response::IntoResponse;
use axum_tungstenite::{WebSocket, WebSocketUpgrade, Message};
use futures_util::StreamExt;
use serde::Serialize;
use crate::AppState;
use crate::models::command::Message as MessageStruct;

use crate::routes::ws::commands::pokemon_info::{fetch_pokemon_info};


pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    // let (mut sender, mut receiver) = socket.split();
    let mut rx = state.channels.get("s").unwrap().subscribe();

    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            println!("{:?}", msg);
            let Message::Text(msg) = msg else {
                tracing::debug!("Received non-text message");
                continue;
            };


            let Ok(command): Result<MessageStruct, _> = serde_json::from_str(&msg) else {
                tracing::debug!("Received non-text message");
                continue;
            };

            tracing::debug!("Received message: {}", msg);
            let Some(response) = identify_command(command).await else {
                let error = Message::Text("Invalid command".to_string());
                let _ = socket.send(error).await;
                continue;
            };
            Message::Text(response)
        } else {
            // client disconnected
            return;
        };

        if socket.send(msg).await.is_err() {
            // client disconnected
            return;
        }
    }
}

async fn identify_command(msg: MessageStruct) -> Option<String> {
    let Some(command) = &msg.command else {
        return None;
    };

    if !msg.has_enoughs_params() {
        return None;
    }

    let arguments = msg.arguments.unwrap_or(Vec::new());

    match command.as_str() {
        "pokemon_info" => {
            let Some(pokemon) = fetch_pokemon_info(arguments[0].as_str()).await else {
                return None;
            };
            serde_json::to_string(&pokemon).ok()
        },
        "algo" => todo!(),
        "otra cosa" => unimplemented!(),
        "Acá no llega" => unreachable!(),
        _ => None
    }
 
}