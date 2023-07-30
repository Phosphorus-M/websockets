use axum::{Router, routing::{get}};
use futures_util::stream::{SplitSink, SplitStream};
use once_cell::sync::{Lazy};
use tokio::sync::{RwLock, Mutex};
use std::{net::SocketAddr, collections::{HashMap}, sync::Arc};

pub mod config;
pub mod models;
pub mod helpers;
pub mod routes;

use config::setup::{fetch_pokemon_list};
use models::{named_api_resource::{NamedAPIResource}, bullshit::Pokemon, user::{User, Roles}};
pub use routes::restful::{
    root::root,
    ws_handler::ws_handler
};

use tokio::sync::broadcast;
use axum_tungstenite::{WebSocket, Message};

#[derive(Debug)]
pub struct AppState {
    // We require unique usernames. This tracks which usernames have been taken.
    users: Mutex<HashMap<String, User>>,
    sockets: RwLock<Vec<(SplitSink<WebSocket, Message>, SplitStream<WebSocket>)>>,
    channels: RwLock<HashMap<String, broadcast::Sender<String>>>,
    // // Channel used to send messages to all connected clients.
    // tx: broadcast::Sender<String>,
}

impl Default for AppState {
    fn default() -> Self {
        let mut users = HashMap::new();
        users.insert("ivan".to_string(), User::new(1, "ivan".to_string(), "rusthater".to_string(), Roles::User));
        users.insert("waifu.rs".to_string(), User::new(2, "waifu.rs".to_string(), "uwu".to_string(), Roles::Admin));
        
        let mut channels = HashMap::new();

        let (tx, _rx) = broadcast::channel::<String>(100);

        channels.insert("s".to_string(), tx);

        Self {
            users: Mutex::new(users),
            sockets: RwLock::new(Vec::new()),
            channels: RwLock::new(channels),
            // tx: broadcast::channel(100).0,
        }
    }
}


pub static POKEMON_LIST: Lazy<RwLock<Vec<NamedAPIResource>>> = Lazy::new(|| RwLock::new(Vec::with_capacity(1010)));
pub static POKEMON_INFO_BY_NAME: Lazy<RwLock<HashMap<String, Pokemon>>> = Lazy::new(|| RwLock::new(HashMap::with_capacity(1010)));
pub static POKEMON_INFO_BY_ID: Lazy<RwLock<HashMap<u32, Pokemon>>> = Lazy::new(|| RwLock::new(HashMap::with_capacity(1010)));

#[tokio::main]
async fn main(){
    tracing_subscriber::fmt::init();

    let Ok(_state) = fetch_pokemon_list().await else {
        println!("Error");
        return;
    };

    println!("Hello, world!");

    let app = Router::new()
    .route("/", get(root))
    .route("/ws",  get(ws_handler))
    .with_state(Arc::new(AppState::default()))
    ;

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
