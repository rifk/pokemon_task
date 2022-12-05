use crate::service::{NoPokemonFoundError, PokemonInfoService};
use axum::http::StatusCode;
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use std::sync::Arc;

/// Response json for GET /pokemon/{name}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PokemonInfo {
    pub name: String,
    pub description: String,
    pub habitat: String,
    pub is_legendary: bool,
}

/// Setup routing
pub fn routes(pokemon_info_service: PokemonInfoService) -> Router {
    Router::new().route(
        "/pokemon/:name",
        get(get_by_name).with_state(Arc::new(pokemon_info_service)),
    )
}

/// Handler for GET /pokemon/{name}
async fn get_by_name(
    State(pokemon_info_service): State<Arc<PokemonInfoService>>,
    Path(name): Path<String>,
) -> Result<Json<PokemonInfo>, StatusCode> {
    pokemon_info_service
        .pokemon_info(&name)
        .await
        .map(Json)
        .map_err(error_to_status)
}

fn error_to_status(err: eyre::Error) -> StatusCode {
    match err.downcast_ref::<NoPokemonFoundError>() {
        Some(_) => {
            println!("pokemon not found");
            StatusCode::NOT_FOUND
        }
        _ => {
            println!("internal server error, returing 500: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
