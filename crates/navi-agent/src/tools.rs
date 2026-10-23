use crate::model::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Permission {
    ReadOnly,
    LaunchApplication,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Permissions {
    pub read_only: bool,
    pub launch_application: bool,
    pub confirm_launch: bool,
}

impl Default for Permissions {
    fn default() -> Self {
        Self {
            read_only: true,
            launch_application: true,
            confirm_launch: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolKind {
    Help,
    CurrentTime,
    SystemStatus,
    OpenCalculator,
}

#[derive(Clone, Copy, Debug)]
pub struct ToolDefinition {
    pub kind: ToolKind,
    pub id: &'static str,
    pub description: &'static str,
    pub permission: Permission,
    pub confirmation_required: bool,
}

impl ToolKind {
    pub fn definition(self) -> ToolDefinition {
        let (id, description) = match self {
            Self::Help => ("help", "Mostrar las solicitudes conocidas"),
            Self::CurrentTime => ("current_time", "Consultar la hora local"),
            Self::SystemStatus => ("system_status", "Consultar sistema y procesadores"),
            Self::OpenCalculator => (
                "open_calculator",
                "Solicitar el inicio de la calculadora de Windows",
            ),
        };
        let launch = self == Self::OpenCalculator;
        ToolDefinition {
            kind: self,
            id,
            description,
            permission: if launch {
                Permission::LaunchApplication
            } else {
                Permission::ReadOnly
            },
            confirmation_required: false,
        }
    }
    pub fn action(self) -> Action {
        Action {
            tool: self.definition().id.into(),
            parameters: if self == Self::OpenCalculator {
                Parameters::Application(Application::Calculator)
            } else {
                Parameters::None
            },
        }
    }
}

/// Closed capabilities supplied by the platform. There is no arbitrary command.
pub trait ToolHost {
    fn current_time(&mut self) -> Result<String, ToolError>;
    fn system_status(&mut self) -> Result<Vec<(String, String)>, ToolError>;
    fn open_calculator(&mut self) -> Result<(), ToolError>;
}

pub struct ToolRegistry {
    tools: Vec<ToolKind>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self {
            tools: vec![
                ToolKind::Help,
                ToolKind::CurrentTime,
                ToolKind::SystemStatus,
                ToolKind::OpenCalculator,
            ],
        }
    }
}

impl ToolRegistry {
    pub fn empty() -> Self {
        Self { tools: Vec::new() }
    }
    pub fn register(&mut self, kind: ToolKind) -> bool {
        if self.tools.contains(&kind) {
            return false;
        }
        self.tools.push(kind);
        true
    }
    pub fn definitions(&self) -> impl Iterator<Item = ToolDefinition> + '_ {
        self.tools.iter().map(|k| k.definition())
    }
    pub(crate) fn validate(
        &self,
        action: &Action,
        permissions: Permissions,
    ) -> Result<ToolDefinition, ToolError> {
        let definition = self
            .definitions()
            .find(|d| d.id == action.tool)
            .ok_or_else(|| {
                ToolError::new(ErrorCode::UnknownTool, "La herramienta no está registrada.")
            })?;
        if action.parameters != definition.kind.action().parameters {
            return Err(ToolError::new(
                ErrorCode::InvalidParameters,
                "Los parámetros no son válidos para esta herramienta.",
            ));
        }
        let allowed = match definition.permission {
            Permission::ReadOnly => permissions.read_only,
            Permission::LaunchApplication => permissions.launch_application,
        };
        if !allowed {
            return Err(ToolError::new(
                ErrorCode::PermissionDenied,
                "La herramienta no tiene permiso para ejecutarse.",
            ));
        }
        Ok(definition)
    }
    pub(crate) fn run(
        &self,
        definition: ToolDefinition,
        host: &mut impl ToolHost,
    ) -> Result<(String, Vec<(String, String)>), ToolError> {
        match definition.kind {
            ToolKind::Help => Ok((
                format!(
                    "Solicitudes disponibles: {}.",
                    self.definitions()
                        .map(|d| match d.kind {
                            ToolKind::Help => "ayuda",
                            ToolKind::CurrentTime => "qué hora es",
                            ToolKind::SystemStatus => "estado del sistema",
                            ToolKind::OpenCalculator => "abrí la calculadora",
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                Vec::new(),
            )),
            ToolKind::CurrentTime => {
                let time = host.current_time()?;
                Ok((
                    format!("Son las {time}."),
                    vec![("hora_local".into(), time)],
                ))
            }
            ToolKind::SystemStatus => {
                let data = host.system_status()?;
                let text = data
                    .iter()
                    .take(16)
                    .map(|(k, v)| format!("{k}: {v}"))
                    .collect::<Vec<_>>()
                    .join(". ");
                Ok((text, data))
            }
            ToolKind::OpenCalculator => {
                host.open_calculator()?;
                Ok((
                    "Solicité el inicio de la calculadora.".into(),
                    vec![("aplicacion".into(), "calculadora".into())],
                ))
            }
        }
    }
}

