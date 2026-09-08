//! Terminal frontend for the existing Case conversation controller.
//!
//! REPLAI owns the transient draft and terminal resource. Only application
//! submission calls commit_parts; execution follows that durable boundary.
use super::conversation_controller::{
    CaseInspection, ConversationAction, ConversationActionResult, ConversationCancellation,
    ConversationController, ConversationControllerStatus, ConversationExecutionResult,
    ConversationInputPart,
};
use super::provider;
use replai::{Editor, Event, Interaction, Prompt, Role};
use signal_hook::consts::SIGINT;
use signal_hook::iterator::{Handle, Signals};
use std::io::Write;
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[path = "conversation_terminal/setup.rs"]
pub(super) mod setup;

// Application vocabulary: shared by this frontend's help and completion.
const COMMANDS: &[&str] = &[
    "/help",
    "/help all",
    "/setup",
    "/case",
    "/history",
    "/participants",
    "/resources",
    "/artifacts",
    "/policy",
    "/reviews",
    "/provider",
    "/capabilities",
    "/workflow",
    "/workflow bind ",
    "/workflow bind",
    "/workflow run ",
    "/workflow input ",
    "/workflow advance",
    "/workflow patch validate ",
    "/workflow patch adopt ",
    "/effects",
    "/operation ",
    "/memory",
    "/graph",
    "/handoffs",
    "/handoff offer ",
    "/handoff accept ",
    "/handoff result ",
    "/handoff reconcile ",
    "/verify",
    "/rebuild",
    "/executor ",
    "/attach ",
    "/attach",
    "/policy publish ",
    "/policy publish",
    "/connect ",
    "/connect",
    "/work ",
    "/review approve ",
    "/review",
    "/review deny ",
    "/review defer ",
    "/read ",
    "/discover ",
    "/query ",
    "/fetch ",
    "/test ",
    "/catalog ",
    "/admit ",
    "/admit",
    "/material ",
    "/thread status",
    "/thread new",
    "/thread list",
    "/thread use ",
    "/thread archive ",
    "/refresh",
    "/transcript on",
    "/transcript off",
    "/transcript status",
    "/memory propose",
    "/retry ",
    "/retry",
    "/cancel",
    "/exit",
    "/quit",
];

// This is application cancellation observation while execution owns the terminal,
// not an editor or a replacement for the controller's dispatch/retry policy.
struct ExecutionInterrupt {
    handle: Handle,
    worker: Option<JoinHandle<()>>,
}

impl ExecutionInterrupt {
    fn observe(cancellation: ConversationCancellation) -> Result<Self, String> {
        let mut signals = Signals::new([SIGINT]).map_err(|e| e.to_string())?;
        let handle = signals.handle();
        let worker = thread::Builder::new()
            .name("yai-conversation-interrupt".into())
            .spawn(move || {
                for _ in signals.forever() {
                    cancellation.request();
                }
            })
            .map_err(|e| e.to_string())?;
        Ok(Self {
            handle,
            worker: Some(worker),
        })
    }
}

