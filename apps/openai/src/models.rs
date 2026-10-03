use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;

/// The same request works with both `Client::create` and `Client::stream`.
/// The client sets the wire-level `stream` flag itself.
#[derive(Debug, Clone, Serialize)]
pub struct ResponseRequest {
    pub model: String,
    pub input: Input,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<Tool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
}

impl ResponseRequest {
    pub fn new(model: impl Into<String>, input: impl Into<Input>) -> Self {
        Self {
            model: model.into(),
            input: input.into(),
            instructions: None,
            tools: Vec::new(),
            tool_choice: None,
            parallel_tool_calls: None,
            previous_response_id: None,
            max_output_tokens: None,
            store: None,
        }
    }

    /// Retains model, instructions, and tools. Unstored responses replay the
    /// conversation, including reasoning items; stored responses use their ID.
    pub fn continue_from(mut self, response: &Response, input: Vec<InputItem>) -> Self {
        if response.store == Some(false) || self.store == Some(false) {
            let mut history = match self.input {
                Input::Text(text) => vec![InputItem::message(Role::User, text)],
                Input::Items(items) => items,
            };
            history.extend(response.raw_output.iter().cloned().map(InputItem::Output));
            history.extend(input);
            self.input = Input::Items(history);
            self.previous_response_id = None;
            self.store = Some(false);
        } else {
            self.previous_response_id = Some(response.id.clone());
            self.input = Input::Items(input);
        }
        self
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Input {
    Text(String),
    Items(Vec<InputItem>),
}

impl From<String> for Input {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for Input {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

impl From<Vec<InputItem>> for Input {
    fn from(value: Vec<InputItem>) -> Self {
        Self::Items(value)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputItem {
    Message {
        role: Role,
        content: String,
    },
    FunctionCallOutput {
        call_id: String,
        output: String,
    },
    /// An exact output item replayed in a stateless conversation.
    #[serde(untagged)]
    Output(Value),
}

impl InputItem {
    pub fn message(role: Role, content: impl Into<String>) -> Self {
        Self::Message {
            role,
            content: content.into(),
        }
    }

    pub fn tool_output(call_id: impl Into<String>, output: impl Into<String>) -> Self {
        Self::FunctionCallOutput {
            call_id: call_id.into(),
            output: output.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
    System,
    Developer,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Tool {
    Function {
        name: String,
        description: String,
        parameters: Value,
        strict: bool,
    },
}

impl Tool {
    /// Strict schemas must require all properties and disallow additional ones.
    pub fn function(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: Value,
    ) -> Self {
        Self::Function {
            name: name.into(),
            description: description.into(),
            parameters,
            strict: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ToolChoice {
    Auto,
    None,
    Required,
    Function(String),
}

impl Serialize for ToolChoice {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Auto => serializer.serialize_str("auto"),
            Self::None => serializer.serialize_str("none"),
            Self::Required => serializer.serialize_str("required"),
            Self::Function(name) => {
                serde_json::json!({"type": "function", "name": name}).serialize(serializer)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub id: String,
    pub status: String,
    pub output: Vec<OutputItem>,
    pub usage: Option<Value>,
    pub error: Option<Value>,
    pub incomplete_details: Option<Value>,
    pub store: Option<bool>,
    // Preserve fields not represented in OutputItem (e.g. encrypted reasoning)
    // for lossless stateless replay with ChatGPT authentication.
    raw_output: Vec<Value>,
}

impl<'de> Deserialize<'de> for Response {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct WireResponse {
            id: String,
            status: String,
            #[serde(default)]
            output: Vec<Value>,
            usage: Option<Value>,
            error: Option<Value>,
            incomplete_details: Option<Value>,
            store: Option<bool>,
        }
        let wire = WireResponse::deserialize(deserializer)?;
        let output = wire
            .output
            .iter()
            .cloned()
            .map(serde_json::from_value)
            .collect::<Result<_, _>>()
            .map_err(serde::de::Error::custom)?;
        Ok(Self {
            id: wire.id,
            status: wire.status,
            output,
            usage: wire.usage,
            error: wire.error,
            incomplete_details: wire.incomplete_details,
            store: wire.store,
            raw_output: wire.output,
        })
    }
}

impl Response {
    /// Some streaming backends omit output from the terminal response. Restore
    /// completed items in output-index order, retaining their exact replay data.
    pub(crate) fn restore_stream_output(&mut self, items: Vec<Value>) -> crate::Result<()> {
        if self.raw_output.is_empty() {
            self.output = items
                .iter()
                .cloned()
                .map(serde_json::from_value)
                .collect::<Result<_, _>>()?;
            self.raw_output = items;
        }
        Ok(())
    }

    pub fn output_text(&self) -> String {
        self.output
            .iter()
            .filter_map(|item| match item {
                OutputItem::Message { content, .. } => Some(content),
                _ => None,
            })
            .flatten()
            .filter_map(|content| match content {
                OutputContent::OutputText { text } => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }

    pub fn tool_calls(&self) -> impl Iterator<Item = &FunctionCall> {
        self.output.iter().filter_map(|item| match item {
            OutputItem::FunctionCall(call) => Some(call),
            _ => None,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputItem {
    Message {
        id: String,
        content: Vec<OutputContent>,
    },
    FunctionCall(FunctionCall),
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputContent {
    OutputText {
        text: String,
    },
    Refusal {
        refusal: String,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FunctionCall {
    pub id: String,
    pub call_id: String,
    pub name: String,
    pub arguments: String,
}

impl FunctionCall {
    /// Decode only completed arguments, never a partial arguments delta.
    pub fn parse_arguments<T: DeserializeOwned>(&self) -> crate::Result<T> {
        Ok(serde_json::from_str(&self.arguments)?)
    }
}

/// Unknown events are ignored for forward compatibility. Complete function
/// calls arrive in `OutputItemDone` and in `Completed.response.output`.
/// Delta strings are fragments, not independently valid JSON.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    #[serde(rename = "response.created")]
    Created { response: Response },
    #[serde(rename = "response.output_text.delta")]
    TextDelta {
        item_id: String,
        output_index: usize,
        content_index: usize,
        delta: String,
    },
    #[serde(rename = "response.output_item.added")]
    OutputItemAdded {
        output_index: usize,
        item: OutputItem,
    },
    #[serde(rename = "response.function_call_arguments.delta")]
    FunctionArgumentsDelta {
        item_id: String,
        output_index: usize,
        delta: String,
    },
    #[serde(rename = "response.function_call_arguments.done")]
    FunctionArgumentsDone {
        item_id: String,
        output_index: usize,
        arguments: String,
    },
    #[serde(rename = "response.output_item.done")]
    OutputItemDone {
        output_index: usize,
        item: OutputItem,
    },
    #[serde(rename = "response.completed")]
    Completed { response: Response },
    #[serde(rename = "response.failed")]
    Failed { response: Response },
    #[serde(rename = "response.incomplete")]
    Incomplete { response: Response },
    #[serde(rename = "error")]
    Error {
        code: Option<String>,
        message: String,
    },
    #[serde(other)]
    Other,
}
