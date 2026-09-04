use std::collections::BTreeMap;

use super::help::HelpRequest;
use super::output::CliError;
use super::registry::{Descriptor, FlagSpec, OutputCapability, SyntaxRule, Visibility, REGISTRY};
use super::syntax_error;

#[derive(Debug)]
pub(crate) struct Invocation {
    pub descriptor: &'static Descriptor,
    pub positionals: BTreeMap<&'static str, String>,
    pub flags: BTreeMap<&'static str, Vec<String>>,
    pub json: bool,
    pub compatibility_syntax: bool,
}

impl Invocation {
    pub(crate) fn positional(&self, name: &str) -> Option<&str> {
        self.positionals.get(name).map(String::as_str)
    }

    pub(crate) fn flag(&self, name: &str) -> Option<&str> {
        self.flags
            .get(name)
            .and_then(|values| values.last())
            .map(String::as_str)
    }

    pub(crate) fn legacy_args(&self) -> Vec<String> {
        let mut args = self
            .descriptor
            .legacy_path
            .iter()
            .map(|value| (*value).to_string())
            .collect::<Vec<_>>();
        for positional in self.descriptor.positionals {
            if let Some(value) = self.positionals.get(positional.name) {
                if let Some(flag) = positional.legacy_flag {
                    args.push(flag.to_string());
                }
                args.push(value.clone());
            }
        }
        for spec in self.descriptor.flags {
            if let Some(values) = self.flags.get(spec.name) {
                for value in values {
                    args.push(spec.legacy_name.unwrap_or(spec.name).to_string());
                    if spec.value_name.is_some() {
                        args.push(value.clone());
                    }
                }
            }
        }
        args
    }
}

pub(crate) enum ParseOutcome {
    Help(HelpRequest),
    Invoke(Invocation),
}

pub(crate) fn parse(args: &[String]) -> Result<ParseOutcome, CliError> {
    super::registry::validate().map_err(CliError::internal)?;

    if args.is_empty() {
        return Ok(ParseOutcome::Help(HelpRequest::root(false, false)));
    }
    if args == ["--version"] {
        return parse(&["version".to_string()]);
    }
    if args.first().map(String::as_str) == Some("help") {
        let (path, advanced, json) = parse_help_command(&args[1..])?;
        return Ok(ParseOutcome::Help(HelpRequest {
            path,
            advanced,
            json,
        }));
    }
    if let Some(index) = args.iter().position(|arg| arg == "--help" || arg == "-h") {
        let path = resolve_help_path(&args[..index])?;
        return Ok(ParseOutcome::Help(HelpRequest {
            path,
            // Explicit leaf/subtree help is discoverable even when the
            // operation is intentionally absent from first-contact help.
            advanced: index > 0 || args.iter().any(|arg| arg == "--advanced"),
            json: args.iter().any(|arg| arg == "--json"),
        }));
    }

    let (descriptor, path_len, invoked_path) = resolve_path(args)?;
    if descriptor.visibility == Visibility::Removed {
        let successor = descriptor
            .removed_successor
            .expect("validated removed successor")
            .join(" ");
        return Err(CliError::removed(
            descriptor.operation_id,
            format!("command `yai {}` was removed", invoked_path.join(" ")),
            format!("use `yai {successor}`"),
        ));
    }
    let path_is_alias = invoked_path
        .iter()
        .map(String::as_str)
        .ne(descriptor.path.iter().copied());
    parse_invocation(descriptor, path_is_alias, &args[path_len..]).map(ParseOutcome::Invoke)
}

fn parse_help_command(args: &[String]) -> Result<(Vec<String>, bool, bool), CliError> {
    let mut path = Vec::new();
    let mut advanced = false;
    let mut json = false;
    let mut self_help = false;
    for token in args {
        match token.as_str() {
            "--advanced" if !advanced => advanced = true,
            "--json" if !json => json = true,
            "--help" | "-h" if !self_help => self_help = true,
            "--advanced" | "--json" => {
                return Err(syntax_error(format!(
                    "duplicate nonrepeatable flag: {token}"
                )))
            }
            "--help" | "-h" => return Err(syntax_error("duplicate nonrepeatable flag: --help")),
            token if token.starts_with('-') => {
                return Err(syntax_error(format!(
                    "unknown flag `{token}` for `yai help`"
                )))
            }
            _ => path.push(token.clone()),
        }
    }
    if !path.is_empty() {
        path = resolve_help_path(&path)?;
        advanced = true;
    }
    Ok((path, advanced, json))
}

