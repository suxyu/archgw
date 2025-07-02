use bytes::Bytes;
use common::configuration::{LlmProvider, ModelUsagePreference};
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::{Request, Response, StatusCode};
use serde_json;
use std::{collections::HashMap, sync::Arc};
use tracing::{info, warn};

pub async fn list_preferences(
    llm_providers: Arc<tokio::sync::RwLock<Vec<LlmProvider>>>,
) -> Response<BoxBody<Bytes, hyper::Error>> {
    let prov = llm_providers.read().await;
    // convert the LlmProvider to UsageBasedProvider
    let providers_with_usage = prov
        .iter()
        .map(|provider| ModelUsagePreference {
            name: provider.name.clone(),
            model: provider.model.clone().unwrap_or_default(),
            usage: provider.usage.clone(),
        })
        .collect::<Vec<ModelUsagePreference>>();

    match serde_json::to_string(&providers_with_usage) {
        Ok(json) => {
            let body = Full::new(Bytes::from(json))
                .map_err(|never| match never {})
                .boxed();
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(body)
                .unwrap()
        }
        Err(_) => {
            let body = Full::new(Bytes::from_static(
                b"{\"error\":\"Failed to serialize models\"}",
            ))
            .map_err(|never| match never {})
            .boxed();
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(body)
                .unwrap()
        }
    }
}

pub async fn update_preferences(
    request: Request<hyper::body::Incoming>,
    llm_providers: Arc<tokio::sync::RwLock<Vec<LlmProvider>>>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let request_body = request.collect().await?.to_bytes();

    let usage: Vec<ModelUsagePreference> = match serde_json::from_slice(&request_body) {
        Ok(usage) => usage,
        Err(_) => {
            let response_body = Full::new(Bytes::from_static(b"Invalid request body: "))
                .map_err(|never| match never {})
                .boxed();
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "text/plain")
                .body(response_body)
                .unwrap());
        }
    };

    let usage_model_map: HashMap<String, ModelUsagePreference> =
        usage.into_iter().map(|u| (u.model.clone(), u)).collect();

    info!(
        "Updating usage preferences for models: {:?}",
        usage_model_map.keys()
    );

    let mut llm_providers = llm_providers.write().await;

    // ensure that models coming in the request are valid
    let llm_provider_names: Vec<String> = llm_providers
        .iter()
        .map(|provider| provider.name.clone())
        .collect();

    for model in usage_model_map.keys() {
        if !llm_provider_names.contains(model) {
            let model_not_found = format!("model not found: {}", model);
            warn!("updating preferences: {}", model_not_found);
            let response_body = Full::new(model_not_found.into())
                .map_err(|never| match never {})
                .boxed();
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "text/plain")
                .body(response_body)
                .unwrap());
        }
    }

    let mut updated_models_list = Vec::new();
    for provider in llm_providers.iter_mut() {
        if let Some(usage_provider) = usage_model_map.get(&provider.name) {
            provider.usage = usage_provider.usage.clone();
            updated_models_list.push(ModelUsagePreference {
                name: provider.name.clone(),
                model: provider.model.clone().unwrap_or_default(),
                usage: provider.usage.clone(),
            });
        }
    }

    if !updated_models_list.is_empty() {
        // return list of updated models
        let response_body = Full::new(Bytes::from(format!(
            "{{\"updated_models\": {}}}",
            serde_json::to_string(&updated_models_list).unwrap()
        )))
        .map_err(|never| match never {})
        .boxed();
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(response_body)
            .unwrap())
    } else {
        let response_body = Full::new(Bytes::from_static(b"Provider not found"))
            .map_err(|never| match never {})
            .boxed();
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "text/plain")
            .body(response_body)
            .unwrap())
    }
}
