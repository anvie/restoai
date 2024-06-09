use std::borrow::Cow;

use openai_dive::v1::resources::{
    chat::{DeltaToolCall, Role, ToolCall},
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
    //#[serde(skip_serializing_if = "Option::is_none")]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ImageUrlDetail {
    Auto,
    High,
    Low,
}

impl From<openai_dive::v1::resources::chat::ImageUrlDetail> for ImageUrlDetail {
    fn from(detail: openai_dive::v1::resources::chat::ImageUrlDetail) -> Self {
        match detail {
            openai_dive::v1::resources::chat::ImageUrlDetail::Auto => ImageUrlDetail::Auto,
            openai_dive::v1::resources::chat::ImageUrlDetail::High => ImageUrlDetail::High,
            openai_dive::v1::resources::chat::ImageUrlDetail::Low => ImageUrlDetail::Low,
        }
    }
}

impl From<ImageUrlDetail> for openai_dive::v1::resources::chat::ImageUrlDetail {
    fn from(detail: ImageUrlDetail) -> openai_dive::v1::resources::chat::ImageUrlDetail {
        match detail {
            ImageUrlDetail::Auto => openai_dive::v1::resources::chat::ImageUrlDetail::Auto,
            ImageUrlDetail::High => openai_dive::v1::resources::chat::ImageUrlDetail::High,
            ImageUrlDetail::Low => openai_dive::v1::resources::chat::ImageUrlDetail::Low,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ImageUrlType {
    /// Either a URL of the image or the base64 encoded image data.
    pub url: String,
    /// Specifies the detail level of the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<ImageUrlDetail>,
}

impl From<openai_dive::v1::resources::chat::ImageUrlType> for ImageUrlType {
    fn from(url: openai_dive::v1::resources::chat::ImageUrlType) -> Self {
        Self {
            url: url.url,
            detail: url.detail.map(|a| a.into()),
        }
    }
}

impl From<ImageUrlType> for openai_dive::v1::resources::chat::ImageUrlType {
    fn from(image_url: ImageUrlType) -> openai_dive::v1::resources::chat::ImageUrlType {
        openai_dive::v1::resources::chat::ImageUrlType {
            url: image_url.url,
            detail: image_url.detail.map(From::from),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ImageUrl {
    /// The type of the content part.
    pub r#type: String,
    /// The text content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// The image URL.
    pub image_url: ImageUrlType,
}

impl From<openai_dive::v1::resources::chat::ImageUrl> for ImageUrl {
    fn from(url: openai_dive::v1::resources::chat::ImageUrl) -> Self {
        Self {
            r#type: url.r#type,
            text: url.text,
            image_url: url.image_url.into(),
        }
    }
}

impl Into<openai_dive::v1::resources::chat::ImageUrl> for ImageUrl {
    fn into(self) -> openai_dive::v1::resources::chat::ImageUrl {
        openai_dive::v1::resources::chat::ImageUrl {
            r#type: self.r#type,
            text: self.text,
            image_url: self.image_url.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ChatMessageContentInner {
    Text { text: String },
    ImageUrl(Vec<ImageUrl>),
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum ChatMessageContent {
    Text(String),
    Multi(Vec<ChatMessageContentInner>),
    ImageUrl(Vec<ImageUrl>),
    None,
}

impl From<openai_dive::v1::resources::chat::ChatMessageContent> for ChatMessageContent {
    fn from(content: openai_dive::v1::resources::chat::ChatMessageContent) -> Self {
        match content {
            openai_dive::v1::resources::chat::ChatMessageContent::Text(text) => {
                ChatMessageContent::Text(text)
            }
            openai_dive::v1::resources::chat::ChatMessageContent::ImageUrl(urls) => {
                ChatMessageContent::ImageUrl(
                    urls.into_iter()
                        .map(|url| ImageUrl {
                            r#type: url.r#type,
                            text: url.text,
                            image_url: ImageUrlType {
                                url: url.image_url.url,
                                detail: url.image_url.detail.map(|a| a.into()),
                            },
                        })
                        .collect(),
                )
            }
            openai_dive::v1::resources::chat::ChatMessageContent::None => ChatMessageContent::None,
        }
    }
}

impl Into<openai_dive::v1::resources::chat::ChatMessageContent> for ChatMessageContent {
    fn into(self) -> openai_dive::v1::resources::chat::ChatMessageContent {
        match self {
            ChatMessageContent::Text(text) => {
                openai_dive::v1::resources::chat::ChatMessageContent::Text(text)
            }
            ChatMessageContent::ImageUrl(urls) => {
                openai_dive::v1::resources::chat::ChatMessageContent::ImageUrl(
                    urls.into_iter()
                        .map(|url| openai_dive::v1::resources::chat::ImageUrl {
                            r#type: url.r#type,
                            text: url.text,
                            image_url: openai_dive::v1::resources::chat::ImageUrlType {
                                url: url.image_url.url,
                                detail: url.image_url.detail.map(|a| a.into()),
                            },
                        })
                        .collect(),
                )
            }
            ChatMessageContent::None => openai_dive::v1::resources::chat::ChatMessageContent::None,
            ChatMessageContent::Multi(content) => match content.first() {
                Some(ChatMessageContentInner::Text { text }) => {
                    openai_dive::v1::resources::chat::ChatMessageContent::Text(text.to_string())
                }
                Some(ChatMessageContentInner::ImageUrl(urls)) => {
                    openai_dive::v1::resources::chat::ChatMessageContent::ImageUrl(
                        urls.iter().map(|a| a.clone().into()).collect(),
                    )
                }
                Some(ChatMessageContentInner::None) => {
                    openai_dive::v1::resources::chat::ChatMessageContent::None
                }
                None => openai_dive::v1::resources::chat::ChatMessageContent::None,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ChatMessage {
    /// The role of the author of this message.
    pub role: Role,
    /// The content of the message.
    pub content: ChatMessageContent,
    /// The tool calls generated by the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    /// An optional name for the participant. Provides the model information to differentiate between participants of the same role.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// When responding to a tool call; provide the id of the tool call
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl From<openai_dive::v1::resources::chat::ChatMessage> for ChatMessage {
    fn from(message: openai_dive::v1::resources::chat::ChatMessage) -> Self {
        Self {
            role: message.role,
            content: message.content.into(),
            tool_calls: message.tool_calls,
            name: message.name,
            tool_call_id: message.tool_call_id,
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
            message: choice.message.into(),
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
    pub logprobs: Option<FinishReason>,
    /// The reason the model stopped generating tokens.
    //#[serde(skip_serializing_if = "Option::is_none")]
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
            logprobs: None,
            finish_reason: choice.finish_reason,
        }
    }
}

use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum StopToken {
    String(String),
    Array(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ChatCompletionParameters {
    /// A list of messages comprising the conversation so far.
    pub messages: Vec<ChatMessage>,
    /// ID of the model to use.
    pub model: String,
    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on their existing frequency in the text so far,
    /// decreasing the model's likelihood to repeat the same line verbatim.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    /// Modify the likelihood of specified tokens appearing in the completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, i32>>,
    /// Whether to return log probabilities of the output tokens or not.
    /// If true, returns the log probabilities of each output token returned in the 'content' of 'message'.
    /// This option is currently not available on the 'gpt-4-vision-preview' model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,
    /// An integer between 0 and 5 specifying the number of most likely tokens to return at each token position,
    /// each with an associated log probability. 'logprobs' must be set to 'true' if this parameter is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u32>,
    /// The maximum number of tokens to generate in the chat completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// How many chat completion choices to generate for each input message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far,
    /// increasing the model's likelihood to talk about new topics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    /// An object specifying the format that the model must output.
    /// Setting to { "type": "json_object" } enables JSON mode, which guarantees the message the model generates is valid JSON.
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub response_format: Option<ChatCompletionResponseFormat>,
    /// This feature is in Beta. If specified, our system will make a best effort to sample deterministically,
    /// such that repeated requests with the same seed and parameters should return the same result.
    /// Determinism is not guaranteed, and you should refer to the system_fingerprint response parameter to monitor changes in the backend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u32>,
    /// Up to 4 sequences where the API will stop generating further tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<StopToken>,
    /// If set, partial messages will be sent, like in ChatGPT. Tokens will be sent as data-only server-sent events
    /// as they become available, with the stream terminated by a data: [DONE] message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the output more random,
    /// while lower values like 0.2 will make it more focused and deterministic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// An alternative to sampling with temperature, called nucleus sampling, where the model considers the results of the tokens with top_p probability mass.
    /// So 0.1 means only the tokens comprising the top 10% probability mass are considered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// A list of tools the model may call. Currently, only functions are supported as a tool.
    /// Use this to provide a list of functions the model may generate JSON inputs for.
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub tools: Option<Vec<ChatCompletionTool>>,

    /// Controls which (if any) function is called by the model. none means the model will not call a function and instead generates a message.
    /// 'auto' means the model can pick between generating a message or calling a function.
    /// Specifying a particular function via {"type: "function", "function": {"name": "my_function"}} forces the model to call that function.
    /// 'none' is the default when no functions are present. 'auto' is the default if functions are present.
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub tool_choice: Option<ChatCompletionToolChoice>,

    /// A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

use tokio::sync::mpsc;

// pub struct ClientCloser(pub mpsc::Sender<std::net::SocketAddr>);