fn resolve_help_path(args: &[String]) -> Result<Vec<String>, CliError> {
    if args.is_empty() {
        return Ok(Vec::new());
    }
    if let Ok((descriptor, _, _)) = resolve_path(args) {
        return Ok(descriptor
            .path
            .iter()
            .map(|word| (*word).to_string())
            .collect());
    }
    let entered = args
        .iter()
        .take_while(|arg| !arg.starts_with('-'))
        .cloned()
        .collect::<Vec<_>>();
    if REGISTRY
        .iter()
        .any(|descriptor| path_has_prefix(descriptor.path, &entered))
    {
        return Ok(entered);
    }
    Err(CliError::unknown_command(
        entered.join(" "),
        nearest_path(&entered),
    ))
}

fn path_has_prefix(path: &[&str], prefix: &[String]) -> bool {
    path.len() >= prefix.len()
        && path
            .iter()
            .zip(prefix)
            .all(|(expected, actual)| expected == actual)
}

fn resolve_path(args: &[String]) -> Result<(&'static Descriptor, usize, Vec<String>), CliError> {
    let mut matches = Vec::new();
    for descriptor in REGISTRY {
        if path_matches(descriptor.path, args) {
            matches.push((descriptor, descriptor.path.len(), descriptor.path));
        }
        for alias in descriptor.aliases {
            if path_matches(alias, args) {
                matches.push((descriptor, alias.len(), *alias));
            }
        }
    }
    matches.sort_by_key(|(_, len, _)| std::cmp::Reverse(*len));
    if let Some((descriptor, len, path)) = matches.first() {
        return Ok((
            descriptor,
            *len,
            path.iter().map(|word| (*word).to_string()).collect(),
        ));
    }

    let entered = args
        .iter()
        .take_while(|arg| !arg.starts_with('-'))
        .cloned()
        .collect::<Vec<_>>();
    let suggestion = nearest_path(&entered);
    Err(CliError::unknown_command(entered.join(" "), suggestion))
}

fn path_matches(path: &[&str], args: &[String]) -> bool {
    args.len() >= path.len()
        && path
            .iter()
            .zip(args)
            .all(|(expected, actual)| expected == actual)
}

fn nearest_path(entered: &[String]) -> Option<String> {
    if entered.is_empty() {
        return None;
    }
    let mut candidates = REGISTRY
        .iter()
        .filter(|descriptor| descriptor.visibility != Visibility::Removed)
        .map(|descriptor| {
            let path = descriptor.path.join(" ");
            let comparable = entered
                .iter()
                .take(descriptor.path.len())
                .cloned()
                .collect::<Vec<_>>()
                .join(" ");
            (bounded_distance(&comparable, &path), path)
        })
        .filter(|(distance, _)| *distance <= 3)
        .collect::<Vec<_>>();
    candidates.sort();
    if candidates.len() == 1
        || candidates.first().map(|item| item.0) < candidates.get(1).map(|item| item.0)
    {
        candidates.first().map(|(_, path)| path.clone())
    } else {
        None
    }
}

fn bounded_distance(left: &str, right: &str) -> usize {
    let mut row = (0..=right.len()).collect::<Vec<_>>();
    for (i, a) in left.bytes().enumerate() {
        let mut next = vec![i + 1];
        for (j, b) in right.bytes().enumerate() {
            next.push(
                (row[j + 1] + 1)
                    .min(next[j] + 1)
                    .min(row[j] + usize::from(a != b)),
            );
        }
        row = next;
    }
    row[right.len()]
}

