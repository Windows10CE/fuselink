use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use fuselink_common::*;
use tokio::sync::RwLock;

type FuseState = Arc<RwLock<HashMap<String, Vec<PokemonData>>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state: FuseState = Default::default();

    let app = Router::new()
        .route("/api/add", post(add_pokemon))
        .route("/api/get", get(get_pokemon))
        .layer(tower_http::compression::CompressionLayer::new())
        .with_state(state);

    axum::Server::bind(&([0, 0, 0, 0], std::env::var("FUSELINK_PORT")?.parse()?).into())
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn add_pokemon(
    State(state): State<FuseState>,
    Json(pokemon): Json<PokemonUploadData>,
) -> StatusCode {
    match state.write().await.entry(pokemon.pass) {
        Entry::Occupied(mut mons) => mons.get_mut().push(pokemon.pokemon),
        Entry::Vacant(e) => {
            e.insert(vec![pokemon.pokemon]);
        }
    }

    StatusCode::NO_CONTENT
}

async fn get_pokemon(
    State(state): State<FuseState>,
    Query(GetPokemonData { pass }): Query<GetPokemonData>,
) -> Result<Json<Vec<PokemonData>>, StatusCode> {
    Ok(Json(
        state
            .read()
            .await
            .get(&pass)
            .ok_or(StatusCode::NOT_FOUND)?
            .to_vec(),
    ))
}
