use crate::api::PokemonInfo;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const TRANSLATE_API_URL: &str = "https://api.funtranslations.com/translate";
const YODA: &str = "yoda.json";
const SHAKESPEARE: &str = "shakespeare.json";
const CAVE: &str = "cave";

pub struct DescriptionTranslatorService {
    client: Client,
    url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct TranslateRequest {
    text: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct TranslateResponse {
    contents: Contents,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Contents {
    translated: String,
}

/// Handles translating description
impl DescriptionTranslatorService {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            url: TRANSLATE_API_URL.to_string(),
        }
    }

    #[cfg(test)]
    fn new_with_url(client: Client, url: String) -> Self {
        Self { client, url }
    }

    /// Translate the decription field using yoda for cave pokemon or shakespeare for others.
    /// If translation fails for any reason then leave the original description.
    pub async fn translate_description(&self, pokemon_info: &mut PokemonInfo) {
        println!("translating description: {:?}", pokemon_info);
        let resp = match self
            .client
            .post(format!(
                "{}/{}",
                self.url,
                if pokemon_info.habitat.eq_ignore_ascii_case(CAVE) {
                    YODA
                } else {
                    SHAKESPEARE
                }
            ))
            .form(&TranslateRequest {
                text: pokemon_info.description.clone(),
            })
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(err) => {
                println!("error requesting translation: {:?}", err);
                return;
            }
        };

        if !resp.status().is_success() {
            println!(
                "unsuccessful response from translation: {:?}",
                resp.status()
            );
            return;
        }

        let resp = match resp.json::<TranslateResponse>().await {
            Ok(resp) => resp,
            Err(err) => {
                println!("error deserializing json translate response: {:?}", err);
                return;
            }
        };

        pokemon_info.description = resp.contents.translated;
    }
}

#[cfg(test)]
mod tests {
    use crate::api::PokemonInfo;
    use crate::service::description_translator_service::{
        Contents, TranslateRequest, TranslateResponse, CAVE, SHAKESPEARE, YODA,
    };
    use crate::DescriptionTranslatorService;
    use eyre::Result;
    use wiremock::matchers::{body_string, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn shakespeare_translate() -> Result<()> {
        let mock_server = MockServer::start().await;
        let service =
            DescriptionTranslatorService::new_with_url(reqwest::Client::new(), mock_server.uri());

        let description = "original desc".to_string();
        let new_description = "shakespear desc".to_string();

        let req = TranslateRequest {
            text: description.clone(),
        };
        let resp = TranslateResponse {
            contents: Contents {
                translated: new_description.clone(),
            },
        };

        Mock::given(method("POST"))
            .and(path(format!("/{}", &SHAKESPEARE)))
            .and(body_string(serde_urlencoded::to_string(req)?))
            .respond_with(ResponseTemplate::new(200).set_body_json(resp))
            .mount(&mock_server)
            .await;

        let pokemon_info = PokemonInfo {
            name: "mewtwo".to_string(),
            habitat: "rare".to_string(),
            description,
            is_legendary: true,
        };
        let mut pokemon_info_new = pokemon_info.clone();

        service.translate_description(&mut pokemon_info_new).await;
        assert_eq!(pokemon_info_new.description, new_description);
        assert_eq!(pokemon_info_new.name, pokemon_info.name);
        assert_eq!(pokemon_info_new.habitat, pokemon_info.habitat);
        assert_eq!(pokemon_info_new.is_legendary, pokemon_info.is_legendary);

        Ok(())
    }

    #[tokio::test]
    async fn yoda_translate() -> Result<()> {
        let mock_server = MockServer::start().await;
        let service =
            DescriptionTranslatorService::new_with_url(reqwest::Client::new(), mock_server.uri());

        let description = "original desc".to_string();
        let new_description = "yoda desc".to_string();

        let req = TranslateRequest {
            text: description.clone(),
        };
        let resp = TranslateResponse {
            contents: Contents {
                translated: new_description.clone(),
            },
        };

        Mock::given(method("POST"))
            .and(path(format!("/{}", &YODA)))
            .and(body_string(serde_urlencoded::to_string(req)?))
            .respond_with(ResponseTemplate::new(200).set_body_json(resp))
            .mount(&mock_server)
            .await;

        let pokemon_info = PokemonInfo {
            name: "zubat".to_string(),
            habitat: CAVE.to_string(),
            description,
            is_legendary: false,
        };
        let mut pokemon_info_new = pokemon_info.clone();

        service.translate_description(&mut pokemon_info_new).await;

        assert_eq!(pokemon_info_new.description, new_description);
        assert_eq!(pokemon_info_new.name, pokemon_info.name);
        assert_eq!(pokemon_info_new.habitat, pokemon_info.habitat);
        assert_eq!(pokemon_info_new.is_legendary, pokemon_info.is_legendary);

        Ok(())
    }
}
