use crate::api::PokemonInfo;
use eyre::{bail, eyre, Result};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const POKEMON_SPECIES_API_URL: &str = "https://pokeapi.co/api/v2/pokemon-species/";
const EN: &str = "en";

pub struct PokemonInfoService {
    client: Client,
    url: String,
}

#[derive(Error, Debug)]
#[error("no pokemon found")]
pub struct NoPokemonFoundError {}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PokemonSpecies {
    name: String,
    habitat: Habitat,
    is_legendary: bool,
    flavor_text_entries: Vec<FlavorText>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Habitat {
    name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct FlavorText {
    flavor_text: String,
    language: Language,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Language {
    name: String,
}

/// Handles getting pokemon info
impl PokemonInfoService {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            url: POKEMON_SPECIES_API_URL.to_string(),
        }
    }

    #[cfg(test)]
    fn new_with_url(client: Client, url: String) -> Self {
        Self { client, url }
    }

    /// Get pokemon info from a name.
    /// Will return wrapped NoPokemonFoundError if cannot find pokemon with given name.
    pub async fn pokemon_info(&self, name: &str) -> Result<PokemonInfo> {
        println!("getting info for pokemon: {}", name);
        let resp = self
            .client
            .get(format!("{}/{}", self.url, name))
            .send()
            .await?;
        match resp.status() {
            StatusCode::NOT_FOUND => bail!(NoPokemonFoundError {}),
            s if !s.is_success() => bail!("error during pokemon lookup: {:?}", s),
            _ => { /* success */ }
        }
        let resp = resp.json::<PokemonSpecies>().await?;

        Ok(PokemonInfo {
            name: resp.name,
            description: resp
                .flavor_text_entries
                .into_iter()
                .find(|ft| ft.language.name.eq_ignore_ascii_case(EN))
                .ok_or_else(|| eyre!("no en flavor text"))?
                .flavor_text
                .lines()
                .collect::<Vec<&str>>()
                .join(" ")
                // lines() doest catch form feed
                .replace('\x0c', " "),
            habitat: resp.habitat.name,
            is_legendary: resp.is_legendary,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::api::PokemonInfo;
    use crate::service::pokemon_info_service::{
        FlavorText, Habitat, Language, NoPokemonFoundError, PokemonSpecies, EN,
    };
    use crate::PokemonInfoService;
    use eyre::Result;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn successful_pokemon_info() -> Result<()> {
        let mock_server = MockServer::start().await;
        let service = PokemonInfoService::new_with_url(reqwest::Client::new(), mock_server.uri());

        let name = "mewtwo".to_string();
        let habitat = "rare".to_string();
        let description = "some description".to_string();

        let resp = get_pokemon_species(
            name.clone(),
            habitat.clone(),
            description.clone(),
            EN.to_string(),
        );

        Mock::given(method("GET"))
            .and(path(format!("/{}", &name)))
            .respond_with(ResponseTemplate::new(200).set_body_json(resp))
            .mount(&mock_server)
            .await;

        let info = service.pokemon_info(&name).await?;
        assert_eq!(
            info,
            PokemonInfo {
                name,
                description,
                habitat,
                is_legendary: true,
            }
        );

        Ok(())
    }

    #[tokio::test]
    async fn pokemon_not_found() -> Result<()> {
        let mock_server = MockServer::start().await;
        let service = PokemonInfoService::new_with_url(reqwest::Client::new(), mock_server.uri());

        let name = "mewtwo".to_string();

        Mock::given(method("GET"))
            .and(path(format!("/{}", &name)))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let info = service.pokemon_info(&name).await;

        assert!(info.is_err());
        assert!(info.unwrap_err().is::<NoPokemonFoundError>());

        Ok(())
    }

    #[tokio::test]
    async fn no_en_description() -> Result<()> {
        let mock_server = MockServer::start().await;
        let service = PokemonInfoService::new_with_url(reqwest::Client::new(), mock_server.uri());

        let name = "mewtwo".to_string();
        let habitat = "rare".to_string();
        let description = "konichwa, none en description".to_string();

        let resp = get_pokemon_species(
            name.clone(),
            habitat.clone(),
            description.clone(),
            "jp".to_string(),
        );

        Mock::given(method("GET"))
            .and(path(format!("/{}", &name)))
            .respond_with(ResponseTemplate::new(200).set_body_json(resp))
            .mount(&mock_server)
            .await;

        let info = service.pokemon_info(&name).await;
        assert!(info.is_err());
        assert!(!info.unwrap_err().is::<NoPokemonFoundError>());

        Ok(())
    }

    fn get_pokemon_species(
        name: String,
        habitat: String,
        description: String,
        language: String,
    ) -> PokemonSpecies {
        PokemonSpecies {
            name: name,
            habitat: Habitat { name: habitat },
            is_legendary: true,
            flavor_text_entries: vec![FlavorText {
                flavor_text: description,
                language: Language { name: language },
            }],
        }
    }
}