impl Drop for ExecutionInterrupt {
    fn drop(&mut self) {
        self.handle.close();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn inspect(
    controller: &mut ConversationController,
) -> Result<ConversationControllerStatus, String> {
    match controller.apply(ConversationAction::Inspect)? {
        ConversationActionResult::Status { value } => Ok(value),
        _ => Err("conversation_inspect_result_mismatch".into()),
    }
}

fn render_execution(value: ConversationExecutionResult) -> Result<(), String> {
    if value.posture
        == super::conversation_controller::ConversationExecutionPosture::ProviderUnconfigured
    {
        eprintln!("Case provider not configured for conversation. Use /connect; this is not evidence that the endpoint is down.");
    }
    if let Some(output) = &value.output {
        println!("{}", literal_external_text(output));
    }
    if let Some(cognition) = &value.cognition {
        println!("conversation_cognition: {}", serde_json::to_string(&serde_json::json!({
            "turn_id": value.turn_id,
            "intent_id": cognition.request.request_id,
            "route": cognition.route,
            "source_closure_id": cognition.closure.closure_id,
            "target_id": cognition.primary.plan.selected_target_id,
            "lane_id": cognition.primary.plan.execution_lane_id,
            "validation_plan_id": cognition.primary.plan.plan_id,
            "execution_plan_id": cognition.primary.execution.as_ref().map(|execution| &execution.plan_id),
            "provider_result_id": value.provider_result_id,
            "selection_id": value.selection_id,
            "invocation_id": value.invocation_id,
            "projection_id": value.projection_id,
            "context_frame_id": value.context_frame_id,
            "recovered": cognition.primary.recovered
        })).map_err(|error| error.to_string())?);
    }
    if let Some(work) = &value.work {
        println!("{}", work.operator_summary());
    }
    // The posture is the controller's observation, never inferred from text.
    println!(
        "conversation_execution: {}",
        serde_json::to_string(&value.posture).map_err(|e| e.to_string())?
    );
    for event in &value.events {
        if let super::conversation_controller::ConversationApplicationEvent::ExecutionUnavailable { detail, .. } = event {
            eprintln!("{}", literal_external_text(detail));
        }
    }
    Ok(())
}

fn execute_visible(
    controller: &mut ConversationController,
    turn_id: &str,
) -> Result<ConversationExecutionResult, String> {
    // REPLAI has released the editor. This is truthful application wait output,
    // not token streaming or terminal mechanics; do not imply transport abort.
    let started = std::time::Instant::now();
    eprintln!("YAI: preparing execution for the committed Turn; waiting for the governed result.");
    std::thread::scope(|scope| {
        let (stop, signal) = std::sync::mpsc::channel::<()>();
        scope.spawn(move || {
            while signal.recv_timeout(Duration::from_secs(5)) == Err(std::sync::mpsc::RecvTimeoutError::Timeout) {
                eprintln!("YAI: execution pending ({}s); Ctrl-C requests application cancellation, not guaranteed transport abort.", started.elapsed().as_secs());
            }
        });
        let result = controller.execute_committed_turn(turn_id);
        drop(stop);
        result
    })
}

/// Application-data normalization for buffered output after REPLAI has released
/// the editor. This is not an ANSI/Markdown renderer: no cursor, style or terminal
/// mechanics live here. External payloads remain unchanged in canonical history.
fn literal_external_text(text: &str) -> String {
    let mut literal = String::with_capacity(text.len());
    for character in text.chars() {
        if (character.is_control() && !matches!(character, '\n' | '\t'))
            || matches!(character, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        {
            literal.extend(character.escape_default());
        } else {
            literal.push(character);
        }
    }
    literal
}

fn command(
    controller: &mut ConversationController,
    args: &[String],
    text: &str,
) -> Result<(), String> {
    if text == "/help" {
        println!("Case: /case /participants /resources /artifacts /history\nSetup: /setup /attach /connect /policy publish\nWork: /work TEXT /review /retry /cancel\nInspect: /policy /effects /workflow /memory /graph /verify\nAll actions: /help all\nExit: /exit");
        return Ok(());
    }
    if text == "/help all" {
        println!("{}", COMMANDS.join("\n"));
        return Ok(());
    }
    if text == "/setup" {
        return setup::participants(controller);
    }
    if text == "/connect" {
        return setup::connect(controller);
    }
    if text == "/attach" {
        let path = setup::ask("Resource definition file", None)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&controller.attach_resource(std::path::Path::new(&path))?)
                .map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    if text == "/policy publish" {
        let path = setup::ask("Policy source file", None)?;
        let reason = setup::ask(
            "Publication reason (explicit approval of this policy source)",
            None,
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(
                &controller.publish_policy(std::path::Path::new(&path), &reason)?
            )
            .map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    if text == "/review" {
        return setup::review(controller);
    }
    if text == "/workflow bind" {
        let path = setup::ask("Workflow definition file", None)?;
        println!(
            "{}",
            serde_json::to_string_pretty(
                &controller.configure_workflow(std::path::Path::new(&path))?
            )
            .map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    if text == "/admit" {
        let resource = super::conversation_controller::scoped_name(
            "resource:",
            &setup::ask("Discovery resource", None)?,
        )?;
        let digest = setup::ask(
            "Exact discovered candidate digest (not the observation digest)",
            None,
        )?;
        let path = setup::ask("Exact discovered relative path", None)?;
        let action = yai_core_engine::effect::access::ResourceAction::AdmitContent {
            path,
            candidate_digest: digest,
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&controller.request_resource(&resource, action)?)
                .map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    if text == "/retry" {
        let id = controller.latest_turn_id()?;
        println!("Retrying exact Turn: {id}");
        let _interrupt = ExecutionInterrupt::observe(controller.cancellation())?;
        return render_execution(execute_visible(controller, &id)?);
    }
    if let Some(id) = text.strip_prefix("/operation ") {
        println!(
            "{}",
            serde_json::to_string_pretty(&controller.inspect_operation(id.trim())?)
                .map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    if text == "/rebuild" {
        println!(
            "{}",
            serde_json::to_string_pretty(&controller.rebuild_case_views()?)
                .map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    if let Some(request) = text.strip_prefix("/handoff ") {
        use super::conversation_controller::CaseHandoffAction;
        use yai_core_engine::handoff::HandoffOutcome;
        let (verb, rest) = request.split_once(' ').ok_or("handoff_action_required")?;
        let action = match verb {
            "offer" => {
                let mut fields = rest.splitn(3, ' ');
                CaseHandoffAction::Offer {
                    target: fields.next().ok_or("handoff_target_required")?.into(),
                    required_role: fields.next().ok_or("handoff_role_required")?.into(),
                    text: fields.next().ok_or("handoff_text_required")?.into(),
                }
            }
            "accept" => {
                let (source, id) = rest
                    .split_once(' ')
                    .ok_or("handoff_syntax: /handoff accept SOURCE ID")?;
                CaseHandoffAction::Accept {
                    source: source.into(),
                    handoff_id: id.into(),
                }
            }
            "result" => {
                let mut fields = rest.splitn(4, ' ');
                let id = fields.next().ok_or("handoff_id_required")?;
                let outcome = match fields.next() {
                    Some("succeeded") => HandoffOutcome::Succeeded,
                    Some("failed") => HandoffOutcome::Failed,
                    Some("cancelled") => HandoffOutcome::Cancelled,
                    _ => return Err("handoff_outcome_required: succeeded|failed|cancelled".into()),
                };
                CaseHandoffAction::Result {
                    handoff_id: id.into(),
                    outcome,
                    evidence_ref: fields
                        .next()
                        .ok_or("handoff_local_evidence_required")?
                        .into(),
                    text: fields.next().ok_or("handoff_text_required")?.into(),
                }
            }
            "reconcile" => CaseHandoffAction::Reconcile {
                handoff_id: rest.into(),
            },
            _ => return Err("handoff_action_unknown".into()),
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&controller.handoff(action)?).map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    let section = match text {
        "/case" => Some(CaseInspection::Status),
        "/history" => Some(CaseInspection::History),
        "/participants" => Some(CaseInspection::Participants),
        "/resources" => Some(CaseInspection::Resources),
        "/artifacts" => Some(CaseInspection::Artifacts),
        "/policy" => Some(CaseInspection::Policy),
        "/reviews" => Some(CaseInspection::Reviews),
        "/provider" => Some(CaseInspection::Provider),
        "/capabilities" => Some(CaseInspection::Capabilities),
        "/workflow" => Some(CaseInspection::Workflow),
        "/effects" => Some(CaseInspection::Effects),
        "/memory" => Some(CaseInspection::Memory),
        "/graph" => Some(CaseInspection::Graph),
        "/handoffs" => Some(CaseInspection::Handoffs),
        "/verify" => Some(CaseInspection::Verify),
        _ => None,
    };
    if let Some(section) = section {
        println!(
            "{}",
            serde_json::to_string_pretty(&controller.inspect_case(section)?)
                .map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    if let Some(executor) = text.strip_prefix("/executor ") {
        controller.select_executor(executor.trim())?;
        println!(
            "executor_selected: {} (no authority transferred)",
            executor.trim()
        );
        return Ok(());
    }
    if let Some(path) = text.strip_prefix("/attach ") {
        println!(
            "{}",
            serde_json::to_string_pretty(
                &controller.attach_resource(std::path::Path::new(path.trim()))?
            )
            .map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    if let Some(request) = text.strip_prefix("/policy publish ") {
        let (path, reason) = request
            .split_once(' ')
            .ok_or("policy_syntax: /policy publish FILE REASON")?;
        println!(
            "{}",
            serde_json::to_string_pretty(
                &controller.publish_policy(std::path::Path::new(path), reason)?
            )
            .map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    if let Some(request) = text.strip_prefix("/connect ") {
        use yai_core_engine::provider_governance::ProviderLocality;
        let fields = request.split_whitespace().collect::<Vec<_>>();
        if fields.len() < 6 {
            return Err("connect_syntax: /connect ENDPOINT MODEL --trust approve --attest evidence:REF [--locality loopback|private_network|remote] [--credential-ref env:NAME] [--replace]".into());
        }
        let (mut trust, mut evidence, mut locality, mut credential, mut replace) =
            (false, None, ProviderLocality::Loopback, "none", false);
        let mut seen = std::collections::BTreeSet::new();
        let mut index = 2;
        while index < fields.len() {
            let flag = fields[index];
            if !seen.insert(flag) {
                return Err("connect_duplicate_option".into());
            }
            if flag == "--replace" {
                replace = true;
                index += 1;
                continue;
            }
            let value = fields
                .get(index + 1)
                .ok_or("connect_option_requires_value")?;
            match flag {
                "--trust" if *value == "approve" => trust = true,
                "--attest" => evidence = Some(*value),
                "--locality" => {
                    locality = match *value {
                        "loopback" => ProviderLocality::Loopback,
                        "private_network" => ProviderLocality::PrivateNetwork,
                        "remote" => ProviderLocality::Remote,
                        _ => return Err("connect_locality_invalid".into()),
                    }
                }
                "--credential-ref" => credential = value,
                _ => return Err("connect_option_invalid".into()),
            }
            index += 2;
        }
        let result =
            controller.connect_provider(super::conversation_controller::ProviderConnection {
                endpoint: fields[0],
                model: fields[1],
                locality,
                credential_ref: credential,
                trust_approved: trust,
                suitability_ref: Some(evidence.ok_or("connect_explicit_attestation_required")?),
                replace,
                expected_generation: None,
            })?;
        println!(
            "{}",
            serde_json::to_string_pretty(&result).map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    for prefix in [
        "/read ",
        "/discover ",
        "/query ",
        "/fetch ",
        "/test ",
        "/catalog ",
        "/admit ",
        "/material ",
    ] {
        if let Some(request) = text.strip_prefix(prefix) {
            use yai_core_engine::effect::access::ResourceAction;
            let (resource, argument) = request
                .trim()
                .split_once(' ')
                .unwrap_or((request.trim(), ""));
            let argument = argument.trim();
            let action = match prefix {
                "/read " => ResourceAction::FilesystemRead {
                    path: argument.into(),
                },
                "/discover " => ResourceAction::Discover {
                    path: argument.into(),
                },
                "/query " => ResourceAction::DatabaseQuery {
                    name: argument.into(),
                },
                "/fetch " => ResourceAction::HttpFetch {
                    name: argument.into(),
                },
                "/test " => ResourceAction::ProcessRun {
                    name: argument.into(),
                },
                "/catalog " if argument.is_empty() => ResourceAction::McpCatalog,
                "/material " => ResourceAction::ContentRead {
                    admission_id: argument.into(),
                },
                "/admit " => {
                    let (digest, path) = argument
                        .split_once(' ')
                        .ok_or("admit_syntax: /admit RESOURCE DIGEST PATH")?;
                    ResourceAction::AdmitContent {
                        path: path.into(),
                        candidate_digest: digest.into(),
                    }
                }
                _ => return Err("resource_action_syntax_invalid".into()),
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&controller.request_resource(
                    &super::conversation_controller::scoped_name("resource:", resource)?,
                    action
                )?)
                .map_err(|e| e.to_string())?
            );
            return Ok(());
        }
    }
    if let Some(task) = text.strip_prefix("/work ") {
        // Deliberate bounded work action, never inferred from prose or modality.
        let committed = controller.commit_work(
            task.to_string(),
            yai_core_engine::conversation::CaseWorkLimits {
                invocations: 24,
                operations: 23,
                effects: 6,
                max_input_units: 65_536,
            },
        )?;
        println!(
            "conversation_turn: {}\nconversation_generation: {}",
            committed.turn.turn_id, committed.generation
        );
        std::io::stdout().flush().map_err(|e| e.to_string())?;
        let _interrupt = ExecutionInterrupt::observe(controller.cancellation())?;
        return render_execution(execute_visible(controller, &committed.turn.turn_id)?);
    }
    if let Some(node) = text.strip_prefix("/workflow run ") {
        let committed = controller.commit_workflow_node(node.trim())?;
        println!(
            "conversation_turn: {}\nconversation_generation: {}",
            committed.turn.turn_id, committed.generation
        );
        std::io::stdout().flush().map_err(|e| e.to_string())?;
        let _interrupt = ExecutionInterrupt::observe(controller.cancellation())?;
        return render_execution(execute_visible(controller, &committed.turn.turn_id)?);
    }
    let workflow_action = if let Some(path) = text.strip_prefix("/workflow bind ") {
        Some(controller.configure_workflow(std::path::Path::new(path.trim()))?)
    } else if let Some(input) = text.strip_prefix("/workflow input ") {
        let (node, value) = input
            .split_once(' ')
            .ok_or("workflow_input_syntax: NODE TEXT")?;
        Some(controller.workflow_input(node, value)?)
    } else if text == "/workflow advance" {
        Some(controller.workflow_advance()?)
    } else if let Some(patch) = text.strip_prefix("/workflow patch validate ") {
        Some(controller.workflow_patch_validate(patch.trim())?)
    } else if let Some(patch) = text.strip_prefix("/workflow patch adopt ") {
        Some(controller.workflow_patch_adopt(patch.trim())?)
    } else {
        None
    };
    if let Some(value) = workflow_action {
        println!(
            "{}",
            serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    if let Some(request) = text.strip_prefix("/review ") {
        let mut fields = request.splitn(4, ' ');
        let action = match fields.next() {
            Some("approve") => yai_core_engine::transition::ReviewActionKind::Approve,
            Some("deny") => yai_core_engine::transition::ReviewActionKind::Deny,
            Some("defer") => yai_core_engine::transition::ReviewActionKind::Defer,
            _ => {
                return Err(
                    "review_syntax: /review approve|deny|defer REVIEW PARTICIPANT reason".into(),
                )
            }
        };
        return controller.review_action(
            fields.next().ok_or("review_id_required")?,
            fields.next().ok_or("reviewer_required")?,
            action,
            fields.next().ok_or("review_reason_required")?,
        );
    }
    let action = match text {
        "/thread status" => Some(ConversationAction::Inspect),
        "/thread list" => Some(ConversationAction::ListThreads),
        "/cancel" => Some(ConversationAction::Cancel),
        _ if text == "/thread new" || text.starts_with("/thread new ") => {
            Some(ConversationAction::NewThread)
        }
        _ if text.starts_with("/thread use ") => Some(ConversationAction::UseThread {
            thread_id: text[12..].trim().into(),
        }),
        _ if text.starts_with("/retry ") => Some(ConversationAction::Retry {
            turn_id: text[7..].trim().into(),
        }),
        _ => None,
    };
    if let Some(action) = action {
        let _interrupt = if matches!(action, ConversationAction::Retry { .. }) {
            Some(ExecutionInterrupt::observe(controller.cancellation())?)
        } else {
            None
        };
        match controller.apply(action)? {
            ConversationActionResult::Execution { value } => render_execution(*value)?,
            value => println!(
                "conversation: {}",
                serde_json::to_string(&value).map_err(|e| e.to_string())?
            ),
        }
        return Ok(());
    }
    if text == "/refresh" {
        // The controller rereads canonical state at every application action.
        println!(
            "case_prompt: refreshed\nconversation: {}",
            serde_json::to_string(&inspect(controller)?).map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    if text.starts_with("/transcript ")
        || text.starts_with("/memory propose")
        || text.starts_with("/thread archive ")
    {
        let status = inspect(controller)?;
        provider::terminal_compatibility_command(args, text, &status.active_thread_id)?;
        if text.strip_prefix("/thread archive ").map(str::trim)
            == Some(status.active_thread_id.as_str())
        {
            // Archive is legacy host metadata, never deletion of canonical Turns.
            // Selecting a fresh thread uses the existing canonical controller seam.
            let selected = controller.apply(ConversationAction::NewThread)?;
            println!(
                "conversation: {}",
                serde_json::to_string(&selected).map_err(|e| e.to_string())?
            );
        }
        return Ok(());
    }
    println!(
        "unknown_command: {}\ncommands: {}",
        literal_external_text(text),
        COMMANDS.join(" | ")
    );
    Ok(())
}

pub(super) fn run(args: &[String]) -> Result<(), String> {
    let dry_run = args.iter().any(|arg| arg == "--dry-run");
    if !dry_run
        && args.iter().any(|arg| {
            matches!(
                arg.as_str(),
                "--provider-id"
                    | "--base-url"
                    | "--model"
                    | "--api-key-env"
                    | "--language-mode"
                    | "--continuation-capable"
                    | "--provider-runtime-id"
                    | "--continuation-ref"
            )
        })
    {
        return Err("interactive_prompt_uses_governed_provider_binding: configure `yai case provider bind`; direct invocation options remain available with --once or --dry-run".into());
    }
    let (case_id, participant) = provider::terminal_context(args)?;
    let mut controller = ConversationController::open(&case_id, participant.as_deref())?;
    if let Some(executor) = super::optional_arg(args, "--executor") {
        controller.select_executor(&executor)?;
    }
    let status = inspect(&mut controller)?;
    let prompt = Prompt::new(&format!("yai({case_id})")).map_err(|e| e.to_string())?;
    let mut input = Interaction::new(Editor::new(65_536, 200));
    println!("case_prompt: entered\ncase_ref: {case_id}\nsubject_ref: {}\ninteraction_thread: {}\nUse /help for Case actions; /connect to configure a provider; /exit to leave.",
        status.participant_id, status.active_thread_id);
    loop {
        input
            .open(&std::io::stdin(), &std::io::stdout(), prompt.clone())
            .map_err(|e| e.to_string())?;
        let text = loop {
            match input
                .poll(Duration::from_millis(100))
                .map_err(|e| e.to_string())?
            {
                Some(Event::Submitted(text)) => break Some(text),
                Some(Event::Interrupted) => break None,
                Some(Event::EndOfInput) => return Ok(()),
                Some(Event::CompletionRequested) => {
                    let cursor = input.editor().cursor();
                    let prefix = &input.editor().text()[..cursor];
                    let candidates: Vec<_> = COMMANDS
                        .iter()
                        .filter(|candidate| candidate.starts_with(prefix))
                        .collect();
                    if prefix.starts_with('/') {
                        if let [candidate] = candidates.as_slice() {
                            input
                                .complete(0..cursor, candidate)
                                .map_err(|e| e.to_string())?;
                        } else if !candidates.is_empty() {
                            input
                                .external_output(
                                    Role::Dim,
                                    &candidates
                                        .iter()
                                        .map(|s| **s)
                                        .collect::<Vec<_>>()
                                        .join(" | "),
                                )
                                .map_err(|e| e.to_string())?;
                        }
                    }
                }
                Some(Event::Rejected(error)) => input
                    .external_output(Role::Warning, &error.to_string())
                    .map_err(|e| e.to_string())?,
                None => {}
            }
        };
        input.editor_mut().map_err(|e| e.to_string())?.clear();
        let Some(text) = text else { continue };
        let trimmed = text.trim();
        if trimmed.is_empty() {
            continue;
        }
        if matches!(trimmed, "/exit" | "/quit") {
            return Ok(());
        }
        if trimmed.starts_with('/') {
            if let Err(error) = command(&mut controller, args, trimmed) {
                eprintln!("{}", literal_external_text(&error));
            }
            continue;
        }
        input
            .editor_mut()
            .map_err(|e| e.to_string())?
            .admit_history(&text)
            .map_err(|e| e.to_string())?;
        let result = if dry_run {
            provider::terminal_dry_run(args, &text)
        } else {
            (|| {
                let committed =
                    controller.commit_parts(vec![ConversationInputPart::Text { text }])?;
                let _interrupt = ExecutionInterrupt::observe(controller.cancellation())?;
                println!(
                    "conversation_turn: {}\nconversation_generation: {}",
                    committed.turn.turn_id, committed.generation
                );
                std::io::stdout().flush().map_err(|e| e.to_string())?;
                render_execution(execute_visible(&mut controller, &committed.turn.turn_id)?)
            })()
        };
        if let Err(error) = result {
            eprintln!("{}", literal_external_text(&error));
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn external_text_cannot_control_terminal_or_hide_direction() {
        let source = "risultato\nè leggibile\t\u{1b}]52;c;secret\u{7}\r\u{202e}hidden\u{9b}2J";
        let display = super::literal_external_text(source);
        assert!(display.starts_with("risultato\nè leggibile\t"));
        assert!(!display
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t'));
        assert!(!display.contains('\u{202e}'));
        assert!(display.contains("\\u{1b}]52;c;secret"));
        assert!(
            source.contains('\u{1b}'),
            "display does not mutate original provider material"
        );
    }
}
