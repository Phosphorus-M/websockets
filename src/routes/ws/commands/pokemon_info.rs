use color_eyre::Result;

use crate::{models::bullshit::Pokemon, POKEMON_INFO_BY_ID, POKEMON_INFO_BY_NAME};

pub async fn fetch_pokemon_info(message: &str) -> Option<Pokemon> {
    if let Ok(id) = message.parse::<u32>() {
        return get_pokemon_by_id(id).await;
    }

    get_pokemon_by_name(message).await
}


async fn fetch_pokemon(path: String) -> Result<Pokemon>{
    let pokemon = reqwest::get(&format!("https://pokeapi.co/api/v2/pokemon/{}", path))
    .await?
    .json::<Pokemon>()
    .await?;

    Ok(pokemon)
}

async fn get_pokemon_by_id(id: u32) -> Option<Pokemon> {
    if let Some(pokemon) = POKEMON_INFO_BY_ID.read().await.get(&id) {
        println!("Tiene cacheado");
        return Some(pokemon.clone());
    };
    println!("NOOOO Tiene cacheado");
    let pokemon = fetch_pokemon(id.to_string()).await.ok()?;
    POKEMON_INFO_BY_ID.write().await.insert(pokemon.id, pokemon.clone());
    POKEMON_INFO_BY_NAME.write().await.insert(pokemon.name.clone(), pokemon.clone());
    
    return Some(pokemon);
}

async fn get_pokemon_by_name(name: &str) -> Option<Pokemon> {
    if let Some(pokemon) = POKEMON_INFO_BY_NAME.read().await.get(name) {
        return Some(pokemon.clone());
    };
    

    let pokemon = fetch_pokemon(name.to_string()).await.ok()?;

    POKEMON_INFO_BY_NAME.write().await.insert(pokemon.name.clone(), pokemon.clone());
    POKEMON_INFO_BY_ID.write().await.insert(pokemon.id, pokemon.clone());

    Some(pokemon)
}