fn parse_invocation(
    descriptor: &'static Descriptor,
    path_is_alias: bool,
    args: &[String],
) -> Result<Invocation, CliError> {
    let mut positionals = BTreeMap::new();
    let mut flags: BTreeMap<&'static str, Vec<String>> = BTreeMap::new();
    let mut positional_selectors: BTreeMap<&'static str, String> = BTreeMap::new();
    let mut positional_values = Vec::new();
    let mut json = false;
    let mut compatibility_syntax = path_is_alias;
    let mut index = 0;

    while index < args.len() {
        let token = &args[index];
        if token == "--json" {
            if descriptor.output != OutputCapability::Structured {
                return Err(syntax_error(format!(
                    "`yai {}` does not declare structured JSON output",
                    descriptor.path.join(" ")
                )));
            }
            if json {
                return Err(syntax_error("duplicate nonrepeatable flag: --json"));
            }
            json = true;
            index += 1;
            continue;
        }
        if token.starts_with('-') {
            if let Some(positional) = descriptor
                .positionals
                .iter()
                .find(|positional| positional.legacy_flag == Some(token.as_str()))
            {
                compatibility_syntax = true;
                if positional_selectors.contains_key(positional.name) {
                    return Err(syntax_error(format!(
                        "duplicate nonrepeatable selector: {token}"
                    )));
                }
                index += 1;
                let value = args
                    .get(index)
                    .filter(|value| !value.starts_with('-'))
                    .cloned()
                    .ok_or_else(|| syntax_error(format!("{token} requires a value")))?;
                positional_selectors.insert(positional.name, value);
                index += 1;
                continue;
            }
            let spec = find_flag(descriptor, token).ok_or_else(|| {
                syntax_error(format!(
                    "unknown flag `{token}` for `yai {}`",
                    descriptor.path.join(" ")
                ))
            })?;
            if spec.name != token {
                compatibility_syntax = true;
            }
            if !spec.repeatable && flags.contains_key(spec.name) {
                return Err(syntax_error(format!(
                    "duplicate nonrepeatable flag: {}",
                    spec.name
                )));
            }
            let value = if spec.value_name.is_some() {
                index += 1;
                args.get(index)
                    .filter(|value| !value.starts_with('-'))
                    .cloned()
                    .ok_or_else(|| syntax_error(format!("{} requires a value", spec.name)))?
            } else {
                "true".to_string()
            };
            validate_flag_value(spec, &value)?;
            flags.entry(spec.name).or_default().push(value);
            index += 1;
        } else {
            positional_values.push(token.clone());
            index += 1;
        }
    }

    for (index, spec) in descriptor.positionals.iter().enumerate() {
        if let Some(value) = positional_values.get(index) {
            if let Some(selector) = spec.legacy_flag {
                if positional_selectors.contains_key(spec.name) {
                    return Err(syntax_error(format!(
                        "conflicting positional {} and {} selector",
                        spec.name, selector
                    )));
                }
            }
            validate_positional_value(spec, value)?;
            positionals.insert(spec.name, value.clone());
        } else if spec.legacy_flag.is_some() {
            if let Some(value) = positional_selectors.remove(spec.name) {
                validate_positional_value(spec, &value)?;
                positionals.insert(spec.name, value);
            } else if spec.required {
                return Err(syntax_error(format!(
                    "missing required positional: {}",
                    spec.name
                )));
            }
        } else if spec.required {
            return Err(syntax_error(format!(
                "missing required positional: {}",
                spec.name
            )));
        }
    }
    if positional_values.len() > descriptor.positionals.len() {
        return Err(syntax_error(format!(
            "unexpected extra positional: {}",
            positional_values[descriptor.positionals.len()]
        )));
    }
    for spec in descriptor.flags {
        if spec.required && !flags.contains_key(spec.name) {
            return Err(syntax_error(format!(
                "missing required flag: {}",
                spec.name
            )));
        }
    }
    for rule in descriptor.rules {
        match rule {
            SyntaxRule::RequiresWhen {
                flag,
                value,
                required_flag,
            } if flags
                .get(flag)
                .is_some_and(|values| values.iter().any(|candidate| candidate == value))
                && !flags.contains_key(required_flag) =>
            {
                return Err(syntax_error(format!(
                    "{required_flag} is required when {flag}={value}"
                )));
            }
            SyntaxRule::ConflictsWhen {
                flag,
                value,
                conflicting_flag,
            } if flags
                .get(flag)
                .is_some_and(|values| values.iter().any(|candidate| candidate == value))
                && flags.contains_key(conflicting_flag) =>
            {
                return Err(syntax_error(format!(
                    "{conflicting_flag} conflicts with {flag}={value}"
                )));
            }
            _ => {}
        }
    }

    Ok(Invocation {
        descriptor,
        positionals,
        flags,
        json,
        compatibility_syntax,
    })
}

fn find_flag(descriptor: &Descriptor, token: &str) -> Option<&'static FlagSpec> {
    descriptor
        .flags
        .iter()
        .find(|spec| spec.name == token || spec.aliases.contains(&token))
}

fn validate_flag_value(spec: &FlagSpec, value: &str) -> Result<(), CliError> {
    if !spec.choices.is_empty() && !spec.choices.contains(&value) {
        return Err(syntax_error(format!(
            "invalid value `{value}` for {}; expected one of: {}",
            spec.name,
            spec.choices.join(", ")
        )));
    }
    if matches!(spec.value_name, Some("N" | "PID")) {
        value
            .parse::<u64>()
            .map_err(|_| syntax_error(format!("{} must be an unsigned integer", spec.name)))?;
    }
    Ok(())
}

