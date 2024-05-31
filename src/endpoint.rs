use actix_web::{
    get, post,
    web::{self, Json},
    HttpRequest, HttpResponse, Responder,
};
use derive_more::{Deref, DerefMut, From};
use either::Either;
use futures::{Stream, StreamExt, TryStream};
use serde_derive::{self, Deserialize, Serialize};

use std::borrow::Cow;

use crate::streamer::Streamer;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "role")]
pub enum ChatMessage<'a> {
    #[serde(rename = "system")]
    System {
        content: Option<Cow<'a, str>>,
        name: Option<Cow<'a, str>>,
    },
}

#[derive(Debug, Serialize, Deserialize, Default, Deref, DerefMut, From)]
pub struct ChatMessages<'a>(
    #[deref]
    #[deref_mut]
    Vec<ChatMessage<'a>>,
);

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateChatCompletionRequest<'a> {
    #[serde(default)]
    pub messages: ChatMessages<'a>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub stream: Option<bool>,
    pub one_shot: Option<bool>,
    pub n: Option<u32>,
    pub model: Cow<'a, str>,
    pub seed: Option<u32>,

    #[serde(default, with = "either::serde_untagged_optional")]
    pub stop: Option<Either<Cow<'a, str>, Vec<Cow<'a, str>>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionChoice<'a> {
    /// The plaintext of the generated message.
    pub message: ChatMessage<'a>,

    /// If present, the reason that generation terminated at this choice.
    ///
    /// This can be:
    ///
    /// - `length`, indicating that the length cutoff was reached, or
    /// - `stop`, indicating that a stop word was reached.
    pub finish_reason: Option<Cow<'a, str>>,

    /// The index of this choice.
    pub index: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionUsage {
    /// The number of generated tokens.
    pub completion_tokens: u32,

    /// The number of tokens in the prompt.
    pub prompt_tokens: u32,

    /// `completion_tokens` + `prompt_tokens`; the total number of tokens in the dialogue
    /// so far.
    pub total_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletion<'a> {
    pub id: Cow<'a, str>,
    pub choices: Vec<ChatCompletionChoice<'a>>,

    pub created: i64,
    pub model: Cow<'a, str>,
    pub system_fingerprint: Cow<'a, str>,
    pub object: Cow<'a, str>,
    pub usage: ChatCompletionUsage,
}

// enum ChatCompletionResponse<'a, S>
// where
//     S: TryStream<Ok = Event> + Send + 'static,
// {
//     Full(Json<ChatCompletion<'a>>),
//     Stream(Sse<S>),
// }

#[derive(Debug, Deserialize)]
struct TestData {
    pub name: String,
}

#[get("/chat/completions")]
pub async fn chat_completions(streamer: web::Data<Streamer>) -> impl Responder {
    streamer.new_client().await
}

#[post("/chat/broadcast")]
pub async fn broadcast(data: web::Json<TestData>, streamer: web::Data<Streamer>) -> impl Responder {
    streamer.broadcast(&format!("hello {}!", &data.name)).await;
    HttpResponse::Ok().body("Sent.")
}

// pub async fn chat_completions(Json(req) = Json<CreateChatCompletionRequest<'_>>) -> Result<impl IntoResponse, ChatCompletionError> {
//
//
//
// }
//
