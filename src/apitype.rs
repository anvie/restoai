use std::borrow::Cow;

use openai_dive::v1::resources::{
    chat::{ChatCompletionParameters, ChatMessage, DeltaToolCall, Role},
    shared::FinishReason,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Model {
    /// The model identifier, which can be referenced in the API endpoints.
    pub id: String,
    /// The Unix timestamp (in seconds) when the model was created.
    pub created: u32,
    /// The object type, which is always "model".
    pub object: String,
    /// The organization that owns the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owned_by: Option<String>,
}

impl From<openai_dive::v1::resources::model::Model> for Model {
    fn from(model: openai_dive::v1::resources::model::Model) -> Self {
        Self {
            id: model.id,
            created: model.created,
            object: model.object,
            owned_by: if model.owned_by.is_empty() {
                None
            } else {
                Some(model.owned_by.to_owned())
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ListModelResponse {
    /// The object type, which is always "list".
    pub object: String,
    /// A list of model objects.
    pub data: Vec<Model>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ChatCompletionUsage {
    /// The number of generated tokens.
    pub completion_tokens: Option<u32>,

    /// The number of tokens in the prompt.
    pub prompt_tokens: u32,

    /// `completion_tokens` + `prompt_tokens`; the total number of tokens in the dialogue
    /// so far.
    pub total_tokens: u32,
}

impl From<openai_dive::v1::resources::shared::Usage> for ChatCompletionUsage {
    fn from(usage: openai_dive::v1::resources::shared::Usage) -> Self {
        Self {
            completion_tokens: usage.completion_tokens,
            prompt_tokens: usage.prompt_tokens,
            total_tokens: usage.total_tokens,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ChatCompletion {
    pub id: String,
    pub choices: Vec<ChatCompletionChoice>,

    pub created: u32,
    //pub model: String,
    //pub system_fingerprint: String,
    pub object: String,
    pub usage: Option<ChatCompletionUsage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatCompletionResponse {
    /// A unique identifier for the chat completion.
    pub id: String,
    /// A list of chat completion choices. Can be more than one if n is greater than 1.
    pub choices: Vec<ChatCompletionChoice>,
    /// The Unix timestamp (in seconds) of when the chat completion was created.
    pub created: u32,
    /// The model used for the chat completion.
    pub model: String,
    /// This fingerprint represents the backend configuration that the model runs with.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
    /// The object type, which is always chat.completion.
    pub object: String,
    /// Usage statistics for the completion request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<ChatCompletionUsage>,
}

impl From<openai_dive::v1::resources::chat::ChatCompletionResponse> for ChatCompletionResponse {
    fn from(response: openai_dive::v1::resources::chat::ChatCompletionResponse) -> Self {
        Self {
            id: response.id,
            choices: response
                .choices
                .into_iter()
                .map(ChatCompletionChoice::from)
                .collect(),
            created: response.created,
            model: response.model,
            system_fingerprint: response.system_fingerprint,
            object: response.object,
            usage: response.usage.map(|a| a.into()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ChatCompletionChunkResponse {
    /// A unique identifier for the chat completion. Each chunk has the same ID.
    pub id: String,
    /// A list of chat completion choices. Can be more than one if n is greater than 1.
    pub choices: Vec<ChatCompletionChunkChoice>,
    /// The Unix timestamp (in seconds) of when the chat completion was created. Each chunk has the same timestamp.
    pub created: u32,
    /// The model to generate the completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// This fingerprint represents the backend configuration that the model runs with.
    /// Can be used in conjunction with the seed request parameter to understand when backend changes have been made that might impact determinism.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
    /// The object type, which is always chat.completion.chunk.
    pub object: String,
}

pub type ModelList = ListModelResponse;

impl From<openai_dive::v1::resources::model::ListModelResponse> for ModelList {
    fn from(list: openai_dive::v1::resources::model::ListModelResponse) -> Self {
        Self {
            object: list.object,
            data: list.data.into_iter().map(Model::from).collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatCompletionChoice {
    /// The plaintext of the generated message.
    pub message: ChatMessage,

    /// If present, the reason that generation terminated at this choice.
    ///
    /// This can be:
    ///
    /// - `length`, indicating that the length cutoff was reached, or
    /// - `stop`, indicating that a stop word was reached.
    pub finish_reason: Option<String>,

    /// The index of this choice.
    pub index: u32,
}

impl From<openai_dive::v1::resources::chat::ChatCompletionChoice> for ChatCompletionChoice {
    fn from(choice: openai_dive::v1::resources::chat::ChatCompletionChoice) -> Self {
        Self {
            message: choice.message,
            finish_reason: serde_json::to_string(&choice.finish_reason).ok(),
            index: choice.index,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DeltaChatMessage {
    /// The role of the author of this message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
    /// The contents of the chunk message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// The tool calls generated by the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<DeltaToolCall>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ChatCompletionChunkChoice {
    /// The index of the choice in the list of choices.
    pub index: Option<u32>,
    /// A chat completion delta generated by streamed model responses.
    pub delta: DeltaChatMessage,
    /// The reason the model stopped generating tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,
}

impl From<openai_dive::v1::resources::chat::ChatCompletionChunkChoice>
    for ChatCompletionChunkChoice
{
    fn from(choice: openai_dive::v1::resources::chat::ChatCompletionChunkChoice) -> Self {
        Self {
            index: choice.index,
            delta: DeltaChatMessage {
                role: choice.delta.role,
                content: choice.delta.content.map(|c| c.to_string()),
                tool_calls: choice.delta.tool_calls,
            },
            finish_reason: choice.finish_reason,
        }
    }
}
