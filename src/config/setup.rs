

use color_eyre::Result;

use crate::{models::named_api_resource::NamedAPIResource, POKEMON_LIST};

pub async fn fetch_pokemon_list() -> Result<()> {

    let pokemon_list = reqwest::get("https://pokeapi.co/api/v2/pokemon")
    .await?
    .json::<serde_json::Value>()
    .await?
    .get("results")
    .ok_or(color_eyre::eyre::eyre!("No results"))?.clone();

    let mut resp: Vec<NamedAPIResource> =  serde_json::from_value(pokemon_list)?;

    POKEMON_LIST.write().await.append(&mut resp);

    Ok(())
}