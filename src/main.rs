use axum::{Router, routing::{get}};
use once_cell::sync::{Lazy};
use tokio::sync::{RwLock, Mutex};
use std::{net::SocketAddr, collections::{HashMap, HashSet}, sync::Arc};

pub mod config;
pub mod models;
pub mod routes;

use config::setup::{fetch_pokemon_list};
use models::{named_api_resource::{NamedAPIResource}, bullshit::Pokemon};
pub use routes::restful::{
    root::root,
    ws_handler::ws_handler
};
use tokio::sync::broadcast::{Sender};

#[derive(Debug)]
pub struct AppState {
    // We require unique usernames. This tracks which usernames have been taken.
    user_set: Mutex<HashSet<String>>,
    channels: HashMap<String, Sender<String>>,
    // // Channel used to send messages to all connected clients.
    // tx: broadcast::Sender<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            user_set: Mutex::new(HashSet::new()),
            channels: HashMap::new(),
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
        .serve(app.into_make_service())
        .await
        .unwrap();
}
