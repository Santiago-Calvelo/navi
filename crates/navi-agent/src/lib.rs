//! Pure, bounded textual core. No Windows, network, rendering or shell execution.
pub mod context;
pub mod model;
pub mod router;
pub mod tools;

use context::Context;
use model::*;
use router::{Intent, Router};
use std::time::Instant;
use tools::{Permissions, ToolHost, ToolRegistry};

pub const CONFIRMATION_TTL_MS: u64 = 60_000;

#[derive(Default)]
pub struct Agent {
    pub registry: ToolRegistry,
    pub router: Router,
    pub permissions: Permissions,
    pub context: Context,
    pending: Option<Confirmation>,
    sequence: u64,
}

impl Agent {
    /// now_ms is a monotonic session clock, supplied by the adapter for testability.
    pub fn handle(&mut self, text: &str, host: &mut impl ToolHost, now_ms: u64) -> Response {
        self.sequence = self.sequence.saturating_add(1);
        let id = self.sequence;
        let started = Instant::now();
        let message = match Message::parse(text) {
            Ok(message) => message,
            Err(error) => {
                self.pending = None;
                self.context.application = None;
                let mut r = self.failure(id, error);
                r.timings.routing = started.elapsed();
                return r;
            }
        };
        let intent = self.router.route(&message, self.context.application);
        let routing = started.elapsed();
        let mut response = match intent {
            Intent::Confirm(request) => self.confirm(id, request, host, now_ms),
            Intent::Cancel => {
                self.context.application = None;
                if self.pending.take().is_some() {
                    Response::new(id, Status::Cancelled, "Acción cancelada.")
                } else {
                    Response::new(id, Status::Clarification, "No hay una acción pendiente.")
                }
            }
            Intent::Tool(action) => {
                self.pending = None;
                self.context.application = match action.parameters {
                    Parameters::Application(app) => Some(app),
                    Parameters::None => None,
                };
                self.execute(action, host, now_ms, id)
            }
            Intent::NeedsProvider => {
                self.pending = None;
                self.context.application = None;
                Response::new(id,Status::NeedsProvider,"Esta solicitud necesita un proveedor de inteligencia. Todavía no hay uno conectado; no ejecuté acciones.")
            }
            Intent::Clarify(prompt) => {
                let confirmation_question =
                    matches!(message.normalized.as_str(), "si" | "confirmar")
                        || message.normalized.starts_with("confirmar ");
                if confirmation_question
                    && self
                        .pending
                        .as_ref()
                        .is_some_and(|p| now_ms >= p.expires_at_ms)
                {
                    self.pending = None;
                    self.failure(
                        id,
                        ToolError::new(
                            ErrorCode::InvalidConfirmation,
                            "La confirmación venció. Pedí la acción nuevamente.",
                        ),
                    )
                } else if let Some(pending) =
                    self.pending.as_ref().filter(|_| confirmation_question)
                {
                    let mut r = Response::new(
                        id,
                        Status::NeedsConfirmation,
                        format!(
                            "Para autorizar la calculadora, escribí confirmar {} o cancelar.",
                            pending.request_id
                        ),
                    );
                    r.tool = Some(pending.action.tool.clone());
                    r.confirmation = Some(pending.clone());
                    r
                } else {
                    self.pending = None;
                    self.context.application = None;
                    Response::new(id, Status::Clarification, prompt)
                }
            }
        };
        response.timings.routing = routing;
        self.context.record(&message.original, &response.text);
        response
    }

    /// Public action entry point still enforces registry, arguments and permissions.
    pub fn request_action(
        &mut self,
        action: Action,
        host: &mut impl ToolHost,
        now_ms: u64,
    ) -> Response {
        self.sequence = self.sequence.saturating_add(1);
        self.pending = None;
        self.execute(action, host, now_ms, self.sequence)
    }

    fn execute(
        &mut self,
        action: Action,
        host: &mut impl ToolHost,
        now_ms: u64,
        id: u64,
    ) -> Response {
        let definition = match self.registry.validate(&action, self.permissions) {
            Ok(d) => d,
            Err(e) => return self.failure(id, e),
        };
        if definition.confirmation_required
            || (definition.permission == tools::Permission::LaunchApplication
                && self.permissions.confirm_launch)
        {
            let confirmation = Confirmation {
                request_id: id,
                action: action.clone(),
                expires_at_ms: now_ms.saturating_add(CONFIRMATION_TTL_MS),
            };
            self.pending = Some(confirmation.clone());
            let mut r=Response::new(id,Status::NeedsConfirmation,format!("¿Abrir la calculadora? Escribí confirmar {id} o cancelar. Válido por 60 segundos."));
            r.tool = Some(action.tool);
            r.confirmation = Some(confirmation);
            return r;
        }
        self.run(id, definition, host)
    }

    fn confirm(
        &mut self,
        id: u64,
        request: u64,
        host: &mut impl ToolHost,
        now_ms: u64,
    ) -> Response {
        let Some(pending) = self.pending.take() else {
            return self.failure(
                id,
                ToolError::new(
                    ErrorCode::InvalidConfirmation,
                    "No hay una confirmación pendiente.",
                ),
            );
        };
        if pending.request_id != request || now_ms >= pending.expires_at_ms {
            return self.failure(
                id,
                ToolError::new(
                    ErrorCode::InvalidConfirmation,
                    "La confirmación no coincide o venció. Pedí la acción nuevamente.",
                ),
            );
        }
        let definition = match self.registry.validate(&pending.action, self.permissions) {
            Ok(d) => d,
            Err(e) => return self.failure(id, e),
        };
        self.run(id, definition, host)
    }

    fn run(
        &self,
        id: u64,
        definition: tools::ToolDefinition,
        host: &mut impl ToolHost,
    ) -> Response {
        let started = Instant::now();
        let result = self.registry.run(definition, host);
        let execution = started.elapsed();
        let mut r = match result {
            Ok((text, data)) => {
                let mut r = Response::new(
                    id,
                    Status::Success,
                    text.chars().take(MAX_MESSAGE_CHARS).collect::<String>(),
                );
                r.data = data
                    .into_iter()
                    .take(16)
                    .map(|(k, v)| (k.chars().take(64).collect(), v.chars().take(128).collect()))
                    .collect();
                r
            }
            Err(error) => {
                let mut r = self.failure(id, error);
                r.status = Status::Error;
                r
            }
        };
        r.tool = Some(definition.id.into());
        r.timings.execution = execution;
        r
    }

    fn failure(&self, id: u64, error: ToolError) -> Response {
        let mut r = Response::new(id, Status::Rejected, error.message.clone());
        r.error = Some(error);
        r
    }
}
