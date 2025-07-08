use std::{collections::HashMap, sync::Arc};

use common::{
    configuration::{LlmProvider, LlmRoute, ModelUsagePreference},
    consts::ARCH_PROVIDER_HINT_HEADER,
};
use hermesllm::providers::openai::types::{ChatCompletionsResponse, ContentType, Message};
use hyper::header;
use thiserror::Error;
use tracing::{debug, info, warn};

use crate::router::router_model_v1::{self};

use super::router_model::RouterModel;

pub struct RouterService {
    router_url: String,
    client: reqwest::Client,
    router_model: Arc<dyn RouterModel>,
    routing_provider_name: String,
    llm_usage_defined: bool,
    llm_provider_map: HashMap<String, LlmProvider>,
}

#[derive(Debug, Error)]
pub enum RoutingError {
    #[error("Failed to send request: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Failed to parse JSON: {0}, JSON: {1}")]
    JsonError(serde_json::Error, String),

    #[error("Router model error: {0}")]
    RouterModelError(#[from] super::router_model::RoutingModelError),
}

pub type Result<T> = std::result::Result<T, RoutingError>;

impl RouterService {
    pub fn new(
        providers: Vec<LlmProvider>,
        router_url: String,
        routing_model_name: String,
        routing_provider_name: String,
    ) -> Self {
        let providers_with_usage = providers
            .iter()
            .filter(|provider| provider.usage.is_some())
            .cloned()
            .collect::<Vec<LlmProvider>>();

        let llm_routes: Vec<LlmRoute> = providers_with_usage.iter().map(LlmRoute::from).collect();

        let router_model = Arc::new(router_model_v1::RouterModelV1::new(
            llm_routes,
            routing_model_name.clone(),
            router_model_v1::MAX_TOKEN_LEN,
        ));

        let llm_provider_map: HashMap<String, LlmProvider> = providers
            .into_iter()
            .map(|provider| (provider.name.clone(), provider))
            .collect();

        RouterService {
            router_url,
            client: reqwest::Client::new(),
            router_model,
            routing_provider_name,
            llm_usage_defined: !providers_with_usage.is_empty(),
            llm_provider_map,
        }
    }

    pub async fn determine_route(
        &self,
        messages: &[Message],
        trace_parent: Option<String>,
        usage_preferences: Option<Vec<ModelUsagePreference>>,
    ) -> Result<Option<String>> {
        if !self.llm_usage_defined {
            return Ok(None);
        }

        let router_request = self
            .router_model
            .generate_request(messages, &usage_preferences);

        info!(
            "sending request to arch-router model: {}, endpoint: {}",
            self.router_model.get_model_name(),
            self.router_url
        );

        debug!(
            "arch request body: {}",
            &serde_json::to_string(&router_request).unwrap(),
        );

        let mut llm_route_request_headers = header::HeaderMap::new();
        llm_route_request_headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        llm_route_request_headers.insert(
            header::HeaderName::from_static(ARCH_PROVIDER_HINT_HEADER),
            header::HeaderValue::from_str(&self.routing_provider_name).unwrap(),
        );

        if let Some(trace_parent) = trace_parent {
            llm_route_request_headers.insert(
                header::HeaderName::from_static("traceparent"),
                header::HeaderValue::from_str(&trace_parent).unwrap(),
            );
        }

        llm_route_request_headers.insert(
            header::HeaderName::from_static("model"),
            header::HeaderValue::from_static("arch-router"),
        );

        let start_time = std::time::Instant::now();
        let res = self
            .client
            .post(&self.router_url)
            .headers(llm_route_request_headers)
            .body(serde_json::to_string(&router_request).unwrap())
            .send()
            .await?;

        let body = res.text().await?;
        let router_response_time = start_time.elapsed();

        let chat_completion_response: ChatCompletionsResponse = match serde_json::from_str(&body) {
            Ok(response) => response,
            Err(err) => {
                warn!(
                    "Failed to parse JSON: {}. Body: {}",
                    err,
                    &serde_json::to_string(&body).unwrap()
                );
                return Err(RoutingError::JsonError(
                    err,
                    format!("Failed to parse JSON: {}", body),
                ));
            }
        };

        if chat_completion_response.choices.is_empty() {
            warn!("No choices in router response: {}", body);
            return Ok(None);
        }

        if let Some(ContentType::Text(content)) =
            &chat_completion_response.choices[0].message.content
        {
            let mut selected_model: Option<String> = None;
            if let Some(selected_llm_name) = self.router_model.parse_response(content)? {
                if selected_llm_name != "other" {
                    if let Some(usage_preferences) = usage_preferences {
                        for usage in usage_preferences {
                            if usage.name == selected_llm_name {
                                selected_model = Some(usage.model);
                                break;
                            }
                        }
                        if selected_model.is_none() {
                            warn!(
                                "Selected LLM model not found in usage preferences: {}",
                                selected_llm_name
                            );
                        }
                    } else if let Some(provider) = self.llm_provider_map.get(&selected_llm_name) {
                        selected_model = provider.model.clone();
                    } else {
                        warn!(
                            "Selected LLM model not found in provider map: {}",
                            selected_llm_name
                        );
                    }
                }
            }
            info!(
                "router response: {}, selected_model: {:?}, response time: {}ms",
                content.replace("\n", "\\n"),
                selected_model,
                router_response_time.as_millis()
            );

            Ok(selected_model)
        } else {
            Ok(None)
        }
    }
}
