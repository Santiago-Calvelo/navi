use std::time::Duration;

pub const MAX_MESSAGE_CHARS: usize = 512;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    pub original: String,
    pub normalized: String,
}

impl Message {
    pub fn parse(text: &str) -> Result<Self, ToolError> {
        if text.chars().count() > MAX_MESSAGE_CHARS {
            return Err(ToolError::new(
                ErrorCode::InvalidInput,
                "El mensaje supera 512 caracteres.",
            ));
        }
        if text.chars().any(|c| c.is_control() && !c.is_whitespace()) {
            return Err(ToolError::new(
                ErrorCode::InvalidInput,
                "El mensaje contiene controles inválidos.",
            ));
        }
        let original = text.trim().to_owned();
        let normalized: String = original
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
            .chars()
            .filter_map(|c| match c {
                'á' => Some('a'),
                'é' => Some('e'),
                'í' => Some('i'),
                'ó' => Some('o'),
                'ú' | 'ü' => Some('u'),
                '\u{301}' => None,
                c => Some(c),
            })
            .collect();
        let normalized = normalized
            .trim_matches(['¿', '¡', '?', '!', '.'])
            .trim()
            .to_owned();
        if normalized.is_empty() {
            return Err(ToolError::new(
                ErrorCode::InvalidInput,
                "Escribí una solicitud.",
            ));
        }
        Ok(Self {
            original,
            normalized,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Application {
    Calculator,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Parameters {
    None,
    Application(Application),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Action {
    pub tool: String,
    pub parameters: Parameters,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Success,
    NeedsConfirmation,
    NeedsProvider,
    Clarification,
    Rejected,
    Cancelled,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    InvalidInput,
    UnknownTool,
    InvalidParameters,
    PermissionDenied,
    InvalidConfirmation,
    ToolFailed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolError {
    pub code: ErrorCode,
    pub message: String,
}

impl ToolError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into().chars().take(MAX_MESSAGE_CHARS).collect(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Timings {
    pub routing: Duration,
    pub execution: Duration,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Confirmation {
    pub request_id: u64,
    pub action: Action,
    pub expires_at_ms: u64,
}

#[derive(Clone, Debug)]
pub struct Response {
    pub request_id: u64,
    pub status: Status,
    pub text: String,
    pub tool: Option<String>,
    pub data: Vec<(String, String)>,
    pub confirmation: Option<Confirmation>,
    pub error: Option<ToolError>,
    pub timings: Timings,
}

impl Response {
    pub(crate) fn new(request_id: u64, status: Status, text: impl Into<String>) -> Self {
        Self {
            request_id,
            status,
            text: text.into(),
            tool: None,
            data: Vec::new(),
            confirmation: None,
            error: None,
            timings: Timings::default(),
        }
    }
}
