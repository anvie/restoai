use actix_web::{
    get,
    http::StatusCode,
    post,
    web::{self, Json},
    HttpRequest, HttpResponse, Responder,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use derive_more::{Deref, DerefMut, From};
use either::Either;
use futures::{Stream, StreamExt, TryStream};
use openai_dive::v1::resources::{
    chat::{ChatCompletionParameters, ChatMessage, DeltaToolCall, Role},
    model::ListModelResponse,
    shared::FinishReason,
};
use parking_lot::Mutex;
use serde_derive::{self, Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;

use std::borrow::Cow;

use crate::{
    apitype,
    appctx::AppContext,
    llm::{LlmBackend, OpenAiBackend},
    streamer::StreamWriter,
};

type OAIAppContext = AppContext<OpenAiBackend>;

lazy_static! {
    static ref AVAILABLE_MODELS: Vec<&'static str> = vec!["programmer", "sysadmin",];
}

#[derive(Debug, Serialize, Deserialize)]
struct HitCounter {
    pub token: String,
    pub hits: u32,
}

pub fn track_metric_counter(path: &str, token: &str, ctx: &OAIAppContext) {
    let db = ctx.db.clone();
    let mut db = db.lock().unwrap();
    let mut hits: HashMap<String, u32> = HashMap::new();
    if let Some(val) = db.get::<Vec<HitCounter>>(path) {
        let mut exists = false;
        val.iter().for_each(|v| {
            if v.token == token {
                let cnt = v.hits + 1;
                hits.insert(token.to_string(), cnt);

                debug!("{} - {} hits", path, cnt);
                exists = true;
            } else {
                hits.insert(v.token.clone(), v.hits);
            }
        });
        if !exists {
            hits.insert(token.to_string(), 1);
        }
    } else {
        hits.insert(token.to_string(), 1);
    }

    let hits_data: Vec<HitCounter> = hits
        .iter()
        .map(|(k, v)| HitCounter {
            token: k.clone(),
            hits: *v,
        })
        .collect();

    db.set(path, &json!(hits_data)).unwrap();
}

fn is_model_supported(model: &str) -> bool {
    AVAILABLE_MODELS.contains(&model)
}

#[post("/chat/completions")]
pub async fn chat_completions(
    data: web::Json<apitype::ChatCompletionParameters>,
    ctx: web::Data<OAIAppContext>,
    credential: BearerAuth,
) -> impl Responder {
    if !is_model_supported(&data.model) {
        return HttpResponse::BadRequest().body("Model not supported");
    }

    // log metric for the current credential
    track_metric_counter("/chat/completions", credential.token(), &ctx);

    let messages = data
        .messages
        .iter()
        .map(|m| ChatMessage {
            role: m.role.clone(),
            content: m.content.clone().into(),
            name: m.name.clone(),
            ..Default::default()
        })
        .collect();

    if data.stream == Some(true) {
        let (tx, mut rx) = mpsc::channel(10);
        let writer = StreamWriter(Arc::new(tx));

        let llm_backend = ctx.llm_backend.clone();

        tokio::spawn(async move {
            llm_backend
                .submit_prompt_stream(messages, writer, &data.model)
                .await;
        });

        HttpResponse::build(StatusCode::OK)
            .insert_header(("Content-Type", "text/event-stream"))
            .insert_header(("Cache-Control", "no-cache"))
            .streaming(Box::pin(async_stream::stream! {
                while let Some(event) = rx.recv().await {
                    debug!("++Event: {}", event);
                    yield Ok::<_,actix_web::error::Error>(web::Bytes::from(["data: ", &event, "\n\n"].concat()));
                }

                // send [DONE] message
                yield Ok::<_,actix_web::error::Error>(web::Bytes::from("data: [DONE]\n\n"));

                trace!("[*] STREAM CLOSED.");
            }))
    } else {
        HttpResponse::Ok().json(ctx.llm_backend.submit_prompt(messages, &data.model).await)
    }
}

#[get("/models")]
pub async fn models(ctx: web::Data<OAIAppContext>) -> impl Responder {
    let models = ctx.llm_backend.models().await;

    let models = apitype::ListModelResponse {
        object: models.object,
        // data: models
        //     .data
        //     .into_iter()
        //     .map(|d| openai_dive::v1::resources::model::Model {
        //         id: d.id,
        //         object: d.object,
        //         created: d.created,
        //         owned_by: d.owned_by,
        //     })
        //     .collect(),
        data: AVAILABLE_MODELS
            .iter()
            .map(|m| apitype::Model {
                id: (*m).into(),
                object: "model".into(),
                created: 0,
                owned_by: None,
            })
            .collect(),
    };

    HttpResponse::Ok().json(models)
}
