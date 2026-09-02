use serde::Serialize;

pub(crate) const REGISTRY_SCHEMA: &str = "yai.cli.registry.v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ProductSection {
    Start,
    Work,
    Govern,
    Runtime,
    Meta,
}

impl ProductSection {
    pub(crate) const ORDERED: [Self; 5] = [
        Self::Start,
        Self::Work,
        Self::Govern,
        Self::Runtime,
        Self::Meta,
    ];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Start => "START",
            Self::Work => "WORK",
            Self::Govern => "GOVERN",
            Self::Runtime => "RUNTIME",
            Self::Meta => "META",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct ProductRoot {
    pub word: &'static str,
    pub description: &'static str,
    pub section: ProductSection,
}

pub(crate) const PRODUCT_ROOTS: &[ProductRoot] = &[
    ProductRoot {
        word: "init",
        description: "Initialize local identity and Tenant state",
        section: ProductSection::Start,
    },
    ProductRoot {
        word: "doctor",
        description: "Diagnose whether this environment is ready",
        section: ProductSection::Start,
    },
    ProductRoot {
        word: "case",
        description: "Create, govern, run, and inspect Cases",
        section: ProductSection::Work,
    },
    ProductRoot {
        word: "workflow",
        description: "Define and bind deterministic progression",
        section: ProductSection::Work,
    },
    ProductRoot {
        word: "review",
        description: "Resolve authenticated human Reviews",
        section: ProductSection::Work,
    },
    ProductRoot {
        word: "policy",
        description: "Manage policy artifacts and lifecycle",
        section: ProductSection::Govern,
    },
    ProductRoot {
        word: "tenant",
        description: "Inspect Tenant membership and scope",
        section: ProductSection::Govern,
    },
    ProductRoot {
        word: "identity",
        description: "Inspect the authenticated Principal",
        section: ProductSection::Govern,
    },
    ProductRoot {
        word: "provider",
        description: "Govern qualified cognitive provider targets",
        section: ProductSection::Govern,
    },
    ProductRoot {
        word: "runtime",
        description: "Host and control bounded RuntimeInstance work",
        section: ProductSection::Runtime,
    },
    ProductRoot {
        word: "help",
        description: "Show product or advanced command discovery",
        section: ProductSection::Meta,
    },
    ProductRoot {
        word: "version",
        description: "Show binary and CLI registry identity",
        section: ProductSection::Meta,
    },
    ProductRoot {
        word: "completion",
        description: "Generate shell completion from the registry",
        section: ProductSection::Meta,
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Visibility {
    Product,
    Advanced,
    Plumbing,
    Compatibility,
    Removed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Lane {
    LocalDomain,
    RuntimeHost,
    RuntimeControl,
    Inspection,
    Compatibility,
    LocalInteractive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Mutation {
    ReadOnly,
    Mutating,
    LongRunning,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OutputCapability {
    Structured,
    PlainCompat,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct PositionalSpec {
    pub name: &'static str,
    pub required: bool,
    pub legacy_flag: Option<&'static str>,
    pub choices: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct FlagSpec {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub value_name: Option<&'static str>,
    pub required: bool,
    pub repeatable: bool,
    pub choices: &'static [&'static str],
    pub legacy_name: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum SyntaxRule {
    RequiresWhen {
        flag: &'static str,
        value: &'static str,
        required_flag: &'static str,
    },
    ConflictsWhen {
        flag: &'static str,
        value: &'static str,
        conflicting_flag: &'static str,
    },
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct Descriptor {
    pub operation_id: &'static str,
    pub handler_id: &'static str,
    pub path: &'static [&'static str],
    pub description: &'static str,
    pub visibility: Visibility,
    pub lane: Lane,
    pub mutation: Mutation,
    pub output: OutputCapability,
    pub positionals: &'static [PositionalSpec],
    pub flags: &'static [FlagSpec],
    pub rules: &'static [SyntaxRule],
    pub aliases: &'static [&'static [&'static str]],
    pub legacy_path: &'static [&'static str],
    pub removed_successor: Option<&'static [&'static str]>,
}

const NO_POS: &[PositionalSpec] = &[];
const NO_FLAGS: &[FlagSpec] = &[];
const NO_ALIASES: &[&[&str]] = &[];
const EMPTY: &[&str] = &[];

const fn pos(name: &'static str, legacy_flag: Option<&'static str>) -> PositionalSpec {
    PositionalSpec {
        name,
        required: true,
        legacy_flag,
        choices: EMPTY,
    }
}

const fn choice_pos(name: &'static str, choices: &'static [&'static str]) -> PositionalSpec {
    PositionalSpec {
        name,
        required: true,
        legacy_flag: None,
        choices,
    }
}

const fn flag(name: &'static str, value_name: Option<&'static str>, required: bool) -> FlagSpec {
    FlagSpec {
        name,
        aliases: EMPTY,
        value_name,
        required,
        repeatable: false,
        choices: EMPTY,
        legacy_name: None,
    }
}

const fn bool_flag(name: &'static str) -> FlagSpec {
    flag(name, None, false)
}

const fn aliased_flag(
    name: &'static str,
    aliases: &'static [&'static str],
    value_name: Option<&'static str>,
    required: bool,
    legacy_name: &'static str,
) -> FlagSpec {
    FlagSpec {
        name,
        aliases,
        value_name,
        required,
        repeatable: false,
        choices: EMPTY,
        legacy_name: Some(legacy_name),
    }
}

const fn choice_flag(
    name: &'static str,
    choices: &'static [&'static str],
    required: bool,
) -> FlagSpec {
    FlagSpec {
        name,
        aliases: EMPTY,
        value_name: Some("VALUE"),
        required,
        repeatable: false,
        choices,
        legacy_name: None,
    }
}

const fn repeat_flag(name: &'static str, value_name: &'static str) -> FlagSpec {
    FlagSpec {
        name,
        aliases: EMPTY,
        value_name: Some(value_name),
        required: false,
        repeatable: true,
        choices: EMPTY,
        legacy_name: None,
    }
}

const fn required_repeat_flag(name: &'static str, value_name: &'static str) -> FlagSpec {
    FlagSpec {
        name,
        aliases: EMPTY,
        value_name: Some(value_name),
        required: true,
        repeatable: true,
        choices: EMPTY,
        legacy_name: None,
    }
}

macro_rules! op {
    ($id:literal, [$($path:literal),+], $description:literal, $visibility:ident, $lane:ident, $mutation:ident, $output:ident, $pos:expr, $flags:expr) => {
        Descriptor {
            operation_id: $id,
            handler_id: $id,
            path: &[$($path),+],
            description: $description,
            visibility: Visibility::$visibility,
            lane: Lane::$lane,
            mutation: Mutation::$mutation,
            output: OutputCapability::$output,
            positionals: $pos,
            flags: $flags,
            rules: &[],
            aliases: NO_ALIASES,
            legacy_path: &[$($path),+],
            removed_successor: None,
        }
    };
}

const SUBJECT_ALIAS: &[&str] = &["--subject"];
const ENDPOINT_ALIAS: &[&str] = &["--base-url"];
const PROVIDER_ALIAS: &[&str] = &["--provider-id"];
const RESOURCE_ALIAS: &[&str] = &["--attachment"];

const TENANT: &[FlagSpec] = &[flag("--tenant", Some("TENANT"), true)];
const CASE_TENANT: &[FlagSpec] = &[flag("--tenant", Some("TENANT"), true)];
const INIT_FLAGS: &[FlagSpec] = &[
    flag("--tenant", Some("TENANT"), true),
    flag("--organization", Some("ORGANIZATION"), true),
];
const CASE_RUN_FLAGS: &[FlagSpec] = &[
    aliased_flag(
        "--participant",
        SUBJECT_ALIAS,
        Some("PARTICIPANT"),
        true,
        "--subject",
    ),
    aliased_flag(
        "--resource",
        RESOURCE_ALIAS,
        Some("RESOURCE"),
        true,
        "--attachment",
    ),
    flag("--prompt", Some("TASK"), true),
    flag("--max-invocations", Some("N"), false),
    flag("--max-operations", Some("N"), false),
    flag("--max-semantic-units", Some("N"), false),
    flag("--max-estimated-input-units", Some("N"), false),
    flag("--max-resident-items", Some("N"), false),
    flag("--max-provider-retries", Some("N"), false),
    flag("--max-runtime-ms", Some("N"), false),
    bool_flag("--stop-on-deny"),
    bool_flag("--continue-after-malformed"),
    flag("--failpoint", Some("FAILPOINT"), false),
];
const CASE_RESUME_FLAGS: &[FlagSpec] = &[
    flag("--max-invocations", Some("N"), false),
    flag("--max-operations", Some("N"), false),
    flag("--max-semantic-units", Some("N"), false),
    flag("--max-estimated-input-units", Some("N"), false),
    flag("--max-resident-items", Some("N"), false),
    flag("--max-provider-retries", Some("N"), false),
    flag("--max-runtime-ms", Some("N"), false),
    bool_flag("--stop-on-deny"),
    bool_flag("--continue-after-malformed"),
    flag("--failpoint", Some("FAILPOINT"), false),
];
const REASON_WITH_SPOOF_PROBE: &[FlagSpec] = &[
    flag("--reason", Some("REASON"), true),
    flag("--as", Some("PRINCIPAL"), false),
];
const PARTICIPANT_ROLE: &[FlagSpec] = &[
    flag("--participant", Some("PARTICIPANT"), true),
    flag("--role", Some("ROLE"), true),
];
const PRINCIPAL_LINK: &[FlagSpec] = &[
    flag("--principal", Some("PRINCIPAL"), true),
    flag("--participant", Some("PARTICIPANT"), true),
];
const PROVIDER_ATTACH: &[FlagSpec] = &[
    aliased_flag(
        "--participant",
        SUBJECT_ALIAS,
        Some("PARTICIPANT"),
        true,
        "--subject",
    ),
    aliased_flag(
        "--endpoint",
        ENDPOINT_ALIAS,
        Some("URL"),
        true,
        "--base-url",
    ),
    flag("--model", Some("MODEL"), true),
    aliased_flag(
        "--provider",
        PROVIDER_ALIAS,
        Some("PROVIDER"),
        false,
        "--provider-id",
    ),
    flag("--api-key-env", Some("ENV"), false),
    flag("--shell", Some("SHELL"), false),
    flag("--provider-runtime-id", Some("ID"), false),
    flag("--continuation-ref", Some("REF"), false),
    bool_flag("--continuation-capable"),
];
const PROVIDER_ADD: &[FlagSpec] = &[
    flag("--tenant", Some("TENANT"), true),
    flag("--provider-key", Some("KEY"), true),
    flag("--endpoint", Some("URL"), true),
    flag("--model", Some("MODEL"), true),
    flag("--credential-ref", Some("REF"), false),
    choice_flag(
        "--locality",
        &["loopback", "private_network", "remote"],
        true,
    ),
    flag("--extension-adapter", Some("ADAPTER"), false),
];
const PROVIDER_QUALIFY: &[FlagSpec] = &[flag("--valid-for-ms", Some("MS"), false)];
const CASE_PROVIDER_BIND: &[FlagSpec] = &[
    flag("--participant", Some("PARTICIPANT"), true),
    required_repeat_flag("--target", "TARGET"),
    choice_flag("--failover", &["none", "safe_only"], false),
    flag("--max-attempts", Some("N"), false),
];
const FILESYSTEM_ATTACH: &[FlagSpec] = &[
    aliased_flag(
        "--resource",
        RESOURCE_ALIAS,
        Some("RESOURCE"),
        true,
        "--attachment",
    ),
    flag("--root", Some("DIR"), true),
    flag("--allow-prefix", Some("RELATIVE_DIR"), true),
    flag("--policy-owner", Some("PARTICIPANT"), true),
    bool_flag("--require-review"),
    flag("--policy-id", Some("POLICY"), false),
    flag("--max-bytes", Some("N"), false),
];
const PROCESS_ATTACH: &[FlagSpec] = &[
    aliased_flag(
        "--resource",
        RESOURCE_ALIAS,
        Some("RESOURCE"),
        true,
        "--attachment",
    ),
    flag("--pid", Some("PID"), true),
    flag("--policy-owner", Some("PARTICIPANT"), true),
    flag("--actions", Some("ACTIONS"), false),
    bool_flag("--require-review"),
    flag("--policy-id", Some("POLICY"), false),
];
const CASE_POLICY_BIND: &[FlagSpec] = &[
    flag("--artifact", Some("POLICY"), true),
    flag("--reason", Some("REASON"), false),
    flag("--expected-generation", Some("N"), false),
    flag("--as", Some("PRINCIPAL"), false),
];
const CASE_POLICY_REPLACE: &[FlagSpec] = &[
    flag("--binding", Some("BINDING"), true),
    flag("--artifact", Some("POLICY"), true),
    flag("--reason", Some("REASON"), false),
    flag("--expected-generation", Some("N"), false),
    flag("--as", Some("PRINCIPAL"), false),
];
const CASE_POLICY_UNBIND: &[FlagSpec] = &[
    flag("--binding", Some("BINDING"), true),
    flag("--reason", Some("REASON"), true),
    flag("--expected-generation", Some("N"), false),
    flag("--as", Some("PRINCIPAL"), false),
];
const WORKFLOW_DEFINE: &[FlagSpec] = &[
    flag("--tenant", Some("TENANT"), true),
    flag("--file", Some("FILE"), true),
];
const WORKFLOW_BIND: &[FlagSpec] = &[
    flag("--definition", Some("DEFINITION"), true),
    repeat_flag("--executor", "SLOT=PARTICIPANT"),
    repeat_flag("--resource", "SLOT=RESOURCE"),
    repeat_flag("--case-slot", "SLOT=CASE"),
];
const WORKFLOW_INPUT: &[FlagSpec] = &[
    flag("--node", Some("NODE"), true),
    flag("--value", Some("VALUE"), true),
];
const WORKFLOW_PATCH_PROPOSE: &[FlagSpec] = &[flag("--file", Some("FILE"), true)];
const HANDOFF_OFFER: &[FlagSpec] = &[
    flag("--target", Some("CASE"), true),
    flag("--value", Some("VALUE"), true),
    choice_flag("--kind", &["text", "json"], false),
    repeat_flag("--role", "ROLE"),
];
const HANDOFF_ACCEPT: &[FlagSpec] = &[
    flag("--source", Some("CASE"), true),
    flag("--handoff", Some("HANDOFF"), true),
    flag("--participant", Some("PARTICIPANT"), true),
];
const HANDOFF_DECLINE: &[FlagSpec] = &[
    flag("--source", Some("CASE"), true),
    flag("--handoff", Some("HANDOFF"), true),
    flag("--participant", Some("PARTICIPANT"), true),
    flag("--reason", Some("REASON"), true),
];
const HANDOFF_RESULT: &[FlagSpec] = &[
    flag("--handoff", Some("HANDOFF"), true),
    flag("--participant", Some("PARTICIPANT"), true),
    choice_flag("--outcome", &["succeeded", "failed", "cancelled"], true),
    flag("--value", Some("VALUE"), true),
    choice_flag("--kind", &["text", "json"], false),
    repeat_flag("--evidence", "REF"),
];
const HANDOFF_ID: &[FlagSpec] = &[flag("--handoff", Some("HANDOFF"), true)];
const WORKFLOW_PATCH_ID: &[FlagSpec] = &[flag("--patch", Some("PATCH"), true)];
const REVIEW_CASE: &[FlagSpec] = &[flag("--case", Some("CASE"), true)];
const REVIEW_RESOLVE: &[FlagSpec] = &[
    flag("--case", Some("CASE"), true),
    flag("--reason", Some("REASON"), true),
    flag("--as", Some("PRINCIPAL"), false),
    flag("--failpoint", Some("FAILPOINT"), false),
];
const POLICY_INGEST: &[FlagSpec] = &[
    flag("--tenant", Some("TENANT"), true),
    flag("--as", Some("PRINCIPAL"), false),
];
const POLICY_REASON: &[FlagSpec] = &[
    flag("--reason", Some("REASON"), false),
    flag("--as", Some("PRINCIPAL"), false),
];
const POLICY_REQUIRED_REASON: &[FlagSpec] = &[
    flag("--reason", Some("REASON"), true),
    flag("--as", Some("PRINCIPAL"), false),
];
const RUNTIME_SERVE: &[FlagSpec] = &[
    flag("--workers", Some("N"), false),
    flag("--max-active-per-tenant", Some("N"), false),
    flag("--max-queued-per-tenant", Some("N"), false),
    flag("--max-queued-total", Some("N"), false),
    flag("--startup-dispatch-delay-ms", Some("N"), false),
    flag("--workflow-work-failpoint", Some("FAILPOINT"), false),
    flag("--failpoint", Some("FAILPOINT"), false),
];
const RUNTIME_QUEUE: &[FlagSpec] = &[flag("--all", None, false)];

pub(crate) static REGISTRY: &[Descriptor] = &[
    op!(
        "yai.meta.help",
        ["help"],
        "Discover commands from the compiled registry",
        Product,
        Inspection,
        ReadOnly,
        Structured,
        NO_POS,
        &[bool_flag("--advanced")]
    ),
    op!(
        "yai.meta.version",
        ["version"],
        "Show binary and interface versions",
        Product,
        Inspection,
        ReadOnly,
        Structured,
        NO_POS,
        NO_FLAGS
    ),
    op!(
        "yai.meta.completion",
        ["completion"],
        "Generate shell completion from the command registry",
        Product,
        Inspection,
        ReadOnly,
        Structured,
        &[choice_pos("shell", &["bash", "zsh", "fish"])],
        NO_FLAGS
    ),
    Descriptor {
        legacy_path: &["security", "bootstrap-local"],
        ..op!(
            "yai.init",
            ["init"],
            "Initialize local YAI identity and Tenant state",
            Product,
            LocalDomain,
            Mutating,
            Structured,
            NO_POS,
            INIT_FLAGS
        )
    },
    op!(
        "yai.doctor",
        ["doctor"],
        "Diagnose local YAI readiness",
        Product,
        Inspection,
        ReadOnly,
        Structured,
        NO_POS,
        NO_FLAGS
    ),
    op!(
        "yai.identity.whoami",
        ["identity", "whoami"],
        "Show the authenticated Principal and Tenant memberships",
        Product,
        LocalDomain,
        ReadOnly,
        Structured,
        NO_POS,
        NO_FLAGS
    ),
    op!(
        "yai.tenant.list",
        ["tenant", "list"],
        "List visible Tenants",
        Product,
        LocalDomain,
        ReadOnly,
        Structured,
        NO_POS,
        NO_FLAGS
    ),
    Descriptor {
        aliases: &[&["tenant", "status"]],
        legacy_path: &["tenant", "status"],
        ..op!(
            "yai.tenant.show",
            ["tenant", "show"],
            "Show one Tenant",
            Product,
            LocalDomain,
            ReadOnly,
            Structured,
            &[pos("tenant", Some("--tenant"))],
            NO_FLAGS
        )
    },
    Descriptor {
        legacy_path: &["tenant", "add-member"],
        ..op!(
            "yai.tenant.member.add",
            ["tenant", "member", "add"],
            "Add a Principal to a Tenant",
            Product,
            LocalDomain,
            Mutating,
            Structured,
            &[pos("tenant", Some("--tenant"))],
            &[flag("--principal", Some("PRINCIPAL"), true)]
        )
    },
    op!(
        "yai.provider.add",
        ["provider", "add"],
        "Register an immutable Tenant provider target",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        NO_POS,
        PROVIDER_ADD
    ),
    op!(
        "yai.provider.list",
        ["provider", "list"],
        "List Tenant provider targets",
        Product,
        LocalDomain,
        ReadOnly,
        Structured,
        NO_POS,
        TENANT
    ),
    op!(
        "yai.provider.show",
        ["provider", "show"],
        "Show configuration, qualification, governance, and health separately",
        Product,
        LocalDomain,
        ReadOnly,
        Structured,
        &[pos("target", Some("--target"))],
        NO_FLAGS
    ),
    op!(
        "yai.provider.probe",
        ["provider", "probe"],
        "Run a bounded synthetic provider health probe",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("target", Some("--target"))],
        NO_FLAGS
    ),
    op!(
        "yai.provider.qualify",
        ["provider", "qualify"],
        "Qualify substrate capabilities from synthetic evidence",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("target", Some("--target"))],
        PROVIDER_QUALIFY
    ),
    op!(
        "yai.provider.trust.approve",
        ["provider", "trust", "approve"],
        "Approve a provider target for future Tenant selections",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("target", Some("--target"))],
        NO_FLAGS
    ),
    op!(
        "yai.provider.trust.deny",
        ["provider", "trust", "deny"],
        "Deny a provider target for future Tenant selections",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("target", Some("--target"))],
        NO_FLAGS
    ),
    op!(
        "yai.case.create",
        ["case", "create"],
        "Create a canonical Case",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("case", Some("--case"))],
        CASE_TENANT
    ),
    op!(
        "yai.case.list",
        ["case", "list"],
        "List visible Cases",
        Product,
        LocalDomain,
        ReadOnly,
        Structured,
        NO_POS,
        &[flag("--tenant", Some("TENANT"), false)]
    ),
    Descriptor {
        aliases: &[&["case", "status"]],
        legacy_path: &["case", "status"],
        ..op!(
            "yai.case.show",
            ["case", "show"],
            "Show canonical Case truth with derived operational overlays",
            Product,
            LocalDomain,
            ReadOnly,
            Structured,
            &[pos("case", Some("--case"))],
            NO_FLAGS
        )
    },
    op!(
        "yai.case.run",
        ["case", "run"],
        "Run bounded governed Case work",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("case", Some("--case"))],
        CASE_RUN_FLAGS
    ),
    op!(
        "yai.case.resume",
        ["case", "resume"],
        "Resume the same bounded Case execution",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("case", Some("--case"))],
        CASE_RESUME_FLAGS
    ),
    op!(
        "yai.case.stop",
        ["case", "stop"],
        "Stop active Case execution without cancelling the Case",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("case", Some("--case"))],
        NO_FLAGS
    ),
    op!(
        "yai.case.cancel",
        ["case", "cancel"],
        "Cancel further Case advancement",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("case", Some("--case"))],
        REASON_WITH_SPOOF_PROBE
    ),
    op!(
        "yai.case.close",
        ["case", "close"],
        "Close a Case non-destructively",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("case", Some("--case"))],
        REASON_WITH_SPOOF_PROBE
    ),
    Descriptor {
        aliases: &[&["case", "bind-participant-role"]],
        legacy_path: &["case", "bind-participant-role"],
        ..op!(
            "yai.case.participant.role.add",
            ["case", "participant", "role", "add"],
            "Bind an existing Participant to a Case role",
            Product,
            LocalDomain,
            Mutating,
            Structured,
            &[pos("case", Some("--case"))],
            PARTICIPANT_ROLE
        )
    },
    Descriptor {
        aliases: &[&["case", "principal", "link"]],
        legacy_path: &["case", "principal", "link"],
        ..op!(
            "yai.case.participant.link_principal",
            ["case", "participant", "link-principal"],
            "Link an authenticated Principal to a Case Participant",
            Product,
            LocalDomain,
            Mutating,
            Structured,
            &[pos("case", Some("--case"))],
            PRINCIPAL_LINK
        )
    },
    op!(
        "yai.case.participant.list",
        ["case", "participant", "list"],
        "List Case Participants and roles",
        Product,
        LocalDomain,
        ReadOnly,
        Structured,
        &[pos("case", Some("--case"))],
        NO_FLAGS
    ),
    Descriptor {
        aliases: &[&["case", "attach-provider"]],
        legacy_path: &["case", "attach-provider"],
        ..op!(
            "yai.case.provider.attach",
            ["case", "provider", "attach"],
            "Attach an endpoint/model to an exact Case Participant",
            Product,
            LocalDomain,
            Mutating,
            Structured,
            &[pos("case", Some("--case"))],
            PROVIDER_ATTACH
        )
    },
    op!(
        "yai.case.provider.bind",
        ["case", "provider", "bind"],
        "Bind an ordered governed provider target pool to a Case Participant",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("case", Some("--case"))],
        CASE_PROVIDER_BIND
    ),
    op!(
        "yai.case.provider.show",
        ["case", "provider", "show"],
        "Show the exact legacy pin or governed Case provider binding",
        Product,
        LocalDomain,
        ReadOnly,
        Structured,
        &[pos("case", Some("--case"))],
        NO_FLAGS
    ),
    Descriptor {
        aliases: &[&["case", "attach-filesystem"]],
        legacy_path: &["case", "attach-filesystem"],
        ..op!(
            "yai.case.resource.attach_filesystem",
            ["case", "resource", "attach", "filesystem"],
            "Attach a fenced filesystem Resource",
            Product,
            LocalDomain,
            Mutating,
            Structured,
            &[pos("case", Some("--case"))],
            FILESYSTEM_ATTACH
        )
    },
    Descriptor {
        aliases: &[&["case", "attach-process"]],
        legacy_path: &["case", "attach-process"],
        ..op!(
            "yai.case.resource.attach_process",
            ["case", "resource", "attach", "process"],
            "Attach an identity-bound process Resource",
            Product,
            LocalDomain,
            Mutating,
            Structured,
            &[pos("case", Some("--case"))],
            PROCESS_ATTACH
        )
    },
    op!(
        "yai.case.resource.list",
        ["case", "resource", "list"],
        "List logical Case Resources",
        Product,
        LocalDomain,
        ReadOnly,
        Structured,
        &[pos("case", Some("--case"))],
        NO_FLAGS
    ),
    op!(
        "yai.case.policy.bind",
        ["case", "policy", "bind"],
        "Bind an exact policy artifact to a Case",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("case", Some("--case"))],
        CASE_POLICY_BIND
    ),
    op!(
        "yai.case.policy.replace",
        ["case", "policy", "replace"],
        "Replace a Case policy binding with optimistic concurrency",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("case", Some("--case"))],
        CASE_POLICY_REPLACE
    ),
    op!(
        "yai.case.policy.unbind",
        ["case", "policy", "unbind"],
        "Remove a Case policy binding with optimistic concurrency",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("case", Some("--case"))],
        CASE_POLICY_UNBIND
    ),
    Descriptor {
        legacy_path: &["case", "policy", "status"],
        aliases: &[&["case", "policy", "status"]],
        ..op!(
            "yai.case.policy.show",
            ["case", "policy", "show"],
            "Show effective Case policy materialization",
            Product,
            LocalDomain,
            ReadOnly,
            Structured,
            &[pos("case", Some("--case"))],
            NO_FLAGS
        )
    },
    op!(
        "yai.case.policy.rebuild",
        ["case", "policy", "rebuild"],
        "Rebuild derived Case policy materialization",
        Advanced,
        LocalDomain,
        Mutating,
        PlainCompat,
        NO_POS,
        &[flag("--case", Some("CASE"), true)]
    ),
    op!(
        "yai.case.handoff.offer",
        ["case", "handoff", "offer"],
        "Offer bounded work information to a same-Tenant Case",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("case", Some("--case"))],
        HANDOFF_OFFER
    ),
    op!(
        "yai.case.handoff.pending",
        ["case", "handoff", "pending"],
        "List Handoff offers addressed to a Case",
        Product,
        LocalDomain,
        ReadOnly,
        Structured,
        &[pos("case", Some("--case"))],
        NO_FLAGS
    ),
    op!(
        "yai.case.handoff.show",
        ["case", "handoff", "show"],
        "Show one visible Handoff protocol posture",
        Product,
        LocalDomain,
        ReadOnly,
        Structured,
        &[pos("case", Some("--case"))],
        HANDOFF_ID
    ),
    op!(
        "yai.case.handoff.accept",
        ["case", "handoff", "accept"],
        "Accept a Handoff using an eligible target Participant",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("case", Some("--case"))],
        HANDOFF_ACCEPT
    ),
    op!(
        "yai.case.handoff.decline",
        ["case", "handoff", "decline"],
        "Decline a Handoff without granting authority",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("case", Some("--case"))],
        HANDOFF_DECLINE
    ),
    op!(
        "yai.case.handoff.result",
        ["case", "handoff", "result"],
        "Record one bounded terminal target result",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("case", Some("--case"))],
        HANDOFF_RESULT
    ),
    op!(
        "yai.case.handoff.reconcile",
        ["case", "handoff", "reconcile"],
        "Reconcile a target disposition into source-local truth",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("case", Some("--case"))],
        HANDOFF_ID
    ),
    op!(
        "yai.workflow.define",
        ["workflow", "define"],
        "Admit an immutable WorkflowDefinition",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        NO_POS,
        WORKFLOW_DEFINE
    ),
    op!(
        "yai.workflow.list",
        ["workflow", "list"],
        "List WorkflowDefinitions",
        Product,
        LocalDomain,
        ReadOnly,
        Structured,
        NO_POS,
        TENANT
    ),
    op!(
        "yai.workflow.show",
        ["workflow", "show"],
        "Show one immutable WorkflowDefinition",
        Product,
        LocalDomain,
        ReadOnly,
        Structured,
        &[pos("definition", None)],
        NO_FLAGS
    ),
    op!(
        "yai.workflow.bind",
        ["workflow", "bind"],
        "Bind one exact WorkflowDefinition to a Case",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("case", Some("--case"))],
        WORKFLOW_BIND
    ),
    op!(
        "yai.workflow.status",
        ["workflow", "status"],
        "Resolve Workflow progression without mutation",
        Product,
        LocalDomain,
        ReadOnly,
        Structured,
        &[pos("case", Some("--case"))],
        NO_FLAGS
    ),
    op!(
        "yai.workflow.input",
        ["workflow", "input"],
        "Record bounded HumanInput for a ready node",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("case", Some("--case"))],
        WORKFLOW_INPUT
    ),
    op!(
        "yai.workflow.patch.propose",
        ["workflow", "patch", "propose"],
        "Propose a bounded Case-local PlanPatch",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("case", Some("--case"))],
        WORKFLOW_PATCH_PROPOSE
    ),
    op!(
        "yai.workflow.patch.list",
        ["workflow", "patch", "list"],
        "List Case-local PlanPatch candidates",
        Product,
        LocalDomain,
        ReadOnly,
        Structured,
        &[pos("case", Some("--case"))],
        NO_FLAGS
    ),
    op!(
        "yai.workflow.patch.propose_model",
        ["workflow", "patch", "propose-model"],
        "Parse a strict PlanPatch candidate from one exact ModelWork result",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("case", Some("--case"))],
        &[flag("--provider-result", Some("PROVIDER_RESULT"), true)]
    ),
    op!(
        "yai.workflow.patch.show",
        ["workflow", "patch", "show"],
        "Show one Case-local PlanPatch candidate",
        Product,
        LocalDomain,
        ReadOnly,
        Structured,
        &[pos("case", Some("--case"))],
        WORKFLOW_PATCH_ID
    ),
    op!(
        "yai.workflow.patch.validate",
        ["workflow", "patch", "validate"],
        "Validate a PlanPatch against current effective topology",
        Product,
        LocalDomain,
        ReadOnly,
        Structured,
        &[pos("case", Some("--case"))],
        WORKFLOW_PATCH_ID
    ),
    op!(
        "yai.workflow.patch.adopt",
        ["workflow", "patch", "adopt"],
        "Adopt a valid PlanPatch as Tenant Owner",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("case", Some("--case"))],
        WORKFLOW_PATCH_ID
    ),
    op!(
        "yai.review.pending",
        ["review", "pending"],
        "List pending Reviews for a Case",
        Product,
        LocalDomain,
        ReadOnly,
        Structured,
        NO_POS,
        REVIEW_CASE
    ),
    op!(
        "yai.review.show",
        ["review", "show"],
        "Show one Review",
        Product,
        LocalDomain,
        ReadOnly,
        Structured,
        &[pos("review", None)],
        REVIEW_CASE
    ),
    op!(
        "yai.review.approve",
        ["review", "approve"],
        "Approve a Review through authenticated authority",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("review", None)],
        REVIEW_RESOLVE
    ),
    op!(
        "yai.review.deny",
        ["review", "deny"],
        "Deny a Review through authenticated authority",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("review", None)],
        REVIEW_RESOLVE
    ),
    op!(
        "yai.review.defer",
        ["review", "defer"],
        "Defer a Review without approving it",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("review", None)],
        REVIEW_RESOLVE
    ),
    op!(
        "yai.policy.ingest",
        ["policy", "ingest"],
        "Ingest immutable policy source",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("source", None)],
        POLICY_INGEST
    ),
    Descriptor {
        aliases: &[&["policy", "inspect"]],
        legacy_path: &["policy", "inspect"],
        ..op!(
            "yai.policy.show",
            ["policy", "show"],
            "Show a policy artifact or Tenant source",
            Product,
            LocalDomain,
            ReadOnly,
            Structured,
            &[pos("policy", None)],
            &[flag("--tenant", Some("TENANT"), false)]
        )
    },
    op!(
        "yai.policy.validate",
        ["policy", "validate"],
        "Validate a policy artifact",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("policy", None)],
        POLICY_REASON
    ),
    op!(
        "yai.policy.publish",
        ["policy", "publish"],
        "Publish a policy artifact",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("policy", None)],
        POLICY_REASON
    ),
    op!(
        "yai.policy.retire",
        ["policy", "retire"],
        "Retire a policy artifact",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("policy", None)],
        POLICY_REQUIRED_REASON
    ),
    op!(
        "yai.policy.revoke",
        ["policy", "revoke"],
        "Revoke a policy artifact",
        Product,
        LocalDomain,
        Mutating,
        Structured,
        &[pos("policy", None)],
        POLICY_REQUIRED_REASON
    ),
    op!(
        "yai.policy.list",
        ["policy", "list"],
        "List policy artifacts for a Tenant",
        Product,
        LocalDomain,
        ReadOnly,
        Structured,
        NO_POS,
        TENANT
    ),
    op!(
        "yai.runtime.serve",
        ["runtime", "serve"],
        "Host the bounded RuntimeInstance",
        Product,
        RuntimeHost,
        LongRunning,
        Structured,
        NO_POS,
        RUNTIME_SERVE
    ),
    op!(
        "yai.runtime.status",
        ["runtime", "status"],
        "Inspect RuntimeInstance state",
        Product,
        RuntimeControl,
        ReadOnly,
        Structured,
        NO_POS,
        NO_FLAGS
    ),
    op!(
        "yai.runtime.queue",
        ["runtime", "queue"],
        "Inspect the Tenant-fair runtime queue",
        Product,
        RuntimeControl,
        ReadOnly,
        Structured,
        NO_POS,
        RUNTIME_QUEUE
    ),
    op!(
        "yai.runtime.stop",
        ["runtime", "stop"],
        "Request RuntimeInstance shutdown",
        Product,
        RuntimeControl,
        Mutating,
        Structured,
        NO_POS,
        NO_FLAGS
    ),
    op!(
        "yai.security.bootstrap_local",
        ["security", "bootstrap-local"],
        "Bootstrap exact local security state",
        Advanced,
        LocalDomain,
        Mutating,
        PlainCompat,
        NO_POS,
        INIT_FLAGS
    ),
    op!(
        "yai.case.enter",
        ["case", "enter"],
        "Inspect a Participant projection or enter developer prompt mode",
        Advanced,
        LocalInteractive,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[
            flag("--case", Some("CASE"), true),
            aliased_flag(
                "--participant",
                SUBJECT_ALIAS,
                Some("PARTICIPANT"),
                true,
                "--subject"
            ),
            choice_flag(
                "--consumer",
                &["model", "operator", "audit", "debug", "agent"],
                false
            ),
            flag("--kind", Some("KIND"), false),
            flag("--shell", Some("SHELL"), false)
        ]
    ),
    op!(
        "yai.runtime.submit",
        ["runtime", "submit"],
        "Submit an exact RuntimeWorkItem",
        Advanced,
        RuntimeControl,
        Mutating,
        PlainCompat,
        NO_POS,
        &[
            flag("--tenant", Some("TENANT"), true),
            flag("--case", Some("CASE"), true),
            flag("--subject", Some("PARTICIPANT"), true),
            flag("--attachment", Some("RESOURCE"), true),
            flag("--prompt", Some("TASK"), true),
            flag("--idempotency-key", Some("KEY"), false),
            flag("--max-invocations", Some("N"), false),
            flag("--max-operations", Some("N"), false),
            flag("--max-semantic-units", Some("N"), false),
            flag("--max-resident-items", Some("N"), false),
            flag("--max-estimated-input-units", Some("N"), false),
            flag("--max-provider-retries", Some("N"), false),
            flag("--max-runtime-ms", Some("N"), false),
            bool_flag("--stop-on-deny"),
            bool_flag("--continue-after-malformed"),
            flag("--failpoint", Some("FAILPOINT"), false)
        ]
    ),
    op!(
        "yai.effect.filesystem_write",
        ["effect", "filesystem-write"],
        "Exercise the governed filesystem effect path directly",
        Advanced,
        LocalDomain,
        Mutating,
        PlainCompat,
        NO_POS,
        &[
            flag("--case", Some("CASE"), true),
            flag("--subject", Some("PARTICIPANT"), true),
            flag("--attachment", Some("RESOURCE"), true),
            flag("--prompt", Some("TASK"), true),
            flag("--base-url", Some("URL"), true),
            flag("--model", Some("MODEL"), true),
            flag("--provider-id", Some("PROVIDER"), false),
            flag("--api-key-env", Some("ENV"), false),
            flag("--provider-runtime-id", Some("ID"), false),
            flag("--continuation-ref", Some("REF"), false),
            bool_flag("--continuation-capable"),
            flag("--second-base-url", Some("URL"), false),
            flag("--second-provider-id", Some("PROVIDER"), false),
            flag("--second-model", Some("MODEL"), false),
            bool_flag("--inject-derived-failure"),
            flag("--failpoint", Some("FAILPOINT"), false)
        ]
    ),
    op!(
        "yai.effect.process_signal",
        ["effect", "process-signal"],
        "Exercise the governed process effect path directly",
        Advanced,
        LocalDomain,
        Mutating,
        PlainCompat,
        NO_POS,
        &[
            flag("--case", Some("CASE"), true),
            flag("--subject", Some("PARTICIPANT"), true),
            flag("--attachment", Some("RESOURCE"), true),
            flag("--prompt", Some("TASK"), true),
            flag("--base-url", Some("URL"), true),
            flag("--model", Some("MODEL"), true),
            flag("--provider-id", Some("PROVIDER"), false),
            flag("--api-key-env", Some("ENV"), false),
            flag("--provider-runtime-id", Some("ID"), false),
            flag("--continuation-ref", Some("REF"), false),
            bool_flag("--continuation-capable"),
            bool_flag("--inject-derived-failure"),
            flag("--failpoint", Some("FAILPOINT"), false)
        ]
    ),
    op!(
        "yai.effect.reconcile",
        ["effect", "reconcile"],
        "Reconcile unresolved physical effect truth",
        Advanced,
        LocalDomain,
        Mutating,
        PlainCompat,
        NO_POS,
        &[
            flag("--case", Some("CASE"), true),
            flag("--effect", Some("EFFECT"), false),
            bool_flag("--retry")
        ]
    ),
    op!(
        "yai.effect.inspect",
        ["effect", "inspect"],
        "Inspect an exact governed Effect",
        Advanced,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[
            flag("--case", Some("CASE"), true),
            flag("--effect", Some("EFFECT"), true)
        ]
    ),
    op!(
        "yai.prompt",
        ["prompt"],
        "Run the advanced local Case prompt surface",
        Advanced,
        LocalInteractive,
        Mutating,
        PlainCompat,
        NO_POS,
        &[
            flag("--once", Some("TEXT"), false),
            bool_flag("--dry-run"),
            choice_flag("--language-mode", &["auto", "none"], false),
            flag("--case", Some("CASE"), false),
            flag("--subject", Some("PARTICIPANT"), false),
            flag("--provider-id", Some("PROVIDER"), false),
            flag("--base-url", Some("URL"), false),
            flag("--model", Some("MODEL"), false),
            flag("--api-key-env", Some("ENV"), false),
            bool_flag("--continuation-capable"),
            flag("--provider-runtime-id", Some("ID"), false),
            flag("--continuation-ref", Some("REF"), false)
        ]
    ),
    op!(
        "yai.info",
        ["info"],
        "Show compatibility build information",
        Compatibility,
        Compatibility,
        ReadOnly,
        PlainCompat,
        NO_POS,
        NO_FLAGS
    ),
    op!(
        "yai.store.status",
        ["store", "status"],
        "Inspect canonical record-store posture",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        NO_FLAGS
    ),
    op!(
        "yai.store.summary",
        ["store", "summary"],
        "Summarize canonical records",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        NO_FLAGS
    ),
    op!(
        "yai.store.record.get",
        ["store", "record", "get"],
        "Read one canonical record",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        &[pos("record", None)],
        NO_FLAGS
    ),
    op!(
        "yai.store.record.list",
        ["store", "record", "list"],
        "Query canonical records",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[
            flag("--case", Some("CASE"), false),
            flag("--kind", Some("KIND"), false),
            flag("--subject", Some("SUBJECT"), false),
            flag("--receipt", Some("RECEIPT"), false),
            flag("--limit", Some("N"), false)
        ]
    ),
    op!(
        "yai.store.tail",
        ["store", "tail"],
        "Import/tail a compatibility journal",
        Plumbing,
        Compatibility,
        Mutating,
        PlainCompat,
        NO_POS,
        &[flag("--journal", Some("FILE"), true)]
    ),
    op!(
        "yai.journal.inspect",
        ["journal", "inspect"],
        "Inspect legacy journal compatibility data",
        Compatibility,
        Compatibility,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[
            flag("--path", Some("FILE"), true),
            bool_flag("--show-errors")
        ]
    ),
    op!(
        "yai.journal.compatibility_inspect",
        ["journal", "compatibility-inspect"],
        "Inspect legacy journal compatibility",
        Compatibility,
        Compatibility,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[flag("--path", Some("FILE"), true)]
    ),
    op!(
        "yai.journal.compatibility_import",
        ["journal", "compatibility-import"],
        "Import legacy journal into an isolated store",
        Compatibility,
        Compatibility,
        Mutating,
        PlainCompat,
        NO_POS,
        &[
            flag("--path", Some("FILE"), true),
            flag("--target", Some("DIR"), true),
            bool_flag("--dry-run")
        ]
    ),
    op!(
        "yai.journal.replay",
        ["journal", "replay"],
        "Replay legacy journal compatibility data",
        Compatibility,
        Compatibility,
        Mutating,
        PlainCompat,
        NO_POS,
        &[flag("--path", Some("FILE"), true), bool_flag("--dry-run")]
    ),
    op!(
        "yai.journal.replay_status",
        ["journal", "replay-status"],
        "Inspect journal replay status",
        Compatibility,
        Compatibility,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[flag("--path", Some("FILE"), true)]
    ),
    op!(
        "yai.journal.replay_report",
        ["journal", "replay-report"],
        "Render journal replay report",
        Compatibility,
        Compatibility,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[flag("--path", Some("FILE"), true)]
    ),
    op!(
        "yai.projection.summary",
        ["projection", "summary"],
        "Inspect a derived Projection summary",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[flag("--journal", Some("FILE"), true)]
    ),
    op!(
        "yai.projection.inspect",
        ["projection", "inspect"],
        "Inspect a derived Projection",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[
            flag("--journal", Some("FILE"), true),
            flag("--consumer", Some("CONSUMER"), false)
        ]
    ),
    op!(
        "yai.projection.request",
        ["projection", "request"],
        "Build a derived Projection request",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[
            flag("--journal", Some("FILE"), true),
            flag("--consumer", Some("CONSUMER"), true),
            flag("--kind", Some("KIND"), true)
        ]
    ),
    op!(
        "yai.context.inspect",
        ["context", "inspect"],
        "Inspect a ContextFrame or rendered input",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[aliased_flag(
            "--id",
            &["--projection", "--frame"],
            Some("ID"),
            true,
            "--id"
        )]
    ),
    op!(
        "yai.control.summary",
        ["control", "summary"],
        "Inspect legacy control summary",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[flag("--journal", Some("FILE"), true)]
    ),
    op!(
        "yai.decision.inspect",
        ["decision", "inspect"],
        "Inspect Decision records",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[flag("--journal", Some("FILE"), true)]
    ),
    op!(
        "yai.receipt.summary",
        ["receipt", "summary"],
        "Summarize Receipt records",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[flag("--journal", Some("FILE"), true)]
    ),
    op!(
        "yai.reconcile.summary",
        ["reconcile", "summary"],
        "Summarize reconciliation records",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[flag("--journal", Some("FILE"), true)]
    ),
    op!(
        "yai.query.summary",
        ["query", "summary"],
        "Run legacy record summary query",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[flag("--journal", Some("FILE"), true)]
    ),
    op!(
        "yai.query.records",
        ["query", "records"],
        "Run legacy record query",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[
            flag("--journal", Some("FILE"), true),
            flag("--kind", Some("KIND"), false),
            flag("--case", Some("CASE"), false),
            flag("--limit", Some("N"), false)
        ]
    ),
    op!(
        "yai.engine.summary",
        ["engine", "summary"],
        "Inspect compatibility engine summary",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[flag("--journal", Some("FILE"), true)]
    ),
    op!(
        "yai.hot.status",
        ["hot", "status"],
        "Inspect legacy hot-state projection",
        Compatibility,
        Compatibility,
        ReadOnly,
        PlainCompat,
        NO_POS,
        NO_FLAGS
    ),
    op!(
        "yai.process.observe",
        ["process", "observe"],
        "Observe a host process without enforcing",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[flag("--pid", Some("PID"), true)]
    ),
    op!(
        "yai.process.signal",
        ["process", "signal"],
        "Send a direct diagnostic process signal",
        Plumbing,
        LocalDomain,
        Mutating,
        PlainCompat,
        NO_POS,
        &[
            flag("--pid", Some("PID"), true),
            choice_flag("--signal", &["TERM", "KILL"], true),
            bool_flag("--dry-run")
        ]
    ),
    op!(
        "yai.observe.compare_process",
        ["observe", "compare-process"],
        "Compare observed process posture",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[
            flag("--pid", Some("PID"), true),
            choice_flag("--expected", &["running", "stopped"], true)
        ]
    ),
    op!(
        "yai.carrier.fs_read",
        ["carrier", "fs-read"],
        "Exercise the direct filesystem carrier read probe",
        Plumbing,
        LocalDomain,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[
            flag("--sandbox", Some("DIR"), true),
            flag("--path", Some("PATH"), true)
        ]
    ),
    op!(
        "yai.graph.summary",
        ["graph", "summary"],
        "Summarize the compatibility graph projection",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[flag("--journal", Some("FILE"), true)]
    ),
    op!(
        "yai.graph.schema",
        ["graph", "schema"],
        "Show graph schema metadata",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        NO_FLAGS
    ),
    op!(
        "yai.graph.runtime_status",
        ["graph", "runtime-status"],
        "Inspect derived runtime graph status",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        NO_FLAGS
    ),
    op!(
        "yai.graph.materialize",
        ["graph", "materialize"],
        "Materialize derived graph relations",
        Plumbing,
        Inspection,
        Mutating,
        PlainCompat,
        NO_POS,
        &[flag("--case", Some("CASE"), true)]
    ),
    op!(
        "yai.graph.relations",
        ["graph", "relations"],
        "List derived graph relations",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[
            flag("--case", Some("CASE"), true),
            flag("--limit", Some("N"), false)
        ]
    ),
    op!(
        "yai.graph.runtime_load",
        ["graph", "runtime-load"],
        "Load derived runtime graph",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[flag("--case", Some("CASE"), true)]
    ),
    op!(
        "yai.graph.runtime_summary",
        ["graph", "runtime-summary"],
        "Summarize derived runtime graph",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[flag("--case", Some("CASE"), true)]
    ),
    Descriptor {
        rules: &[
            SyntaxRule::RequiresWhen {
                flag: "--from",
                value: "journal",
                required_flag: "--path",
            },
            SyntaxRule::ConflictsWhen {
                flag: "--from",
                value: "graph-relations",
                conflicting_flag: "--path",
            },
        ],
        ..op!(
            "yai.graph.rebuild",
            ["graph", "rebuild"],
            "Rebuild derived graph state",
            Plumbing,
            Inspection,
            Mutating,
            PlainCompat,
            NO_POS,
            &[
                flag("--case", Some("CASE"), true),
                choice_flag("--from", &["graph-relations", "journal"], true),
                flag("--path", Some("FILE"), false)
            ]
        )
    },
    op!(
        "yai.graph.rebuild_report",
        ["graph", "rebuild-report"],
        "Show derived graph rebuild report",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[flag("--case", Some("CASE"), true)]
    ),
    op!(
        "yai.graph.fanout",
        ["graph", "fanout"],
        "Inspect graph fanout",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[
            flag("--case", Some("CASE"), true),
            flag("--node", Some("NODE"), true),
            flag("--edge-kind", Some("KIND"), false),
            flag("--limit", Some("N"), false)
        ]
    ),
    op!(
        "yai.graph.fanin",
        ["graph", "fanin"],
        "Inspect graph fanin",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[
            flag("--case", Some("CASE"), true),
            flag("--node", Some("NODE"), true),
            flag("--edge-kind", Some("KIND"), false),
            flag("--limit", Some("N"), false)
        ]
    ),
    op!(
        "yai.graph.neighborhood",
        ["graph", "neighborhood"],
        "Inspect bounded graph neighborhood",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[
            flag("--case", Some("CASE"), true),
            flag("--node", Some("NODE"), true),
            choice_flag("--depth", &["1", "2"], false),
            flag("--limit", Some("N"), false)
        ]
    ),
    op!(
        "yai.graph.path",
        ["graph", "path"],
        "Inspect a bounded graph path",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[
            flag("--case", Some("CASE"), true),
            flag("--from", Some("NODE"), true),
            flag("--to", Some("NODE"), true),
            flag("--max-depth", Some("N"), false)
        ]
    ),
    op!(
        "yai.facts.status",
        ["facts", "status"],
        "Inspect derived analytics store status",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        NO_FLAGS
    ),
    op!(
        "yai.facts.schema",
        ["facts", "schema"],
        "Show derived analytics schemas",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        NO_FLAGS
    ),
    op!(
        "yai.facts.init",
        ["facts", "init"],
        "Initialize the derived analytics store",
        Plumbing,
        Inspection,
        Mutating,
        PlainCompat,
        NO_POS,
        NO_FLAGS
    ),
    op!(
        "yai.facts.extract",
        ["facts", "extract"],
        "Extract derived facts from canonical records",
        Plumbing,
        Inspection,
        Mutating,
        PlainCompat,
        NO_POS,
        &[
            flag("--case", Some("CASE"), true),
            choice_flag(
                "--kind",
                &[
                    "receipt",
                    "decision",
                    "projection",
                    "model_behavior",
                    "policy_outcome",
                    "carrier_outcome",
                    "divergence",
                    "memory_quality",
                    "core",
                    "behavior",
                    "operational",
                    "all"
                ],
                true
            )
        ]
    ),
    op!(
        "yai.facts.summary",
        ["facts", "summary"],
        "Summarize derived facts",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[flag("--case", Some("CASE"), true)]
    ),
    op!(
        "yai.facts.report",
        ["facts", "report"],
        "Render a derived fact report",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[
            flag("--case", Some("CASE"), true),
            choice_flag(
                "--section",
                &[
                    "receipts",
                    "decisions",
                    "projections",
                    "policy",
                    "carriers",
                    "divergence",
                    "memory",
                    "model"
                ],
                false
            ),
            choice_flag("--format", &["plain"], false)
        ]
    ),
    op!(
        "yai.memory.summary",
        ["memory", "summary"],
        "Summarize legacy MemoryCandidate compatibility",
        Compatibility,
        Compatibility,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[flag("--journal", Some("FILE"), true)]
    ),
    op!(
        "yai.memory.rebuild",
        ["memory", "rebuild"],
        "Rebuild derived operational memory",
        Plumbing,
        Inspection,
        Mutating,
        PlainCompat,
        NO_POS,
        &[flag("--case", Some("CASE"), true), bool_flag("--dry-run")]
    ),
    op!(
        "yai.memory.clear",
        ["memory", "clear"],
        "Drop rebuildable derived operational memory",
        Plumbing,
        Inspection,
        Mutating,
        PlainCompat,
        NO_POS,
        &[flag("--case", Some("CASE"), true)]
    ),
    op!(
        "yai.memory.list",
        ["memory", "list"],
        "List derived operational memory",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[
            flag("--case", Some("CASE"), true),
            bool_flag("--include-superseded"),
            flag("--limit", Some("N"), false)
        ]
    ),
    op!(
        "yai.memory.show",
        ["memory", "show"],
        "Show one derived memory item",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        &[pos("memory", None)],
        NO_FLAGS
    ),
    op!(
        "yai.memory.provenance",
        ["memory", "provenance"],
        "Show exact memory provenance",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        &[pos("memory", None)],
        NO_FLAGS
    ),
    op!(
        "yai.memory.retrieve",
        ["memory", "retrieve"],
        "Exercise bounded memory retrieval",
        Plumbing,
        Inspection,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[
            flag("--case", Some("CASE"), true),
            flag("--participant", Some("PARTICIPANT"), true),
            choice_flag(
                "--purpose",
                &[
                    "conversation",
                    "filesystem_write_proposal",
                    "effect_consequence",
                    "inspection"
                ],
                true
            ),
            flag("--resource", Some("RESOURCE"), false),
            flag("--kind", Some("KIND"), false),
            flag("--causal-ref", Some("REF"), false),
            bool_flag("--include-superseded"),
            flag("--limit", Some("N"), false)
        ]
    ),
    op!(
        "yai.daemon.status",
        ["daemon", "status"],
        "Inspect optional legacy daemon status",
        Compatibility,
        Compatibility,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[flag("--socket", Some("SOCKET"), true)]
    ),
    op!(
        "yai.daemon.info",
        ["daemon", "info"],
        "Inspect optional legacy daemon information",
        Compatibility,
        Compatibility,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[flag("--socket", Some("SOCKET"), true)]
    ),
    op!(
        "yai.daemon.shutdown",
        ["daemon", "shutdown"],
        "Stop optional legacy daemon",
        Compatibility,
        Compatibility,
        Mutating,
        PlainCompat,
        NO_POS,
        &[flag("--socket", Some("SOCKET"), true)]
    ),
    op!(
        "yai.daemon.run_minimum_loop",
        ["daemon", "run-minimum-loop"],
        "Run legacy daemon minimum-loop probe",
        Compatibility,
        Compatibility,
        Mutating,
        PlainCompat,
        NO_POS,
        &[flag("--socket", Some("SOCKET"), true)]
    ),
    op!(
        "yai.daemon.run_filesystem_loop",
        ["daemon", "run-filesystem-loop"],
        "Run legacy daemon filesystem-loop probe",
        Compatibility,
        Compatibility,
        Mutating,
        PlainCompat,
        NO_POS,
        &[flag("--socket", Some("SOCKET"), true)]
    ),
    op!(
        "yai.daemon.journal_summary",
        ["daemon", "journal-summary"],
        "Request legacy daemon journal summary",
        Compatibility,
        Compatibility,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[
            flag("--socket", Some("SOCKET"), true),
            flag("--journal", Some("FILE"), true)
        ]
    ),
    op!(
        "yai.daemon.projection_summary",
        ["daemon", "projection-summary"],
        "Request legacy daemon projection summary",
        Compatibility,
        Compatibility,
        ReadOnly,
        PlainCompat,
        NO_POS,
        &[
            flag("--socket", Some("SOCKET"), true),
            flag("--journal", Some("FILE"), true)
        ]
    ),
    Descriptor {
        operation_id: "yai.removed.observe_process",
        handler_id: "removed",
        path: &["observe", "process"],
        description: "Removed duplicate process observation path",
        visibility: Visibility::Removed,
        lane: Lane::Compatibility,
        mutation: Mutation::ReadOnly,
        output: OutputCapability::PlainCompat,
        positionals: NO_POS,
        flags: &[flag("--pid", Some("PID"), true)],
        rules: &[],
        aliases: NO_ALIASES,
        legacy_path: &[],
        removed_successor: Some(&["process", "observe"]),
    },
];

