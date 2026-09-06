//! Terminal frontend for the existing Case conversation controller.
//!
//! REPLAI owns the transient draft and terminal resource. Only application
//! submission calls commit_parts; execution follows that durable boundary.
use super::conversation_controller::{
    ConversationAction, ConversationActionResult, ConversationCancellation, ConversationController,
    ConversationControllerStatus, ConversationExecutionResult, ConversationInputPart,
};
use super::provider;
use replai::{Editor, Event, Interaction, Prompt, Role};
use signal_hook::consts::SIGINT;
use signal_hook::iterator::{Handle, Signals};
use std::io::Write;
use std::thread::{self, JoinHandle};
use std::time::Duration;

// Application vocabulary: shared by this frontend's help and completion.
const COMMANDS: &[&str] = &[
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
    if let Some(output) = &value.output {
        println!("{output}");
    }
    // The posture is the controller's observation, never inferred from text.
    println!(
        "conversation_execution: {}",
        serde_json::to_string(&value.posture).map_err(|e| e.to_string())?
    );
    for event in &value.events {
        if let super::conversation_controller::ConversationApplicationEvent::ExecutionUnavailable { detail, .. } = event {
            eprintln!("{detail}");
        }
    }
    Ok(())
}

fn command(
    controller: &mut ConversationController,
    args: &[String],
    text: &str,
) -> Result<(), String> {
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
            ConversationActionResult::Execution { value } => render_execution(value)?,
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
        "unknown_command: {text}\ncommands: {}",
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
    let status = inspect(&mut controller)?;
    let prompt = Prompt::new(&format!("yai({case_id})")).map_err(|e| e.to_string())?;
    let mut input = Interaction::new(Editor::new(65_536, 200));
    println!("case_prompt: entered\ncase_ref: {case_id}\nsubject_ref: {}\ninteraction_thread: {}\ncommands: {}",
        status.participant_id, status.active_thread_id, COMMANDS.join(" | "));
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
                eprintln!("{error}");
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
                render_execution(controller.execute_committed_turn(&committed.turn.turn_id)?)
            })()
        };
        if let Err(error) = result {
            eprintln!("{error}");
        }
    }
}
