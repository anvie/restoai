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
    streamer::{StreamChannel, StreamWriterBytes, Streamer},
};

type OAIAppContext = AppContext<OpenAiBackend>;

lazy_static! {
    static ref AVAILABLE_MODELS: Vec<&'static str> = vec!["programmer", "sysadmin",];
}

// #[derive(Debug, Serialize, Deserialize)]
// #[serde(tag = "role")]
// pub enum ChatMessage<'a> {
//     #[serde(rename = "system")]
//     System {
//         content: Option<Cow<'a, str>>,
//         name: Option<Cow<'a, str>>,
//     },
// }
//
// #[derive(Debug, Serialize, Deserialize, Default, Deref, DerefMut, From)]
// pub struct ChatMessages<'a>(
//     #[deref]
//     #[deref_mut]
//     Vec<ChatMessage<'a>>,
// );
//
// #[derive(Debug, Serialize, Deserialize)]
// pub struct CreateChatCompletionRequest<'a> {
//     #[serde(default)]
//     pub messages: ChatMessages<'a>,
//     pub max_tokens: Option<u32>,
//     pub temperature: Option<f32>,
//     pub top_p: Option<f32>,
//     pub presence_penalty: Option<f32>,
//     pub frequency_penalty: Option<f32>,
//     pub stream: Option<bool>,
//     pub one_shot: Option<bool>,
//     pub n: Option<u32>,
//     pub model: Cow<'a, str>,
//     pub seed: Option<u32>,
//
//     #[serde(default, with = "either::serde_untagged_optional")]
//     pub stop: Option<Either<Cow<'a, str>, Vec<Cow<'a, str>>>>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct ModelList<'a> {
//     pub object: Cow<'a, str>,
//     pub data: Vec<Model>,
// }

// type ChatCompletionChoices<'a> = Vec<ChatCompletionChoice<'a>>;
//
// impl From<Vec<openai_dive::v1::resources::chat::ChatCompletionChoice>> for ChatCompletionChoices<'_> {
//     fn from(choices: Vec<openai_dive::v1::resources::chat::ChatCompletionChoice>) -> Self {
//         choices.into_iter().map(ChatCompletionChoice::from).collect()
//     }
// }

//
// enum ChatCompletionResponse<'a, S>
// where
//     S: TryStream<Ok = Event> + Send + 'static,
// {
//     Full(Json<ChatCompletion<'a>>),
//     Stream(Sse<S>),
// }
//

#[derive(Debug, Deserialize)]
struct TestData {
    pub name: String,
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

// /// Inspector endpoint
// #[post("/chat/completions_")]
// pub async fn chat_completions_(
//     data: web::Json<serde_json::Value>,
//     ctx: web::Data<OAIAppContext>,
//     credential: BearerAuth,
// ) -> impl Responder {
//     // log print inside data
//     //println!("{:?}", serde_json::to_string(&data).ok());
//     println!(
//         "{}",
//         serde_json::to_string(&data)
//             .ok()
//             .unwrap_or("error to_string".to_string())
//     );
//
//     let chat_params =
//         serde_json::from_value::<apitype::ChatCompletionParameters>(data.into_inner())
//             .expect("Cannot parse json");
//
//     println!("chat_params:\n {:?}", chat_params);
//
//     HttpResponse::Ok().body("Sent.")
// }

#[post("/chat/completions")]
pub async fn chat_completions(
    req: HttpRequest,
    data: web::Json<apitype::ChatCompletionParameters>,
    ctx: web::Data<OAIAppContext>,
    credential: BearerAuth,
    //tx_closer: web::Data<apitype::ClientCloser>,
) -> impl Responder {
    // let socket = req.tcp_stream();
    // if let Some(socket) = socket {
    //     trace!("Request from: {}", socket);
    // }

    if !is_model_supported(&data.model) {
        //return Err(actix_web::error::ErrorBadRequest("Model not supported"));
        //
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
        // let stream_channel: StreamChannel = ctx.streamer.new_client().await;
        // let writer = stream_channel.get_stream_writer();

        let (tx, mut rx) = mpsc::channel(10);
        let writer = StreamWriterBytes(Arc::new(tx));

        let llm_backend = ctx.llm_backend.clone();

        tokio::spawn(async move {
            llm_backend
                .submit_prompt_stream(messages, writer, &data.model)
                .await;

            // close socket
            // if let Some(s) = socket {
            //     trace!("Closing socket: {}", s);
            //     tx_closer.as_ref().0.send(s).await.unwrap();
            // }
        });

        //HttpResponse::Ok().body(stream_channel.stream)

        //let rx = Arc::new(Mutex::new(rx));

        HttpResponse::build(StatusCode::OK)
            .insert_header(("Content-Type", "text/event-stream"))
            .insert_header(("Cache-Control", "no-cache"))
            .streaming(Box::pin(async_stream::stream! {
                //let rx = rx.clone();
                //let mut rx = rx.lock();

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

#[post("/chat/broadcast")]
pub async fn broadcast(data: web::Json<TestData>, ctx: web::Data<OAIAppContext>) -> impl Responder {
    ctx.streamer
        .broadcast(&format!("hello {}!", &data.name))
        .await;
    HttpResponse::Ok().body("Sent.")
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

// pub async fn chat_completions(Json(req) = Json<CreateChatCompletionRequest<'_>>) -> Result<impl IntoResponse, ChatCompletionError> {
//
//
//
// }
//
