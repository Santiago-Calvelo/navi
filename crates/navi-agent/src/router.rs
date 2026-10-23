use crate::{model::*, tools::ToolKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Intent {
    Tool(Action),
    Confirm(u64),
    Cancel,
    NeedsProvider,
    Clarify(&'static str),
}

pub struct Router {
    rules: Vec<(&'static str, ToolKind)>,
}

impl Default for Router {
    fn default() -> Self {
        Self {
            rules: vec![
                ("ayuda", ToolKind::Help),
                ("que podes hacer", ToolKind::Help),
                ("que hora es", ToolKind::CurrentTime),
                ("hora", ToolKind::CurrentTime),
                ("hora actual", ToolKind::CurrentTime),
                ("estado del sistema", ToolKind::SystemStatus),
                ("estado de la computadora", ToolKind::SystemStatus),
                ("informacion del sistema", ToolKind::SystemStatus),
            ],
        }
    }
}

impl Router {
    pub fn register(&mut self, normalized_phrase: &'static str, tool: ToolKind) -> bool {
        if self
            .rules
            .iter()
            .any(|(phrase, _)| *phrase == normalized_phrase)
        {
            return false;
        }
        self.rules.push((normalized_phrase, tool));
        true
    }
    pub fn route(&self, message: &Message, application: Option<Application>) -> Intent {
        let text = message.normalized.as_str();
        if let Some((_, tool)) = self.rules.iter().find(|(phrase, _)| *phrase == text) {
            return Intent::Tool(tool.action());
        }
        if text == "cancelar" {
            return Intent::Cancel;
        }
        if text == "confirmar" || text == "si" {
            return Intent::Clarify(
                "Indicá el ID: confirmar <número>. Ninguna acción fue ejecutada.",
            );
        }
        if let Some(id) = text.strip_prefix("confirmar ") {
            return match id.parse::<u64>() {
                Ok(id) if id > 0 => Intent::Confirm(id),
                _ => Intent::Clarify("La confirmación necesita un único ID numérico válido."),
            };
        }
        if matches!(text, "abrila" | "abrilo" | "abrela" | "abrirla") {
            return if application == Some(Application::Calculator) {
                Intent::Tool(ToolKind::OpenCalculator.action())
            } else {
                Intent::Clarify("¿Qué aplicación querés abrir? No hay un referente inequívoco.")
            };
        }
        if matches!(text, "abrir" | "abri" | "abre") {
            return Intent::Clarify(
                "¿Qué aplicación querés abrir? La herramienta disponible es calculadora.",
            );
        }
        for prefix in ["abrir ", "abri ", "abre "] {
            if let Some(target) = text.strip_prefix(prefix) {
                return if matches!(target, "calculadora" | "la calculadora") {
                    Intent::Tool(ToolKind::OpenCalculator.action())
                } else {
                    Intent::NeedsProvider
                };
            }
        }
        Intent::NeedsProvider
    }
}
