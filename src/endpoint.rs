use actix_web::{
    get, post,
    web::{self, Json},
    HttpRequest, HttpResponse, Responder,
};
use derive_more::{Deref, DerefMut, From};
use either::Either;
use futures::{Stream, StreamExt, TryStream};
use openai_dive::v1::resources::{
    chat::{ChatCompletionParameters, ChatMessage, DeltaToolCall, Role},
    model::ListModelResponse,
    shared::FinishReason,
};
use serde_derive::{self, Deserialize, Serialize};

use std::borrow::Cow;

use crate::{
    apitype,
    appctx::AppContext,
    llm::{LlmBackend, OpenAiBackend},
    streamer::{StreamChannel, Streamer},
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

#[post("/chat/completions")]
pub async fn chat_completions(data: web::Json<ChatCompletionParameters>, ctx: web::Data<OAIAppContext>) -> impl Responder {
    if data.stream == Some(true) {
        let stream_channel: StreamChannel = ctx.streamer.new_client().await;
        let writer = stream_channel.get_stream_writer();

        let messages = data
            .messages
            .iter()
            .map(|m| ChatMessage {
                role: m.role.clone(),
                content: m.content.clone(),
                name: m.name.clone(),
                ..Default::default()
            })
            .collect();

        let llm_backend = ctx.llm_backend.clone();

        tokio::spawn(async move {
            llm_backend.submit_prompt_stream(messages, writer).await;
        });

        HttpResponse::Ok().body(stream_channel.stream)
    } else {
        HttpResponse::Ok().json(ctx.llm_backend.submit_prompt(data.messages.clone()).await)
    }
}

#[post("/chat/broadcast")]
pub async fn broadcast(data: web::Json<TestData>, ctx: web::Data<OAIAppContext>) -> impl Responder {
    ctx.streamer.broadcast(&format!("hello {}!", &data.name)).await;
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