pub(crate) fn registry_digest() -> String {
    let bytes = serde_json::to_vec(&(REGISTRY_SCHEMA, PRODUCT_ROOTS, REGISTRY))
        .expect("static CLI registry serializes");
    yai_core_engine::effect::digest_bytes(&bytes)
}

pub(crate) fn validate() -> Result<(), String> {
    use std::collections::HashSet;
    let mut ids = HashSet::new();
    let mut paths = HashSet::new();
    let mut product_roots = HashSet::new();
    for root in PRODUCT_ROOTS {
        if !product_roots.insert(root.word) {
            return Err(format!("duplicate product root: {}", root.word));
        }
        if !REGISTRY.iter().any(|descriptor| {
            descriptor.visibility == Visibility::Product
                && descriptor.path.first().copied() == Some(root.word)
        }) {
            return Err(format!(
                "product root has no Product operation: {}",
                root.word
            ));
        }
    }
    for descriptor in REGISTRY {
        if !ids.insert(descriptor.operation_id) {
            return Err(format!(
                "duplicate operation ID: {}",
                descriptor.operation_id
            ));
        }
        if descriptor.handler_id.is_empty() {
            return Err(format!(
                "operation {} has no handler adapter",
                descriptor.operation_id
            ));
        }
        let path = descriptor.path.join(" ");
        if !paths.insert(path.clone()) {
            return Err(format!("duplicate canonical path: {path}"));
        }
        for alias in descriptor.aliases {
            let alias = alias.join(" ");
            if !paths.insert(alias.clone()) {
                return Err(format!("duplicate or ambiguous alias: {alias}"));
            }
        }
        let mut flags = HashSet::new();
        for flag in descriptor.flags {
            if !flags.insert(flag.name) {
                return Err(format!("duplicate flag {} for {path}", flag.name));
            }
            if flag.required && flag.value_name.is_none() {
                return Err(format!("required boolean flag {} for {path}", flag.name));
            }
        }
        for rule in descriptor.rules {
            let names = match rule {
                SyntaxRule::RequiresWhen {
                    flag,
                    required_flag,
                    ..
                } => [*flag, *required_flag],
                SyntaxRule::ConflictsWhen {
                    flag,
                    conflicting_flag,
                    ..
                } => [*flag, *conflicting_flag],
            };
            for name in names {
                if !descriptor.flags.iter().any(|flag| flag.name == name) {
                    return Err(format!(
                        "syntax rule references unknown flag {name} for {path}"
                    ));
                }
            }
        }
        if descriptor.visibility == Visibility::Removed && descriptor.removed_successor.is_none() {
            return Err(format!("removed operation {path} has no successor"));
        }
        if descriptor.visibility == Visibility::Product
            && !product_roots.contains(descriptor.path[0])
        {
            return Err(format!(
                "Product operation has no product-root projection: {path}"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_is_self_consistent() {
        validate().unwrap();
    }

    #[test]
    fn product_operations_are_structured() {
        for descriptor in REGISTRY
            .iter()
            .filter(|descriptor| descriptor.visibility == Visibility::Product)
        {
            assert_eq!(
                descriptor.output,
                OutputCapability::Structured,
                "{}",
                descriptor.operation_id
            );
        }
    }
}
