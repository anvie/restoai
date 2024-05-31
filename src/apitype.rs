use std::borrow::Cow;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Model<'a> {
    /// The model identifier, which can be referenced in the API endpoints.
    pub id: Cow<'a, str>,
    /// The Unix timestamp (in seconds) when the model was created.
    pub created: u32,
    /// The object type, which is always "model".
    pub object: Cow<'a, str>,
    /// The organization that owns the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owned_by: Option<Cow<'a, str>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ListModelResponse<'a> {
    /// The object type, which is always "list".
    pub object: Cow<'a, str>,
    /// A list of model objects.
    pub data: Vec<Model<'a>>,
}
