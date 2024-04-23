use reqwest::{header, Client, Error};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SignupResponse {
    pub code_delivery_details: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GetAIGenerationFeedGeneratedImage {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub enum GetAIGenerationFeedGenerationStatus {
    COMPLETE,
    FAILED,
}

#[derive(Debug, Deserialize)]
pub struct GetAIGenerationFeedGeneration {
    pub generated_images: Vec<GetAIGenerationFeedGeneratedImage>,
    pub status: GetAIGenerationFeedGenerationStatus,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GetAIGenerationFeedResponse {
    pub generations: Vec<GetAIGenerationFeedGeneration>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GraphqlError {
    pub message: String,
}

#[derive(Debug)]
pub enum GraphqlRequestError {
    UnexpectedResponseFormat,
    GraphqlError(GraphqlError),
    NetworkError(reqwest::Error),
}

impl From<reqwest::Error> for GraphqlRequestError {
    fn from(err: reqwest::Error) -> Self {
        GraphqlRequestError::NetworkError(err)
    }
}

#[derive(std::default::Default)]
pub struct Leonardo {
    client: Client,
    user_id: Option<String>,
}

impl Leonardo {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            client: Client::builder().cookie_store(true).timeout(std::time::Duration::from_secs(30)).build()?,
            user_id: None,
        })
    }

    pub async fn signup(&mut self, email: String, password: &str) -> Result<SignupResponse, Error> {
        let response = self
            .client
            .post("https://app.leonardo.ai/api/auth/signup")
            .json(&json!({"email": email, "password": password}))
            .send()
            .await?
            .json::<SignupResponse>()
            .await?;

        Ok(response)
    }

    pub async fn confirm_signup(
        &mut self,
        email: String,
        password: &str,
        confirmation_code: String,
    ) -> Result<(), Error> {
        self.client
            .post("https://app.leonardo.ai/api/auth/confirm-signup")
            .json(&json!({
                "email":email,
                "password":password,
                "confirmation_code":confirmation_code,
            }))
            .send()
            .await?;

        Ok(())
    }

    pub async fn login(&mut self, username: String, password: &str) -> Result<(), Error> {
        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields, rename_all = "camelCase")]
        struct CsrfResponse {
            csrf_token: String,
        }

        let csrf_response = self
            .client
            .get("https://app.leonardo.ai/api/auth/csrf")
            .send()
            .await?
            .json::<CsrfResponse>()
            .await?;

        self.client
            .post("https://app.leonardo.ai/api/auth/callback/credentials")
            .json(&serde_json::json!({
                "username": username,
                "password": password,
                "redirect": false,
                "csrfToken": csrf_response.csrf_token,
                "callbackUrl": "https://app.leonardo.ai/api/auth/session".to_string(),
                "json":true
            }))
            .send()
            .await?;

        self.get_session().await?;

        Ok(())
    }

    async fn get_session(&mut self) -> Result<String, Error> {
        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct GetSessionResponse {
            access_token: String,
        }

        let session_response = self
            .client
            .get("https://app.leonardo.ai/api/auth/session")
            .send()
            .await?
            .json::<GetSessionResponse>()
            .await?;

        let access_token = session_response.access_token;

        Ok(access_token)
    }

    async fn graphql<T, U: Serialize>(
        &mut self,
        operation_name: &str,
        query: &str,
        variables: Option<U>,
    ) -> Result<T, GraphqlRequestError>
    where
        T: DeserializeOwned,
        U: Serialize,
    {
        #[derive(Debug, Deserialize)]
        struct GraphQLResponse<T> {
            data: Option<T>,
            errors: Option<Vec<GraphqlError>>,
        }

        let access_token = self.get_session().await?;

        let response = self
            .client
            .post("https://api.leonardo.ai/v1/graphql")
            .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
            .json(&json!({
                "query": query,
                "operationName": operation_name,
                "variables": variables,
            }))
            .send()
            .await?;

        let response_json = response.json::<GraphQLResponse<T>>().await?;

        let result = match response_json {
            GraphQLResponse {
                data: Some(data),
                errors: None,
            } => Ok(data),
            GraphQLResponse {
                data: None,
                errors: Some(errors),
            } => Err(GraphqlRequestError::GraphqlError(
                errors.get(0).unwrap().clone(),
            )),
            _ => Err(GraphqlRequestError::UnexpectedResponseFormat),
        };

        result
    }

    pub async fn update_username(&mut self, username: String) -> Result<(), GraphqlRequestError> {
        #[derive(Debug, Deserialize)]
        struct ResponseUpdateUsername {
            id: String,
        }

        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Response {
            update_username: ResponseUpdateUsername,
        }

        let query = "
            mutation UpdateUsername($arg1: UpdateUsernameInput!) {
                updateUsername(arg1: $arg1) {
                    id
                    __typename
                }
            }
        ";

        let response = self
            .graphql::<Response, serde_json::Value>(
                "UpdateUsername",
                query,
                Some(json!({
                    "arg1": {
                        "username": username
                    }
                })),
            )
            .await?;

        self.user_id = Some(response.update_username.id);

        Ok(())
    }

    pub async fn update_user_details(&mut self) -> Result<(), GraphqlRequestError> {
        let query = "
            mutation UpdateUserDetails(
                $where: user_details_bool_exp!
                $_set: user_details_set_input
            ) {
                update_user_details(where: $where, _set: $_set) {
                    affected_rows
                    __typename
                }
            }
        ";

        self.graphql::<serde_json::Value, serde_json::Value>(
            "UpdateUserDetails",
            query,
            Some(json!({
                "where": {
                    "userId": {
                        "_eq": self.user_id.clone().unwrap()
                    }
                },
                "_set": {
                    "showNsfw": true,
                    "interests": vec!["OTHER".to_string()]
                }
            })),
        )
        .await?;

        Ok(())
    }

    pub async fn start_user_alchemy_trial(&mut self) -> Result<(), GraphqlRequestError> {
        let query = "
            mutation StartUserAlchemyTrial {
                startUserAlchemyTrial {
                    id
                    isInTrialPeriod
                    hasReachedDailyLimit
                    __typename
                }
            }
        ";

        self.graphql::<serde_json::Value, serde_json::Value>("StartUserAlchemyTrial", query, None)
            .await?;

        Ok(())
    }

    pub async fn create_sd_generation_job(
        &mut self,
        prompt: String,
        num_images: u8,
        width: u32,
        height: u32,
    ) -> Result<String, GraphqlRequestError> {
        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields, rename_all = "camelCase")]
        struct SdGenerationJobData {
            generation_id: String,
        }

        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields, rename_all = "camelCase")]
        struct CreateSDGenerationJobResponse {
            sd_generation_job: SdGenerationJobData,
        }

        let query = "
            mutation CreateSDGenerationJob($arg1: SDGenerationInput!) {
                sdGenerationJob(arg1: $arg1) {
                    generationId
                }
            }
        ";

        let result = self
            .graphql::<CreateSDGenerationJobResponse, serde_json::Value>(
                "CreateSDGenerationJob",
                query,
                Some(json!({
                  "arg1": {
                    "negative_prompt": "",
                    "nsfw": true,
                    "num_inference_steps": 10,
                    "guidance_scale": 15,
                    "sd_version": "SDXL_0_9",
                    "presetStyle": "CINEMATIC",
                    "scheduler": "LEONARDO",
                    "public": true,
                    "tiling": false,
                    "leonardoMagic": false,
                    "alchemy": true,
                    "highResolution": false,
                    "contrastRatio": 0.5,
                    "poseToImage": false,
                    "poseToImageType": "POSE",
                    "weighting": 0.75,
                    "highContrast": true,
                    "expandedDomain": true,
                    "elements": [],
                    "controlnets": [],
                    "photoReal": true,
                    "photoRealVersion": "v2",
                    "transparency": "disabled",
                    "modelId": "aa77f04e-3eec-4034-9c07-d0f619684628",
                    "prompt": prompt,
                    "num_images": num_images,
                    "width": width,
                    "height": height,
                  },
                })),
            )
            .await?;

        Ok(result.sd_generation_job.generation_id)
    }

    pub async fn get_ai_generation_job(
        &mut self,
        generation_id: String,
    ) -> Result<GetAIGenerationFeedResponse, GraphqlRequestError> {
        let query = "
            query GetAIGenerationFeed(
                $where: generations_bool_exp = {}
                $userId: uuid
                $limit: Int
                $offset: Int = 0
            ) {
                generations(
                    limit: $limit
                    offset: $offset
                    order_by: [{ createdAt: desc }]
                    where: $where
                ) {
                    id
                    status
                    generated_images(order_by: [{ url: desc }]) {
                        id
                        url
                    }
                }
            }
        ";

        let result = self
            .graphql::<GetAIGenerationFeedResponse, serde_json::Value>(
                "GetAIGenerationFeed",
                query,
                Some(json!({
                        "where": {
                        "userId": {
                          "_eq": self.user_id,
                        },
                        "teamId": {
                          "_is_null": true,
                        },
                        "status": {
                          "_in": vec!["COMPLETE", "FAILED"],
                        },
                        "id": {
                          "_in": vec![generation_id],
                        },
                        "isStoryboard": {
                          "_eq": false,
                        },
                      },
                      "offset": 0

                })),
            )
            .await?;

        Ok(result)
    }
}