fn validate_positional_value(
    spec: &super::registry::PositionalSpec,
    value: &str,
) -> Result<(), CliError> {
    if !spec.choices.is_empty() && !spec.choices.contains(&value) {
        return Err(syntax_error(format!(
            "invalid value `{value}` for {}; expected one of: {}",
            spec.name,
            spec.choices.join(", ")
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn canonical_and_compatibility_paths_resolve_same_operation() {
        let canonical = parse(&strings(&["case", "show", "case:a"])).unwrap();
        let alias = parse(&strings(&["case", "status", "--case", "case:a"])).unwrap();
        let ParseOutcome::Invoke(canonical) = canonical else {
            panic!()
        };
        let ParseOutcome::Invoke(alias) = alias else {
            panic!()
        };
        assert_eq!(
            canonical.descriptor.operation_id,
            alias.descriptor.operation_id
        );
        assert_eq!(canonical.positional("case"), alias.positional("case"));
    }

    #[test]
    fn duplicate_nonrepeatable_flag_is_rejected() {
        let error = parse(&strings(&[
            "case", "create", "case:a", "--tenant", "tenant:a", "--tenant", "tenant:a",
        ]))
        .err()
        .unwrap();
        assert_eq!(error.code, "usage_error");
    }

    #[test]
    fn stable_product_references_do_not_require_generated_ids() {
        for args in [
            vec![
                "provider",
                "qualify",
                "--tenant",
                "tenant:a",
                "--provider-key",
                "cognition",
            ],
            vec![
                "case",
                "provider",
                "bind",
                "case:a",
                "--participant",
                "participant:model",
                "--provider-key",
                "cognition",
            ],
            vec!["case", "memory", "index", "verify", "case:a"],
        ] {
            assert!(
                matches!(parse(&strings(&args)), Ok(ParseOutcome::Invoke(_))),
                "{args:?}"
            );
        }
    }

    #[test]
    fn removed_path_refuses_with_successor() {
        let error = parse(&strings(&["observe", "process", "--pid", "1"]))
            .err()
            .unwrap();
        assert_eq!(error.code, "removed_command");
        assert!(error.remediation.unwrap().contains("process observe"));
    }

    #[test]
    fn parser_negative_contract_is_centralized() {
        for args in [
            vec!["case", "create", "case:a", "--tenant"],
            vec!["process", "signal", "--pid", "x", "--signal", "TERM"],
            vec!["process", "signal", "--pid", "1", "--signal", "NOPE"],
            vec!["case", "show", "case:a", "extra"],
            vec!["case", "show", "case:a", "--unknown"],
            vec!["case", "show", "case:a", "--case", "case:b"],
        ] {
            let error = parse(&strings(&args)).err().expect("syntax must fail");
            assert_eq!(error.code, "usage_error", "{args:?}");
        }
    }

    #[test]
    fn descriptor_requires_and_conflicts_are_enforced() {
        let missing = parse(&strings(&[
            "graph", "rebuild", "--case", "case:a", "--from", "journal",
        ]))
        .err()
        .unwrap();
        assert!(missing.message.contains("--path is required"));
        let conflict = parse(&strings(&[
            "graph",
            "rebuild",
            "--case",
            "case:a",
            "--from",
            "graph-relations",
            "--path",
            "x",
        ]))
        .err()
        .unwrap();
        assert!(conflict.message.contains("conflicts"));
    }

    #[test]
    fn help_is_reachable_for_every_canonical_path() {
        for descriptor in REGISTRY {
            let mut args = descriptor
                .path
                .iter()
                .map(|word| (*word).to_string())
                .collect::<Vec<_>>();
            args.push("--help".to_string());
            assert!(
                matches!(parse(&args), Ok(ParseOutcome::Help(_))),
                "{}",
                descriptor.operation_id
            );
        }
    }

    #[test]
    fn help_rejects_unknown_flags_and_paths() {
        assert!(parse(&strings(&["help", "--unknown"])).is_err());
        assert!(parse(&strings(&["case", "shwo", "--help"])).is_err());
        assert!(matches!(
            parse(&strings(&["case", "show", "case:a", "--help"])),
            Ok(ParseOutcome::Help(_))
        ));
    }
}
