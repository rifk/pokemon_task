use crate::service::{DescriptionTranslatorService, PokemonInfoService};
use eyre::Result;
use reqwest::ClientBuilder;
use std::net::SocketAddr;
use std::time::Duration;

mod api;
mod service;

#[tokio::main]
async fn main() -> Result<()> {
    let client = ClientBuilder::new()
        // timout value could be configurable or need tuning in prod
        .timeout(Duration::from_secs(10))
        .build()?;

    let pokemon_info_service = PokemonInfoService::new(client.clone());
    let description_translator_service = DescriptionTranslatorService::new(client);
    let routes = api::routes(pokemon_info_service, description_translator_service);

    println!("starting server listening 127.0.0.1:5000");
    let addr = SocketAddr::from(([127, 0, 0, 1], 5000));
    // prod should handle server shutdown gracefully
    axum::Server::bind(&addr)
        .serve(routes.into_make_service())
        .await?;

    Ok(())
}
