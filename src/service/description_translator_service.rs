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

    /// Translate the decription field using yoda for cave or legendary pokemon or shakespeare for others.
    /// If translation fails for any reason then leave the original description.
    pub async fn translate_description(&self, pokemon_info: &mut PokemonInfo) {
        println!("translating description: {:?}", pokemon_info);
        let resp = match self
            .client
            .post(format!(
                "{}/{}",
                self.url,
                if pokemon_info.habitat.eq_ignore_ascii_case(CAVE) || pokemon_info.is_legendary {
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
        perform_translation_test(
            PokemonInfo {
                name: "pikachu".to_string(),
                habitat: "forest".to_string(),
                description: "pikachu description ...".to_string(),
                is_legendary: false,
            },
            "shakespeare pikachu translation".to_string(),
            SHAKESPEARE.to_string(),
        )
        .await
    }

    #[tokio::test]
    async fn cave_yoda_translate() -> Result<()> {
        perform_translation_test(
            PokemonInfo {
                name: "zubat".to_string(),
                habitat: CAVE.to_string(),
                description: "zubat description".to_string(),
                is_legendary: false,
            },
            "zubat yoda translation".to_string(),
            YODA.to_string(),
        )
        .await
    }

    #[tokio::test]
    async fn legendary_yoda_translate() -> Result<()> {
        perform_translation_test(
            PokemonInfo {
                name: "mewtwo".to_string(),
                habitat: "rare".to_string(),
                description: "mewtwo description".to_string(),
                is_legendary: true,
            },
            "mewtwo yoda translation".to_string(),
            YODA.to_string(),
        )
        .await
    }

    async fn perform_translation_test(
        pokemon_info: PokemonInfo,
        new_description: String,
        translation_type: String,
    ) -> Result<()> {
        let mock_server = MockServer::start().await;
        let service =
            DescriptionTranslatorService::new_with_url(reqwest::Client::new(), mock_server.uri());

        let req = TranslateRequest {
            text: pokemon_info.description.clone(),
        };
        let resp = TranslateResponse {
            contents: Contents {
                translated: new_description.clone(),
            },
        };

        Mock::given(method("POST"))
            .and(path(format!("/{}", translation_type)))
            .and(body_string(serde_urlencoded::to_string(req)?))
            .respond_with(ResponseTemplate::new(200).set_body_json(resp))
            .expect(1)
            .mount(&mock_server)
            .await;

        let mut translated = pokemon_info.clone();
        service.translate_description(&mut translated).await;

        let expected = PokemonInfo {
            description: new_description,
            ..pokemon_info
        };

        assert_eq!(expected, translated);

        mock_server.verify().await;
        Ok(())
    }

    #[tokio::test]
    /// Error during tranlation leaves descripiton as is
    async fn failed_translation() -> Result<()> {
        let mock_server = MockServer::start().await;
        let service =
            DescriptionTranslatorService::new_with_url(reqwest::Client::new(), mock_server.uri());

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(429))
            .mount(&mock_server)
            .await;

        let mut pokemon_info = PokemonInfo {
            name: "pikachu".to_string(),
            habitat: "forest".to_string(),
            description: "pikachu description ...".to_string(),
            is_legendary: false,
        };

        let expected = pokemon_info.clone();
        service.translate_description(&mut pokemon_info).await;

        assert_eq!(expected, pokemon_info);

        Ok(())
    }
}
