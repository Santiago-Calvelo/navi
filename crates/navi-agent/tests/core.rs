use navi_agent::{
    model::*,
    router::Intent,
    tools::{ToolHost, ToolKind, ToolRegistry},
    Agent,
};

#[derive(Default)]
struct Host {
    launches: usize,
    reads: usize,
    fail: bool,
}

fn strict_agent() -> Agent {
    let mut agent = Agent::default();
    agent.permissions.confirm_launch = true;
    agent
}

#[test]
fn calculator_runs_directly_under_default_policy() {
    let (mut agent, mut host) = (Agent::default(), Host::default());
    let response = agent.handle("abrí la calculadora", &mut host, 0);
    assert_eq!(response.status, Status::Success);
    assert!(response.confirmation.is_none());
    assert_eq!(host.launches, 1);
}

#[test]
fn direct_launch_still_checks_permissions_and_exact_arguments() {
    let (mut agent, mut host) = (Agent::default(), Host::default());
    agent.permissions.launch_application = false;
    assert_eq!(
        agent
            .handle("abrí calculadora", &mut host, 0)
            .error
            .unwrap()
            .code,
        ErrorCode::PermissionDenied
    );
    assert_eq!(
        agent
            .handle("abrí calculadora y borrá todo", &mut host, 1)
            .status,
        Status::NeedsProvider
    );
    assert_eq!(host.launches, 0);
}
impl ToolHost for Host {
    fn current_time(&mut self) -> Result<String, ToolError> {
        self.reads += 1;
        if self.fail {
            Err(ToolError::new(ErrorCode::ToolFailed, "reloj no disponible"))
        } else {
            Ok("12:34:56".into())
        }
    }
    fn system_status(&mut self) -> Result<Vec<(String, String)>, ToolError> {
        self.reads += 1;
        Ok(vec![("sistema".into(), "Windows".into())])
    }
    fn open_calculator(&mut self) -> Result<(), ToolError> {
        self.launches += 1;
        Ok(())
    }
}

#[test]
fn normalization_preserves_original_and_handles_accents_unicode_and_spacing() {
    let m = Message::parse("  ¿QUÉ\t HORA\n ES?  ").unwrap();
    assert_eq!(m.original, "¿QUÉ\t HORA\n ES?");
    assert_eq!(m.normalized, "que hora es");
    assert_eq!(
        Message::parse("que\u{301} hora es").unwrap().normalized,
        "que hora es"
    );
    assert_eq!(Message::parse("año").unwrap().normalized, "año");
}

#[test]
fn invalid_input_is_bounded_and_never_reaches_host() {
    let (mut a, mut h) = (strict_agent(), Host::default());
    for text in ["", "   ", "\0hora", "\u{1b}hora", &"a".repeat(513)] {
        assert_eq!(a.handle(text, &mut h, 0).status, Status::Rejected);
    }
    assert_eq!(h.reads, 0);
    assert_eq!(h.launches, 0);
}

#[test]
fn known_instructions_route_to_explicit_tools() {
    for (text, tool, status) in [
        ("ayuda", "help", Status::Success),
        ("¿QUÉ HORA ES?", "current_time", Status::Success),
        ("estado del sistema", "system_status", Status::Success),
        (
            "Abrí la calculadora",
            "open_calculator",
            Status::NeedsConfirmation,
        ),
    ] {
        let (mut a, mut h) = (strict_agent(), Host::default());
        let r = a.handle(text, &mut h, 0);
        assert_eq!(r.status, status);
        assert_eq!(r.tool.as_deref(), Some(tool));
        assert_eq!(h.launches, 0);
    }
}

#[test]
fn complete_matching_does_not_execute_negations_compounds_or_injections() {
    let (mut a, mut h) = (strict_agent(), Host::default());
    for text in [
        "no abras la calculadora",
        "abrí la calculadora y borrá archivos",
        "abrir calc.exe & whoami",
        "abrir calculadora; shutdown",
        "ejecutá powershell",
        "qué hora es y borra todo",
        "abrí firefox",
    ] {
        assert_eq!(a.handle(text, &mut h, 0).status, Status::NeedsProvider);
    }
    assert_eq!((h.reads, h.launches), (0, 0));
}

