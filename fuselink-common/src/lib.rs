use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct PokemonData {
    pub owner_name: String,
    pub species_name: String,
    pub nickname: String,
    pub obtained_location: String,
    pub data: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct PokemonUploadData {
    pub pokemon: PokemonData,
    pub pass: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct GetPokemonData {
    pub pass: String,
}
