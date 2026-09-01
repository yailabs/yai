use serde::Serialize;

use super::registry::{
    registry_digest, Descriptor, ProductRoot, ProductSection, Visibility, PRODUCT_ROOTS, REGISTRY,
    REGISTRY_SCHEMA,
};

#[derive(Clone, Debug)]
pub(crate) struct HelpRequest {
    pub path: Vec<String>,
    pub advanced: bool,
    pub json: bool,
}

impl HelpRequest {
    pub(crate) fn root(advanced: bool, json: bool) -> Self {
        Self {
            path: Vec::new(),
            advanced,
            json,
        }
    }
}

#[derive(Serialize)]
struct Discovery<'a> {
    schema: &'static str,
    cli_registry_schema: &'static str,
    cli_registry_digest: String,
    product: &'static str,
    product_roots: &'static [ProductRoot],
    operations: Vec<&'a Descriptor>,
}

pub(crate) fn render(request: &HelpRequest) {
    let operations = visible_operations(request);
    if request.json {
        let discovery = Discovery {
            schema: "yai.cli.command_discovery.v1",
            cli_registry_schema: REGISTRY_SCHEMA,
            cli_registry_digest: registry_digest(),
            product: "YAI — governed operational AI runtime",
            product_roots: PRODUCT_ROOTS,
            operations,
        };
        println!(
            "{}",
            serde_json::to_string(&discovery).expect("command discovery serializes")
        );
        return;
    }
    if request.path.is_empty() {
        render_root(request.advanced);
    } else if let Some(operation) = operations
        .iter()
        .find(|operation| path_equals(operation.path, &request.path))
    {
        render_leaf(operation);
    } else {
        render_subtree(&request.path, &operations);
    }
}

fn visible_operations(request: &HelpRequest) -> Vec<&'static Descriptor> {
    REGISTRY
        .iter()
        .filter(|descriptor| {
            (request.advanced || descriptor.visibility == Visibility::Product)
                && (request.path.is_empty() || path_starts_with(descriptor.path, &request.path))
        })
        .collect()
}

fn render_root(advanced: bool) {
    println!("YAI — governed operational AI runtime");
    println!();
    println!("Usage: yai <command> [arguments]");
    println!();
    for (index, section) in ProductSection::ORDERED.into_iter().enumerate() {
        if index > 0 {
            println!();
        }
        println!("{}", section.label());
        for root in PRODUCT_ROOTS.iter().filter(|root| root.section == section) {
            print_command(root.word, root.description);
        }
    }
    if advanced {
        println!();
        println!("ADVANCED / PLUMBING / COMPATIBILITY");
        let mut roots = REGISTRY
            .iter()
            .filter(|descriptor| descriptor.visibility != Visibility::Product)
            .filter_map(|descriptor| descriptor.path.first().copied())
            .collect::<Vec<_>>();
        roots.sort();
        roots.dedup();
        for root in roots {
            let mut classes = REGISTRY
                .iter()
                .filter(|descriptor| descriptor.path.first() == Some(&root))
                .map(|descriptor| format!("{:?}", descriptor.visibility).to_lowercase())
                .collect::<Vec<_>>();
            classes.sort();
            classes.dedup();
            print_command(root, &classes.join(", "));
        }
    } else {
        println!();
        println!("Use `yai help --advanced` for engineering and compatibility tools.");
    }
    println!("Use `yai <command> --help` for exact syntax; add `--json` for machine output.");
}

fn render_subtree(path: &[String], operations: &[&Descriptor]) {
    println!("YAI {}", path.join(" ").to_uppercase());
    println!();
    println!("Usage: yai {} <command> [arguments]", path.join(" "));
    println!();
    for descriptor in operations {
        let suffix = &descriptor.path[path.len()..];
        if suffix.is_empty() {
            continue;
        }
        let display = suffix.join(" ");
        print_command(&display, descriptor.description);
    }
}

fn render_leaf(descriptor: &Descriptor) {
    println!("{}", descriptor.description);
    println!();
    print!("Usage: yai {}", descriptor.path.join(" "));
    for positional in descriptor.positionals {
        if positional.required {
            print!(" <{}>", positional.name.to_uppercase());
        } else {
            print!(" [{}]", positional.name.to_uppercase());
        }
    }
    if !descriptor.flags.is_empty() {
        print!(" [options]");
    }
    if !descriptor.rules.is_empty() {
        println!();
        println!("Constraints:");
        for rule in descriptor.rules {
            println!("  {:?}", rule);
        }
    }
    if matches!(
        descriptor.output,
        super::registry::OutputCapability::Structured
    ) {
        print!(" [--json]");
    }
    println!();
    println!();
    println!("Operation:  {}", descriptor.operation_id);
    println!("Lane:       {:?}", descriptor.lane);
    println!("Visibility: {:?}", descriptor.visibility);
    if !descriptor.positionals.is_empty() {
        println!();
        println!("Arguments:");
        for positional in descriptor.positionals {
            let choices = if positional.choices.is_empty() {
                String::new()
            } else {
                format!(" [{}]", positional.choices.join("|"))
            };
            println!(
                "  {:<20} {}{}",
                positional.name.to_uppercase(),
                if positional.required {
                    "required"
                } else {
                    "optional"
                },
                choices
            );
        }
    }
    if !descriptor.flags.is_empty() {
        println!();
        println!("Options:");
        for flag in descriptor.flags {
            let value = flag
                .value_name
                .map(|name| format!(" <{name}>"))
                .unwrap_or_default();
            let required = if flag.required {
                "required"
            } else {
                "optional"
            };
            let repeatable = if flag.repeatable { ", repeatable" } else { "" };
            let choices = if flag.choices.is_empty() {
                String::new()
            } else {
                format!(" [{}]", flag.choices.join("|"))
            };
            println!(
                "  {:<20} {required}{repeatable}{choices}",
                format!("{}{value}", flag.name)
            );
        }
    }
    println!("  {:<20} show this help", "-h, --help");
    if matches!(
        descriptor.output,
        super::registry::OutputCapability::Structured
    ) {
        println!("  {:<20} structured yai.cli.result.v1", "--json");
    }
    if !descriptor.aliases.is_empty() {
        println!();
        println!("Compatibility aliases:");
        for alias in descriptor.aliases {
            println!("  yai {}", alias.join(" "));
        }
    }
}

fn print_command(command: &str, description: &str) {
    println!("  {:<18} {}", command, description);
}

fn path_starts_with(path: &[&str], prefix: &[String]) -> bool {
    path.len() >= prefix.len() && path.iter().zip(prefix).all(|(left, right)| left == right)
}

fn path_equals(path: &[&str], candidate: &[String]) -> bool {
    path.len() == candidate.len() && path_starts_with(path, candidate)
}