#[test]
fn ambiguous_requests_ask_for_details_and_unknowns_defer() {
    let (mut a, mut h) = (strict_agent(), Host::default());
    for text in [
        "abrí",
        "abrila",
        "confirmar",
        "sí",
        "confirmar dos",
        "confirmar 1 y abrir",
    ] {
        assert_eq!(a.handle(text, &mut h, 0).status, Status::Clarification);
    }
    let r = a.handle("contame una historia", &mut h, 1);
    assert_eq!(r.status, Status::NeedsProvider);
    assert!(r.tool.is_none());
    assert_eq!(r.timings.execution, std::time::Duration::ZERO);
}

#[test]
fn unknown_tool_and_invalid_parameters_are_rejected_by_executor() {
    let (mut a, mut h) = (strict_agent(), Host::default());
    let r = a.request_action(
        Action {
            tool: "powershell".into(),
            parameters: Parameters::None,
        },
        &mut h,
        0,
    );
    assert_eq!(r.error.unwrap().code, ErrorCode::UnknownTool);
    let r = a.request_action(
        Action {
            tool: "open_calculator".into(),
            parameters: Parameters::None,
        },
        &mut h,
        0,
    );
    assert_eq!(r.error.unwrap().code, ErrorCode::InvalidParameters);
    let r = a.request_action(
        Action {
            tool: "current_time".into(),
            parameters: Parameters::Application(Application::Calculator),
        },
        &mut h,
        0,
    );
    assert_eq!(r.error.unwrap().code, ErrorCode::InvalidParameters);
    assert_eq!((h.reads, h.launches), (0, 0));
}

#[test]
fn permissions_are_checked_before_tool_and_before_confirmation_execution() {
    let (mut a, mut h) = (strict_agent(), Host::default());
    a.permissions.read_only = false;
    assert_eq!(
        a.handle("hora", &mut h, 0).error.unwrap().code,
        ErrorCode::PermissionDenied
    );
    let pending = a
        .handle("abrí calculadora", &mut h, 1)
        .confirmation
        .unwrap();
    a.permissions.launch_application = false;
    assert_eq!(
        a.handle(&format!("confirmar {}", pending.request_id), &mut h, 2)
            .error
            .unwrap()
            .code,
        ErrorCode::PermissionDenied
    );
    assert_eq!((h.reads, h.launches), (0, 0));
}

#[test]
fn explicit_matching_confirmation_executes_once_and_replay_is_rejected() {
    let (mut a, mut h) = (strict_agent(), Host::default());
    let r = a.handle("abrí calculadora", &mut h, 0);
    assert_eq!(h.launches, 0);
    let pending = r.confirmation.unwrap();
    assert_eq!(
        pending.action.parameters,
        Parameters::Application(Application::Calculator)
    );
    let text = format!("confirmar {}", pending.request_id);
    assert_eq!(a.handle(&text, &mut h, 1).status, Status::Success);
    assert_eq!(a.handle(&text, &mut h, 2).status, Status::Rejected);
    assert_eq!(h.launches, 1);
}

#[test]
fn mismatched_expired_cancelled_and_superseded_confirmations_do_not_execute() {
    for case in 0..4 {
        let (mut a, mut h) = (strict_agent(), Host::default());
        let id = a.handle("abrí calculadora", &mut h, 0).request_id;
        let (confirm, now) = match case {
            0 => (format!("confirmar {}", id + 1), 1),
            1 => (format!("confirmar {id}"), 60_000),
            2 => {
                assert_eq!(a.handle("cancelar", &mut h, 1).status, Status::Cancelled);
                (format!("confirmar {id}"), 2)
            }
            _ => {
                a.handle("hora", &mut h, 1);
                (format!("confirmar {id}"), 2)
            }
        };
        assert_eq!(a.handle(&confirm, &mut h, now).status, Status::Rejected);
        assert_eq!(h.launches, 0);
    }
}

#[test]
fn registry_is_revalidated_after_confirmation() {
    let (mut a, mut h) = (strict_agent(), Host::default());
    let id = a.handle("abrí calculadora", &mut h, 0).request_id;
    a.registry = ToolRegistry::empty();
    assert_eq!(
        a.handle(&format!("confirmar {id}"), &mut h, 1)
            .error
            .unwrap()
            .code,
        ErrorCode::UnknownTool
    );
    assert_eq!(h.launches, 0);
}

#[test]
fn vague_consent_repeats_the_original_confirmation_without_extending_its_lifetime() {
    let (mut a, mut h) = (strict_agent(), Host::default());
    let original = a
        .handle("abrí calculadora", &mut h, 0)
        .confirmation
        .unwrap();
    let repeated = a.handle("sí", &mut h, 1000);
    assert_eq!(repeated.status, Status::NeedsConfirmation);
    assert_eq!(repeated.confirmation.unwrap(), original);
    assert_eq!(h.launches, 0);
    assert_eq!(
        a.handle("confirmar", &mut h, 60_000).status,
        Status::Rejected
    );
    assert_eq!(h.launches, 0);
}

#[test]
fn reference_requires_the_most_recent_unambiguous_application() {
    let (mut a, mut h) = (strict_agent(), Host::default());
    a.handle("abrí calculadora", &mut h, 0);
    let r = a.handle("abrila", &mut h, 1);
    assert_eq!(r.status, Status::NeedsConfirmation);
    assert_eq!(
        r.confirmation.unwrap().action.parameters,
        Parameters::Application(Application::Calculator)
    );
    a.handle("hora", &mut h, 2);
    assert_eq!(a.handle("abrila", &mut h, 3).status, Status::Clarification);
    assert_eq!(h.launches, 0);
}

#[test]
fn context_is_volatile_and_limited_to_eight_turns() {
    let (mut a, mut h) = (strict_agent(), Host::default());
    for _ in 0..20 {
        a.handle("hora", &mut h, 0);
    }
    assert_eq!(a.context.turns().count(), 8);
    assert!(a
        .context
        .turns()
        .all(|t| t.user == "hora" && t.assistant.contains("12:34:56")));
    assert_eq!(strict_agent().context.turns().count(), 0);
}

#[test]
fn tool_failure_is_structured_without_claiming_success() {
    let (mut a, mut h) = (
        strict_agent(),
        Host {
            fail: true,
            ..Default::default()
        },
    );
    let r = a.handle("hora", &mut h, 0);
    assert_eq!(r.status, Status::Error);
    assert_eq!(r.error.unwrap().code, ErrorCode::ToolFailed);
    assert_eq!(r.tool.as_deref(), Some("current_time"));
    assert!(r.data.is_empty());
}

#[test]
fn router_and_registry_can_be_extended_independently_without_bypassing_registration() {
    let (mut a, mut h) = (strict_agent(), Host::default());
    assert!(a.router.register("reloj local", ToolKind::CurrentTime));
    assert!(!a.router.register("reloj local", ToolKind::SystemStatus));
    a.registry = ToolRegistry::empty();
    assert_eq!(
        a.handle("reloj local", &mut h, 0).error.unwrap().code,
        ErrorCode::UnknownTool
    );
    assert!(a.registry.register(ToolKind::CurrentTime));
    assert!(!a.registry.register(ToolKind::CurrentTime));
    assert_eq!(a.handle("reloj local", &mut h, 1).status, Status::Success);
    assert_eq!(h.reads, 1);
    assert!(matches!(
        a.router
            .route(&Message::parse("reloj local").unwrap(), None),
        Intent::Tool(_)
    ));
}
