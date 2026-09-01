//! Immutable workflow definitions and pure Case-bound progression.
//!
//! Workflow describes deterministic progression. It does not own Case truth,
//! policy authority, RuntimeWorkItems, provider identity, resource leases, or
//! physical effects. Canonical progression facts remain Case transitions;
//! `WorkflowResolution` is a rebuildable view.

use crate::effect::{
    digest_bytes, normalize_relative_path, DecisionOutcome, OperationKind, ProcessSignalAction,
};
use crate::transition::{
    CaseLifecycle, CaseState, EffectLifecycle, ReviewResolution, Transition, TransitionPayload,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const WORKFLOW_DEFINITION_SCHEMA: &str = "yai.workflow_definition.v1";
pub const CASE_WORKFLOW_BINDING_SCHEMA: &str = "yai.case_workflow_binding.v1";
pub const WORKFLOW_NODE_EXECUTION_SCHEMA: &str = "yai.workflow_node_execution.v1";
pub const WORKFLOW_NODE_SATISFACTION_SCHEMA: &str = "yai.workflow_node_satisfaction.v1";
pub const WORKFLOW_CONDITION_RESOLUTION_SCHEMA: &str = "yai.workflow_condition_resolution.v1";
pub const WORKFLOW_HUMAN_INPUT_SCHEMA: &str = "yai.workflow_human_input.v1";
pub const WORKFLOW_DETERMINISTIC_PROPOSAL_SCHEMA: &str = "yai.workflow_deterministic_proposal.v1";
pub const WORKFLOW_RESOLUTION_SCHEMA: &str = "yai.workflow_resolution.v1";

pub const MAX_WORKFLOW_NODES: usize = 128;
pub const MAX_WORKFLOW_EDGES: usize = 512;
pub const MAX_WORKFLOW_ID_BYTES: usize = 128;
pub const MAX_WORKFLOW_LABEL_BYTES: usize = 512;
pub const MAX_WORKFLOW_TASK_BYTES: usize = 64 * 1024;
pub const MAX_WORKFLOW_INPUT_BYTES: usize = 16 * 1024;
pub const MAX_WORKFLOW_OPERATION_CONTENT_BYTES: usize = 64 * 1024;
pub const MAX_WORKFLOW_DEFINITION_BYTES: usize = 2 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowDefinitionInput {
    pub schema: String,
    pub tenant_id: String,
    pub workflow_key: String,
    pub declared_version: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub nodes: Vec<WorkflowNode>,
    #[serde(default)]
    pub edges: Vec<WorkflowEdge>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowDefinition {
    pub schema: String,
    pub workflow_definition_id: String,
    pub content_digest: String,
    pub integrity_digest: String,
    pub tenant_id: String,
    pub workflow_key: String,
    pub declared_version: String,
    pub name: String,
    pub description: String,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
    pub created_by_principal_id: String,
    pub created_at_unix_ms: u64,
}

#[derive(Serialize)]
struct DefinitionSemanticMaterial<'a> {
    schema: &'a str,
    tenant_id: &'a str,
    workflow_key: &'a str,
    declared_version: &'a str,
    name: &'a str,
    description: &'a str,
    nodes: &'a [WorkflowNode],
    edges: &'a [WorkflowEdge],
}

#[derive(Serialize)]
struct DefinitionIntegrityMaterial<'a> {
    semantic: DefinitionSemanticMaterial<'a>,
    workflow_definition_id: &'a str,
    content_digest: &'a str,
    created_by_principal_id: &'a str,
    created_at_unix_ms: u64,
}

impl WorkflowDefinition {
    pub fn build(
        input: WorkflowDefinitionInput,
        created_by_principal_id: &str,
        created_at_unix_ms: u64,
    ) -> Result<Self, String> {
        input.validate()?;
        if !created_by_principal_id.starts_with("principal:") {
            return Err("workflow_definition_creator_invalid".to_string());
        }
        let semantic = DefinitionSemanticMaterial {
            schema: WORKFLOW_DEFINITION_SCHEMA,
            tenant_id: &input.tenant_id,
            workflow_key: &input.workflow_key,
            declared_version: &input.declared_version,
            name: &input.name,
            description: &input.description,
            nodes: &input.nodes,
            edges: &input.edges,
        };
        let content_digest = digest_serializable(&semantic)?;
        let workflow_definition_id =
            format!("workflow-definition:{}", digest_component(&content_digest));
        let mut definition = Self {
            schema: WORKFLOW_DEFINITION_SCHEMA.to_string(),
            workflow_definition_id,
            content_digest,
            integrity_digest: String::new(),
            tenant_id: input.tenant_id,
            workflow_key: input.workflow_key,
            declared_version: input.declared_version,
            name: input.name,
            description: input.description,
            nodes: input.nodes,
            edges: input.edges,
            created_by_principal_id: created_by_principal_id.to_string(),
            created_at_unix_ms,
        };
        definition.integrity_digest = definition.expected_integrity_digest()?;
        definition.validate_integrity()?;
        Ok(definition)
    }

    pub fn node(&self, node_id: &str) -> Option<&WorkflowNode> {
        self.nodes.iter().find(|node| node.node_id == node_id)
    }

    pub fn validate_integrity(&self) -> Result<(), String> {
        WorkflowDefinitionInput {
            schema: self.schema.clone(),
            tenant_id: self.tenant_id.clone(),
            workflow_key: self.workflow_key.clone(),
            declared_version: self.declared_version.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
        }
        .validate()?;
        if !self.created_by_principal_id.starts_with("principal:") {
            return Err("workflow_definition_creator_invalid".to_string());
        }
        let semantic = DefinitionSemanticMaterial {
            schema: &self.schema,
            tenant_id: &self.tenant_id,
            workflow_key: &self.workflow_key,
            declared_version: &self.declared_version,
            name: &self.name,
            description: &self.description,
            nodes: &self.nodes,
            edges: &self.edges,
        };
        let content_digest = digest_serializable(&semantic)?;
        let expected_id = format!("workflow-definition:{}", digest_component(&content_digest));
        if self.content_digest != content_digest || self.workflow_definition_id != expected_id {
            return Err("workflow_definition_content_identity_mismatch".to_string());
        }
        if self.integrity_digest != self.expected_integrity_digest()? {
            return Err("workflow_definition_integrity_mismatch".to_string());
        }
        Ok(())
    }

    fn expected_integrity_digest(&self) -> Result<String, String> {
        digest_serializable(&DefinitionIntegrityMaterial {
            semantic: DefinitionSemanticMaterial {
                schema: &self.schema,
                tenant_id: &self.tenant_id,
                workflow_key: &self.workflow_key,
                declared_version: &self.declared_version,
                name: &self.name,
                description: &self.description,
                nodes: &self.nodes,
                edges: &self.edges,
            },
            workflow_definition_id: &self.workflow_definition_id,
            content_digest: &self.content_digest,
            created_by_principal_id: &self.created_by_principal_id,
            created_at_unix_ms: self.created_at_unix_ms,
        })
    }
}

impl WorkflowDefinitionInput {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != WORKFLOW_DEFINITION_SCHEMA {
            return Err("unsupported_workflow_definition_schema".to_string());
        }
        let encoded_bytes = serde_json::to_vec(self)
            .map_err(|error| format!("workflow_definition_encode_failed: {error}"))?
            .len();
        if encoded_bytes > MAX_WORKFLOW_DEFINITION_BYTES {
            return Err("workflow_definition_aggregate_bounds_invalid".to_string());
        }
        if !self.tenant_id.starts_with("tenant:")
            || !valid_id(&self.workflow_key)
            || !valid_id(&self.declared_version)
            || self.name.is_empty()
            || self.name.len() > MAX_WORKFLOW_LABEL_BYTES
            || self.description.len() > MAX_WORKFLOW_TASK_BYTES
            || self.nodes.is_empty()
            || self.nodes.len() > MAX_WORKFLOW_NODES
            || self.edges.len() > MAX_WORKFLOW_EDGES
        {
            return Err("workflow_definition_bounds_invalid".to_string());
        }
        let mut ids = BTreeSet::new();
        for node in &self.nodes {
            node.validate()?;
            if !ids.insert(node.node_id.clone()) {
                return Err("workflow_duplicate_node_id".to_string());
            }
        }
        for node in &self.nodes {
            let predicate = match &node.kind {
                WorkflowNodeKind::ModelWork { completion, .. }
                | WorkflowNodeKind::DeterministicWork { completion, .. } => Some(completion),
                WorkflowNodeKind::Condition { predicate }
                | WorkflowNodeKind::Wait { predicate }
                | WorkflowNodeKind::EffectGoal { predicate } => Some(predicate),
                WorkflowNodeKind::HumanInput { .. } => None,
            };
            if let Some(WorkflowPredicate::NodeSatisfied { node_id }) = predicate {
                if !ids.contains(node_id) || node_id == &node.node_id {
                    return Err("workflow_predicate_node_reference_invalid".to_string());
                }
            }
        }
        let mut edge_ids = BTreeSet::new();
        let mut edge_pairs = BTreeSet::new();
        for edge in &self.edges {
            edge.validate()?;
            if !ids.contains(&edge.from) || !ids.contains(&edge.to) {
                return Err("workflow_dangling_edge".to_string());
            }
            if edge.from == edge.to {
                return Err("workflow_self_cycle".to_string());
            }
            if !edge_ids.insert((edge.from.clone(), edge.to.clone(), edge.kind.clone())) {
                return Err("workflow_duplicate_edge".to_string());
            }
            if !edge_pairs.insert((edge.from.clone(), edge.to.clone())) {
                return Err("workflow_ambiguous_parallel_edge".to_string());
            }
            let source = self
                .nodes
                .iter()
                .find(|node| node.node_id == edge.from)
                .expect("edge source validated");
            if matches!(
                edge.kind,
                WorkflowEdgeKind::OnTrue | WorkflowEdgeKind::OnFalse
            ) && !matches!(source.kind, WorkflowNodeKind::Condition { .. })
            {
                return Err("workflow_conditional_edge_requires_condition_source".to_string());
            }
        }
        topological_order(&self.nodes, &self.edges)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub node_id: String,
    #[serde(flatten)]
    pub kind: WorkflowNodeKind,
}

impl WorkflowNode {
    fn validate(&self) -> Result<(), String> {
        if !valid_id(&self.node_id) {
            return Err("workflow_node_id_invalid".to_string());
        }
        match &self.kind {
            WorkflowNodeKind::ModelWork {
                executor_slot,
                task,
                completion,
                budgets,
                resource_slot,
            } => {
                require_slot(executor_slot)?;
                if task.is_empty() || task.len() > MAX_WORKFLOW_TASK_BYTES {
                    return Err("workflow_model_task_bounds_invalid".to_string());
                }
                if let Some(slot) = resource_slot {
                    require_slot(slot)?;
                }
                completion.validate()?;
                budgets.validate()?;
            }
            WorkflowNodeKind::DeterministicWork {
                proposer_slot,
                operation,
                completion,
            } => {
                require_slot(proposer_slot)?;
                operation.validate()?;
                completion.validate()?;
            }
            WorkflowNodeKind::HumanInput {
                actor_slot,
                prompt,
                required_roles,
                input_kind: _,
                max_bytes,
            } => {
                require_slot(actor_slot)?;
                if prompt.is_empty()
                    || prompt.len() > MAX_WORKFLOW_LABEL_BYTES
                    || *max_bytes == 0
                    || *max_bytes > MAX_WORKFLOW_INPUT_BYTES
                    || required_roles.len() > 16
                    || required_roles.iter().any(|role| !valid_id(role))
                {
                    return Err("workflow_human_input_contract_invalid".to_string());
                }
            }
            WorkflowNodeKind::Condition { predicate }
            | WorkflowNodeKind::Wait { predicate }
            | WorkflowNodeKind::EffectGoal { predicate } => predicate.validate()?,
        }
        validate_predicate_placement(&self.kind)?;
        Ok(())
    }

    pub fn is_executable(&self) -> bool {
        matches!(
            self.kind,
            WorkflowNodeKind::ModelWork { .. } | WorkflowNodeKind::DeterministicWork { .. }
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum WorkflowNodeKind {
    ModelWork {
        executor_slot: String,
        task: String,
        completion: WorkflowPredicate,
        #[serde(default)]
        budgets: WorkflowBudgets,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        resource_slot: Option<String>,
    },
    DeterministicWork {
        proposer_slot: String,
        operation: DeterministicOperationTemplate,
        completion: WorkflowPredicate,
    },
    HumanInput {
        actor_slot: String,
        prompt: String,
        #[serde(default)]
        required_roles: Vec<String>,
        #[serde(default)]
        input_kind: HumanInputKind,
        #[serde(default = "default_human_input_bytes")]
        max_bytes: usize,
    },
    Condition {
        predicate: WorkflowPredicate,
    },
    Wait {
        predicate: WorkflowPredicate,
    },
    EffectGoal {
        predicate: WorkflowPredicate,
    },
}

fn default_human_input_bytes() -> usize {
    4096
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowBudgets {
    #[serde(default = "default_model_turns")]
    pub max_turns: usize,
    #[serde(default = "default_model_operations")]
    pub max_operations: usize,
    #[serde(default = "default_semantic_units")]
    pub max_semantic_units: usize,
}

fn default_model_turns() -> usize {
    3
}
fn default_model_operations() -> usize {
    2
}
fn default_semantic_units() -> usize {
    6000
}

impl Default for WorkflowBudgets {
    fn default() -> Self {
        Self {
            max_turns: default_model_turns(),
            max_operations: default_model_operations(),
            max_semantic_units: default_semantic_units(),
        }
    }
}

impl WorkflowBudgets {
    fn validate(&self) -> Result<(), String> {
        if self.max_turns == 0
            || self.max_turns > 32
            || self.max_operations == 0
            || self.max_operations > 32
            || self.max_semantic_units == 0
            || self.max_semantic_units > 1_000_000
        {
            return Err("workflow_model_budget_invalid".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HumanInputKind {
    #[default]
    Text,
    Json,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum DeterministicOperationTemplate {
    FilesystemWrite {
        resource_slot: String,
        relative_path: String,
        content: String,
    },
    ProcessSignal {
        resource_slot: String,
        action: ProcessSignalAction,
    },
}

impl DeterministicOperationTemplate {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::FilesystemWrite {
                resource_slot,
                relative_path,
                content,
            } => {
                require_slot(resource_slot)?;
                if !safe_relative_path(relative_path)
                    || content.is_empty()
                    || content.len() > MAX_WORKFLOW_OPERATION_CONTENT_BYTES
                {
                    return Err("workflow_filesystem_template_invalid".to_string());
                }
            }
            Self::ProcessSignal {
                resource_slot,
                action: _,
            } => require_slot(resource_slot)?,
        }
        Ok(())
    }

    pub fn operation_kind(&self) -> OperationKind {
        match self {
            Self::FilesystemWrite { .. } => OperationKind::FilesystemWrite,
            Self::ProcessSignal { .. } => OperationKind::ProcessSignal,
        }
    }

    pub fn resource_slot(&self) -> &str {
        match self {
            Self::FilesystemWrite { resource_slot, .. }
            | Self::ProcessSignal { resource_slot, .. } => resource_slot,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowEdge {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub kind: WorkflowEdgeKind,
}

impl WorkflowEdge {
    fn validate(&self) -> Result<(), String> {
        if !valid_id(&self.from) || !valid_id(&self.to) {
            return Err("workflow_edge_id_invalid".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowEdgeKind {
    #[default]
    Always,
    OnTrue,
    OnFalse,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "predicate", rename_all = "snake_case", deny_unknown_fields)]
pub enum WorkflowPredicate {
    ExecutionProviderResult,
    ExecutionEffectFinalized,
    ExecutionFilesystemEffectFinalized {
        relative_path: String,
    },
    HumanInputRecorded,
    CaseLifecycle {
        lifecycle: CaseLifecycle,
    },
    NodeSatisfied {
        node_id: String,
    },
    DecisionOutcome {
        outcome: DecisionOutcome,
    },
    ReviewTerminal,
    FinalizedEffect {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        resource_slot: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        operation_kind: Option<OperationKind>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowPredicateScope {
    Execution,
    Node,
    Case,
    Progression,
}

impl WorkflowPredicate {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::NodeSatisfied { node_id } => {
                if !valid_id(node_id) {
                    return Err("workflow_predicate_node_id_invalid".to_string());
                }
            }
            Self::FinalizedEffect {
                resource_slot: Some(slot),
                ..
            } => require_slot(slot)?,
            Self::ExecutionFilesystemEffectFinalized { relative_path } => {
                if relative_path.len() > MAX_WORKFLOW_LABEL_BYTES
                    || normalize_relative_path(relative_path)? != *relative_path
                {
                    return Err("workflow_predicate_relative_path_invalid".to_string());
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn digest(&self) -> Result<String, String> {
        digest_serializable(self)
    }

    pub fn scope(&self) -> WorkflowPredicateScope {
        match self {
            Self::ExecutionProviderResult
            | Self::ExecutionEffectFinalized
            | Self::ExecutionFilesystemEffectFinalized { .. }
            | Self::DecisionOutcome { .. }
            | Self::ReviewTerminal => WorkflowPredicateScope::Execution,
            Self::HumanInputRecorded => WorkflowPredicateScope::Node,
            Self::CaseLifecycle { .. } | Self::FinalizedEffect { .. } => {
                WorkflowPredicateScope::Case
            }
            Self::NodeSatisfied { .. } => WorkflowPredicateScope::Progression,
        }
    }
}

fn validate_predicate_placement(kind: &WorkflowNodeKind) -> Result<(), String> {
    let (predicate, permitted) = match kind {
        WorkflowNodeKind::ModelWork { completion, .. }
        | WorkflowNodeKind::DeterministicWork { completion, .. } => (
            Some(completion),
            completion.scope() == WorkflowPredicateScope::Execution,
        ),
        WorkflowNodeKind::HumanInput { .. } => (None, true),
        WorkflowNodeKind::Condition { predicate } | WorkflowNodeKind::Wait { predicate } => (
            Some(predicate),
            matches!(
                predicate.scope(),
                WorkflowPredicateScope::Case | WorkflowPredicateScope::Progression
            ),
        ),
        WorkflowNodeKind::EffectGoal { predicate } => (
            Some(predicate),
            matches!(predicate, WorkflowPredicate::FinalizedEffect { .. }),
        ),
    };
    if permitted {
        return Ok(());
    }
    if predicate.is_some_and(|value| value.scope() == WorkflowPredicateScope::Node) {
        return Err("workflow_human_input_predicate_placement_invalid".to_string());
    }
    Err("workflow_predicate_node_kind_incompatible".to_string())
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowExecutorBinding {
    pub slot: String,
    pub participant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowResourceBinding {
    pub slot: String,
    pub attachment_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseWorkflowBinding {
    pub schema: String,
    pub binding_id: String,
    pub integrity_digest: String,
    pub tenant_id: String,
    pub case_id: String,
    pub workflow_definition_id: String,
    pub workflow_definition_digest: String,
    pub executor_bindings: Vec<WorkflowExecutorBinding>,
    pub resource_bindings: Vec<WorkflowResourceBinding>,
    pub bound_at_generation: u64,
    pub bound_by_principal_id: String,
    pub bound_at_unix_ms: u64,
}

impl CaseWorkflowBinding {
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        tenant_id: &str,
        case_id: &str,
        definition: &WorkflowDefinition,
        executor_bindings: Vec<WorkflowExecutorBinding>,
        resource_bindings: Vec<WorkflowResourceBinding>,
        bound_at_generation: u64,
        bound_by_principal_id: &str,
        bound_at_unix_ms: u64,
    ) -> Result<Self, String> {
        let material = serde_json::json!({
            "schema": CASE_WORKFLOW_BINDING_SCHEMA,
            "tenant_id": tenant_id,
            "case_id": case_id,
            "workflow_definition_id": definition.workflow_definition_id,
            "workflow_definition_digest": definition.integrity_digest,
            "executor_bindings": executor_bindings,
            "resource_bindings": resource_bindings,
            "bound_at_generation": bound_at_generation,
            "bound_by_principal_id": bound_by_principal_id,
            "bound_at_unix_ms": bound_at_unix_ms,
        });
        let integrity_digest = digest_serializable(&material)?;
        let result = Self {
            schema: CASE_WORKFLOW_BINDING_SCHEMA.to_string(),
            binding_id: format!(
                "case-workflow-binding:{}",
                digest_component(&integrity_digest)
            ),
            integrity_digest,
            tenant_id: tenant_id.to_string(),
            case_id: case_id.to_string(),
            workflow_definition_id: definition.workflow_definition_id.clone(),
            workflow_definition_digest: definition.integrity_digest.clone(),
            executor_bindings,
            resource_bindings,
            bound_at_generation,
            bound_by_principal_id: bound_by_principal_id.to_string(),
            bound_at_unix_ms,
        };
        result.validate(definition)?;
        Ok(result)
    }

    pub fn validate(&self, definition: &WorkflowDefinition) -> Result<(), String> {
        if self.schema != CASE_WORKFLOW_BINDING_SCHEMA
            || self.tenant_id != definition.tenant_id
            || self.workflow_definition_id != definition.workflow_definition_id
            || self.workflow_definition_digest != definition.integrity_digest
            || !self.tenant_id.starts_with("tenant:")
            || self.case_id.is_empty()
            || !self.bound_by_principal_id.starts_with("principal:")
        {
            return Err("case_workflow_binding_contract_invalid".to_string());
        }
        let mut executor_slots = BTreeSet::new();
        for binding in &self.executor_bindings {
            require_slot(&binding.slot)?;
            if binding.participant_id.is_empty() || !executor_slots.insert(binding.slot.clone()) {
                return Err("workflow_executor_binding_invalid".to_string());
            }
        }
        let mut resource_slots = BTreeSet::new();
        for binding in &self.resource_bindings {
            require_slot(&binding.slot)?;
            if binding.attachment_id.is_empty() || !resource_slots.insert(binding.slot.clone()) {
                return Err("workflow_resource_binding_invalid".to_string());
            }
        }
        for node in &definition.nodes {
            match &node.kind {
                WorkflowNodeKind::ModelWork {
                    executor_slot,
                    resource_slot,
                    ..
                } => {
                    if !executor_slots.contains(executor_slot)
                        || resource_slot
                            .as_ref()
                            .is_some_and(|slot| !resource_slots.contains(slot))
                    {
                        return Err("workflow_model_slot_unbound".to_string());
                    }
                }
                WorkflowNodeKind::DeterministicWork {
                    proposer_slot,
                    operation,
                    ..
                } => {
                    if !executor_slots.contains(proposer_slot)
                        || !resource_slots.contains(operation.resource_slot())
                    {
                        return Err("workflow_deterministic_slot_unbound".to_string());
                    }
                }
                WorkflowNodeKind::HumanInput { actor_slot, .. } => {
                    if !executor_slots.contains(actor_slot) {
                        return Err("workflow_human_slot_unbound".to_string());
                    }
                }
                WorkflowNodeKind::Condition { predicate }
                | WorkflowNodeKind::Wait { predicate }
                | WorkflowNodeKind::EffectGoal { predicate } => {
                    if let WorkflowPredicate::FinalizedEffect {
                        resource_slot: Some(slot),
                        ..
                    } = predicate
                    {
                        if !resource_slots.contains(slot) {
                            return Err("workflow_predicate_resource_slot_unbound".to_string());
                        }
                    }
                }
            }
        }
        let material = serde_json::json!({
            "schema": self.schema,
            "tenant_id": self.tenant_id,
            "case_id": self.case_id,
            "workflow_definition_id": self.workflow_definition_id,
            "workflow_definition_digest": self.workflow_definition_digest,
            "executor_bindings": self.executor_bindings,
            "resource_bindings": self.resource_bindings,
            "bound_at_generation": self.bound_at_generation,
            "bound_by_principal_id": self.bound_by_principal_id,
            "bound_at_unix_ms": self.bound_at_unix_ms,
        });
        let digest = digest_serializable(&material)?;
        if self.integrity_digest != digest
            || self.binding_id != format!("case-workflow-binding:{}", digest_component(&digest))
        {
            return Err("case_workflow_binding_integrity_mismatch".to_string());
        }
        Ok(())
    }

    pub fn participant_for_slot(&self, slot: &str) -> Option<&str> {
        self.executor_bindings
            .iter()
            .find(|binding| binding.slot == slot)
            .map(|binding| binding.participant_id.as_str())
    }

    pub fn attachment_for_slot(&self, slot: &str) -> Option<&str> {
        self.resource_bindings
            .iter()
            .find(|binding| binding.slot == slot)
            .map(|binding| binding.attachment_id.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowNodeExecution {
    pub schema: String,
    pub execution_id: String,
    pub binding_id: String,
    pub workflow_definition_id: String,
    pub node_id: String,
    pub case_id: String,
    pub started_at_generation: u64,
    pub started_at_unix_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowNodeSatisfaction {
    pub schema: String,
    pub satisfaction_id: String,
    pub binding_id: String,
    pub workflow_definition_id: String,
    pub node_id: String,
    pub execution_id: Option<String>,
    pub predicate_digest: String,
    pub evaluated_at_generation: u64,
    pub evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowConditionResolution {
    pub schema: String,
    pub resolution_id: String,
    pub binding_id: String,
    pub workflow_definition_id: String,
    pub node_id: String,
    pub result: bool,
    pub predicate_digest: String,
    pub evaluated_at_generation: u64,
    pub evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowHumanInputRecord {
    pub schema: String,
    pub input_id: String,
    pub binding_id: String,
    pub workflow_definition_id: String,
    pub node_id: String,
    pub principal_id: String,
    pub participant_id: String,
    pub value: String,
    pub value_digest: String,
    pub recorded_at_generation: u64,
    pub recorded_at_unix_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowDeterministicProposalRecord {
    pub schema: String,
    pub proposal_id: String,
    pub binding_id: String,
    pub workflow_definition_id: String,
    pub node_id: String,
    pub execution_id: String,
    pub participant_id: String,
    pub operation_kind: OperationKind,
    pub resource_attachment_id: String,
    pub template_digest: String,
    pub recorded_at_generation: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowNodePosture {
    PendingDependency,
    Ready,
    Active,
    WaitingHumanInput,
    WaitingCondition,
    WaitingEffect,
    Satisfied,
    Skipped,
    Cancelled,
    Closed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkflowNodeResolution {
    pub node_id: String,
    pub node_kind: String,
    pub posture: WorkflowNodePosture,
    pub reason: String,
    pub execution_id: Option<String>,
    pub evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReadyWorkflowWork {
    pub node_id: String,
    pub node_kind: String,
    pub topological_rank: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkflowResolution {
    pub schema: String,
    pub tenant_id: String,
    pub case_id: String,
    pub case_generation: u64,
    pub workflow_definition_id: String,
    pub workflow_binding_id: String,
    pub nodes: Vec<WorkflowNodeResolution>,
    pub ready_work: Vec<ReadyWorkflowWork>,
    pub satisfied_count: usize,
    pub skipped_count: usize,
    pub active_count: usize,
    pub waiting_count: usize,
    pub completed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PredicateEvaluation {
    pub value: bool,
    pub evidence_refs: Vec<String>,
}

pub fn resolve_workflow(
    definition: &WorkflowDefinition,
    binding: &CaseWorkflowBinding,
    state: &CaseState,
    history: &[Transition],
) -> Result<WorkflowResolution, String> {
    definition.validate_integrity()?;
    binding.validate(definition)?;
    validate_history_snapshot(state, history)?;
    if binding.case_id != state.case_id
        || state.tenant_id.as_deref() != Some(binding.tenant_id.as_str())
        || state.workflow_binding.as_ref() != Some(binding)
    {
        return Err("workflow_resolution_binding_case_mismatch".to_string());
    }
    let order = topological_order(&definition.nodes, &definition.edges)?;
    let rank: BTreeMap<String, usize> = order
        .iter()
        .enumerate()
        .map(|(rank, id)| (id.clone(), rank))
        .collect();
    let mut satisfied: BTreeSet<String> = state
        .workflow_satisfactions
        .iter()
        .map(|fact| fact.node_id.clone())
        .collect();
    satisfied.extend(
        state
            .workflow_conditions
            .iter()
            .map(|fact| fact.node_id.clone()),
    );
    satisfied.extend(
        state
            .workflow_human_inputs
            .iter()
            .map(|fact| fact.node_id.clone()),
    );
    let conditions: BTreeMap<String, bool> = state
        .workflow_conditions
        .iter()
        .map(|fact| (fact.node_id.clone(), fact.result))
        .collect();
    let executions: BTreeMap<String, &WorkflowNodeExecution> = state
        .workflow_executions
        .iter()
        .map(|execution| (execution.node_id.clone(), execution))
        .collect();
    let mut skipped_nodes = BTreeSet::new();
    let mut nodes = Vec::with_capacity(definition.nodes.len());
    let mut ready_work = Vec::new();
    for node_id in &order {
        let node = definition.node(node_id).expect("topological node exists");
        let execution = executions.get(node_id).copied();
        let (dependencies_ready, skipped, dependency_reason) =
            dependency_posture(node_id, definition, &satisfied, &conditions, &skipped_nodes);
        let (posture, reason, evidence_refs) = if state.lifecycle == CaseLifecycle::Closed {
            (
                WorkflowNodePosture::Closed,
                "case_closed".to_string(),
                Vec::new(),
            )
        } else if state.cancellation.is_some() {
            (
                WorkflowNodePosture::Cancelled,
                "case_cancelled".to_string(),
                Vec::new(),
            )
        } else if satisfied.contains(node_id) {
            let evidence_refs = state
                .workflow_satisfactions
                .iter()
                .find(|fact| fact.node_id == *node_id)
                .map(|fact| fact.evidence_refs.clone())
                .or_else(|| {
                    state
                        .workflow_conditions
                        .iter()
                        .find(|fact| fact.node_id == *node_id)
                        .map(|fact| fact.evidence_refs.clone())
                })
                .or_else(|| {
                    state
                        .workflow_human_inputs
                        .iter()
                        .find(|fact| fact.node_id == *node_id)
                        .map(|fact| vec![fact.input_id.clone()])
                })
                .unwrap_or_default();
            (
                WorkflowNodePosture::Satisfied,
                "canonical_satisfaction_recorded".to_string(),
                evidence_refs,
            )
        } else if skipped {
            (WorkflowNodePosture::Skipped, dependency_reason, Vec::new())
        } else if !dependencies_ready {
            (
                WorkflowNodePosture::PendingDependency,
                dependency_reason,
                Vec::new(),
            )
        } else if execution.is_some() {
            let execution_id = execution.expect("checked").execution_id.as_str();
            let predicate = node_completion_predicate(node);
            let evaluation = predicate
                .map(|predicate| {
                    evaluate_predicate(
                        definition,
                        binding,
                        state,
                        history,
                        node_id,
                        Some(execution_id),
                        predicate,
                    )
                })
                .transpose()?;
            if evaluation.as_ref().is_some_and(|value| value.value) {
                (
                    WorkflowNodePosture::Active,
                    "completion_proven_pending_canonical_satisfaction".to_string(),
                    evaluation.expect("checked").evidence_refs,
                )
            } else {
                (
                    WorkflowNodePosture::Active,
                    "workflow_execution_active".to_string(),
                    Vec::new(),
                )
            }
        } else {
            match &node.kind {
                WorkflowNodeKind::ModelWork { .. } | WorkflowNodeKind::DeterministicWork { .. } => {
                    ready_work.push(ReadyWorkflowWork {
                        node_id: node_id.clone(),
                        node_kind: node_kind_name(&node.kind).to_string(),
                        topological_rank: rank[node_id],
                    });
                    (
                        WorkflowNodePosture::Ready,
                        "dependencies_satisfied".to_string(),
                        Vec::new(),
                    )
                }
                WorkflowNodeKind::HumanInput { .. } => (
                    WorkflowNodePosture::WaitingHumanInput,
                    "human_input_required".to_string(),
                    Vec::new(),
                ),
                WorkflowNodeKind::Condition { predicate } => {
                    let evaluation = evaluate_predicate(
                        definition, binding, state, history, node_id, None, predicate,
                    )?;
                    (
                        WorkflowNodePosture::WaitingCondition,
                        if evaluation.value {
                            "condition_resolvable_true"
                        } else {
                            "condition_resolvable_false"
                        }
                        .to_string(),
                        evaluation.evidence_refs,
                    )
                }
                WorkflowNodeKind::Wait { predicate }
                | WorkflowNodeKind::EffectGoal { predicate } => {
                    let evaluation = evaluate_predicate(
                        definition, binding, state, history, node_id, None, predicate,
                    )?;
                    (
                        WorkflowNodePosture::WaitingEffect,
                        if evaluation.value {
                            "passive_predicate_satisfied_pending_commit"
                        } else {
                            "passive_predicate_not_satisfied"
                        }
                        .to_string(),
                        evaluation.evidence_refs,
                    )
                }
            }
        };
        nodes.push(WorkflowNodeResolution {
            node_id: node_id.clone(),
            node_kind: node_kind_name(&node.kind).to_string(),
            posture,
            reason,
            execution_id: execution.map(|value| value.execution_id.clone()),
            evidence_refs,
        });
        if skipped {
            skipped_nodes.insert(node_id.clone());
        }
    }
    ready_work.sort_by(|left, right| {
        left.topological_rank
            .cmp(&right.topological_rank)
            .then_with(|| left.node_id.cmp(&right.node_id))
    });
    let satisfied_count = nodes
        .iter()
        .filter(|node| node.posture == WorkflowNodePosture::Satisfied)
        .count();
    let skipped_count = nodes
        .iter()
        .filter(|node| node.posture == WorkflowNodePosture::Skipped)
        .count();
    let active_count = nodes
        .iter()
        .filter(|node| node.posture == WorkflowNodePosture::Active)
        .count();
    let waiting_count = nodes
        .iter()
        .filter(|node| {
            matches!(
                node.posture,
                WorkflowNodePosture::PendingDependency
                    | WorkflowNodePosture::WaitingHumanInput
                    | WorkflowNodePosture::WaitingCondition
                    | WorkflowNodePosture::WaitingEffect
            )
        })
        .count();
    let completed = satisfied_count + skipped_count == nodes.len() && active_count == 0;
    Ok(WorkflowResolution {
        schema: WORKFLOW_RESOLUTION_SCHEMA.to_string(),
        tenant_id: binding.tenant_id.clone(),
        case_id: binding.case_id.clone(),
        case_generation: state.generation,
        workflow_definition_id: definition.workflow_definition_id.clone(),
        workflow_binding_id: binding.binding_id.clone(),
        nodes,
        ready_work,
        satisfied_count,
        skipped_count,
        active_count,
        waiting_count,
        completed,
    })
}

fn validate_history_snapshot(state: &CaseState, history: &[Transition]) -> Result<(), String> {
    if history.len() as u64 != state.generation {
        return Err("workflow_history_generation_mismatch".to_string());
    }
    for (index, transition) in history.iter().enumerate() {
        if transition.case_id != state.case_id {
            return Err("workflow_history_case_mismatch".to_string());
        }
        if transition.sequence != index as u64 + 1 {
            return Err("workflow_history_sequence_invalid".to_string());
        }
    }
    Ok(())
}

pub fn evaluate_predicate(
    _definition: &WorkflowDefinition,
    binding: &CaseWorkflowBinding,
    state: &CaseState,
    history: &[Transition],
    node_id: &str,
    execution_id: Option<&str>,
    predicate: &WorkflowPredicate,
) -> Result<PredicateEvaluation, String> {
    predicate.validate()?;
    let mut evidence_refs = Vec::new();
    let value = match predicate {
        WorkflowPredicate::ExecutionProviderResult => {
            let execution_id = execution_id
                .ok_or_else(|| "workflow_execution_predicate_requires_execution".to_string())?;
            history.iter().any(|transition| {
                if !transition
                    .causal_refs
                    .iter()
                    .any(|value| value == execution_id)
                {
                    return false;
                }
                if let TransitionPayload::ProviderResultRecorded { result_id, .. } =
                    &transition.payload
                {
                    evidence_refs.push(result_id.clone());
                    true
                } else {
                    false
                }
            })
        }
        WorkflowPredicate::ExecutionEffectFinalized => {
            let execution_id = execution_id
                .ok_or_else(|| "workflow_execution_predicate_requires_execution".to_string())?;
            execution_effect_finalized(history, execution_id, None, &mut evidence_refs)
        }
        WorkflowPredicate::ExecutionFilesystemEffectFinalized { relative_path } => {
            let execution_id = execution_id
                .ok_or_else(|| "workflow_execution_predicate_requires_execution".to_string())?;
            execution_effect_finalized(
                history,
                execution_id,
                Some(relative_path),
                &mut evidence_refs,
            )
        }
        WorkflowPredicate::HumanInputRecorded => state.workflow_human_inputs.iter().any(|input| {
            if input.node_id == node_id && input.binding_id == binding.binding_id {
                evidence_refs.push(input.input_id.clone());
                true
            } else {
                false
            }
        }),
        WorkflowPredicate::CaseLifecycle { lifecycle } => {
            if &state.lifecycle == lifecycle {
                evidence_refs.push(format!(
                    "case:{}:generation:{}",
                    state.case_id, state.generation
                ));
                true
            } else {
                false
            }
        }
        WorkflowPredicate::NodeSatisfied { node_id } => {
            if let Some(satisfaction) = state
                .workflow_satisfactions
                .iter()
                .find(|satisfaction| satisfaction.node_id == *node_id)
            {
                evidence_refs.push(satisfaction.satisfaction_id.clone());
                true
            } else if let Some(input) = state
                .workflow_human_inputs
                .iter()
                .find(|input| input.node_id == *node_id)
            {
                evidence_refs.push(input.input_id.clone());
                true
            } else if let Some(condition) = state
                .workflow_conditions
                .iter()
                .find(|condition| condition.node_id == *node_id)
            {
                evidence_refs.push(condition.resolution_id.clone());
                true
            } else {
                false
            }
        }
        WorkflowPredicate::DecisionOutcome { outcome } => {
            let execution_id = execution_id
                .ok_or_else(|| "workflow_execution_predicate_requires_execution".to_string())?;
            let operation_ids = execution_operation_ids(history, execution_id);
            history.iter().any(|transition| {
                if let TransitionPayload::DecisionRecorded { decision } = &transition.payload {
                    if operation_ids.contains(&decision.operation_id)
                        && &decision.outcome == outcome
                    {
                        evidence_refs.push(decision.decision_id.clone());
                        return true;
                    }
                }
                false
            })
        }
        WorkflowPredicate::ReviewTerminal => {
            let execution_id = execution_id
                .ok_or_else(|| "workflow_execution_predicate_requires_execution".to_string())?;
            let operation_ids = execution_operation_ids(history, execution_id);
            state.reviews.iter().any(|review| {
                let linked = operation_ids.contains(&review.operation_id);
                if linked
                    && !matches!(
                        review.status,
                        ReviewResolution::Pending
                            | ReviewResolution::PendingOperator
                            | ReviewResolution::Deferred
                    )
                {
                    evidence_refs.push(review.review_id.clone());
                    true
                } else {
                    false
                }
            })
        }
        WorkflowPredicate::FinalizedEffect {
            resource_slot,
            operation_kind,
        } => {
            let attachment = resource_slot
                .as_ref()
                .and_then(|slot| binding.attachment_for_slot(slot));
            state.effects.iter().any(|effect| {
                let matches = effect.status == EffectLifecycle::Finalized
                    && attachment
                        .map(|attachment| effect.resource_attachment_id == attachment)
                        .unwrap_or(true)
                    && operation_kind
                        .as_ref()
                        .map(|kind| effect.kind == *kind)
                        .unwrap_or(true);
                if matches {
                    evidence_refs.push(effect.effect_id.clone());
                }
                matches
            })
        }
    };
    evidence_refs.sort();
    evidence_refs.dedup();
    Ok(PredicateEvaluation {
        value,
        evidence_refs,
    })
}

fn execution_effect_finalized(
    history: &[Transition],
    execution_id: &str,
    filesystem_relative_path: Option<&str>,
    evidence_refs: &mut Vec<String>,
) -> bool {
    let operation_ids = execution_operation_ids(history, execution_id);
    let effect_ids = history
        .iter()
        .filter_map(|transition| match &transition.payload {
            TransitionPayload::EffectPrepared { prepared }
                if operation_ids.contains(&prepared.operation_id)
                    && filesystem_relative_path
                        .map(|relative_path| prepared.relative_path == relative_path)
                        .unwrap_or(true) =>
            {
                Some(prepared.effect_id.clone())
            }
            TransitionPayload::ProcessEffectPrepared { prepared }
                if filesystem_relative_path.is_none()
                    && operation_ids.contains(&prepared.operation_id) =>
            {
                Some(prepared.effect_id.clone())
            }
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    history.iter().any(|transition| match &transition.payload {
        TransitionPayload::EffectFinalized {
            effect_id, receipt, ..
        } if effect_ids.contains(effect_id) => {
            evidence_refs.extend([effect_id.clone(), receipt.receipt_id.clone()]);
            true
        }
        TransitionPayload::ProcessEffectFinalized {
            effect_id, receipt, ..
        } if effect_ids.contains(effect_id) => {
            evidence_refs.extend([effect_id.clone(), receipt.receipt_id.clone()]);
            true
        }
        TransitionPayload::EffectReconciled {
            effect_id,
            receipt: Some(receipt),
            ..
        } if effect_ids.contains(effect_id) => {
            evidence_refs.extend([effect_id.clone(), receipt.receipt_id.clone()]);
            true
        }
        _ => false,
    })
}

fn execution_operation_ids(history: &[Transition], execution_id: &str) -> BTreeSet<String> {
    let provider_results = history
        .iter()
        .filter_map(|transition| {
            if !transition
                .causal_refs
                .iter()
                .any(|value| value == execution_id)
            {
                return None;
            }
            match &transition.payload {
                TransitionPayload::ProviderResultRecorded { result_id, .. } => {
                    Some(result_id.clone())
                }
                _ => None,
            }
        })
        .collect::<BTreeSet<_>>();
    history
        .iter()
        .filter_map(|transition| match &transition.payload {
            TransitionPayload::OperationRecorded { operation }
                if match &operation.origin {
                    crate::effect::OperationOrigin::ProviderResult {
                        provider_result_id, ..
                    } => provider_results.contains(provider_result_id),
                    crate::effect::OperationOrigin::WorkflowDeterministicProposal {
                        workflow_execution_id,
                        ..
                    } => workflow_execution_id == execution_id,
                    crate::effect::OperationOrigin::CompatibilityReview { .. } => false,
                } =>
            {
                Some(operation.operation_id.clone())
            }
            _ => None,
        })
        .collect()
}

pub fn node_completion_predicate(node: &WorkflowNode) -> Option<&WorkflowPredicate> {
    match &node.kind {
        WorkflowNodeKind::ModelWork { completion, .. }
        | WorkflowNodeKind::DeterministicWork { completion, .. } => Some(completion),
        WorkflowNodeKind::HumanInput { .. } => None,
        WorkflowNodeKind::Condition { predicate }
        | WorkflowNodeKind::Wait { predicate }
        | WorkflowNodeKind::EffectGoal { predicate } => Some(predicate),
    }
}

pub fn node_kind_name(kind: &WorkflowNodeKind) -> &'static str {
    match kind {
        WorkflowNodeKind::ModelWork { .. } => "model_work",
        WorkflowNodeKind::DeterministicWork { .. } => "deterministic_work",
        WorkflowNodeKind::HumanInput { .. } => "human_input",
        WorkflowNodeKind::Condition { .. } => "condition",
        WorkflowNodeKind::Wait { .. } => "wait",
        WorkflowNodeKind::EffectGoal { .. } => "effect_goal",
    }
}

fn dependency_posture(
    node_id: &str,
    definition: &WorkflowDefinition,
    satisfied: &BTreeSet<String>,
    conditions: &BTreeMap<String, bool>,
    skipped: &BTreeSet<String>,
) -> (bool, bool, String) {
    let mut incoming: Vec<(String, WorkflowEdgeKind)> = definition
        .edges
        .iter()
        .filter(|edge| edge.to == node_id)
        .map(|edge| (edge.from.clone(), edge.kind.clone()))
        .collect();
    if let Some(WorkflowPredicate::NodeSatisfied {
        node_id: dependency,
    }) = definition.node(node_id).and_then(node_completion_predicate)
    {
        if !incoming.iter().any(|(source, _)| source == dependency) {
            incoming.push((dependency.clone(), WorkflowEdgeKind::Always));
        }
    }
    incoming.sort();
    if incoming.is_empty() {
        return (true, false, "root_node".to_string());
    }
    let mut active = Vec::new();
    let mut unresolved_condition = false;
    for (source, edge_kind) in incoming {
        if skipped.contains(&source) {
            continue;
        }
        let selected = match edge_kind {
            WorkflowEdgeKind::Always => true,
            WorkflowEdgeKind::OnTrue => match conditions.get(&source) {
                Some(result) => *result,
                None => {
                    unresolved_condition = true;
                    false
                }
            },
            WorkflowEdgeKind::OnFalse => match conditions.get(&source) {
                Some(result) => !*result,
                None => {
                    unresolved_condition = true;
                    false
                }
            },
        };
        if selected {
            active.push(source);
        }
    }
    if unresolved_condition {
        return (false, false, "condition_not_resolved".to_string());
    }
    if active.is_empty() {
        return (false, true, "conditional_branch_not_selected".to_string());
    }
    if active.iter().all(|source| satisfied.contains(source)) {
        (true, false, "dependencies_satisfied".to_string())
    } else {
        (false, false, "dependency_not_satisfied".to_string())
    }
}

fn topological_order(
    nodes: &[WorkflowNode],
    edges: &[WorkflowEdge],
) -> Result<Vec<String>, String> {
    let mut indegree: BTreeMap<String, usize> = nodes
        .iter()
        .map(|node| (node.node_id.clone(), 0usize))
        .collect();
    let mut outgoing = BTreeMap::<String, Vec<String>>::new();
    let mut dependencies = edges
        .iter()
        .map(|edge| (edge.from.clone(), edge.to.clone()))
        .collect::<BTreeSet<_>>();
    for node in nodes {
        if let Some(WorkflowPredicate::NodeSatisfied { node_id }) = node_completion_predicate(node)
        {
            dependencies.insert((node_id.clone(), node.node_id.clone()));
        }
    }
    for (from, to) in dependencies {
        *indegree
            .get_mut(&to)
            .ok_or_else(|| "workflow_dangling_edge".to_string())? += 1;
        outgoing.entry(from).or_default().push(to);
    }
    for values in outgoing.values_mut() {
        values.sort();
    }
    let mut queue: VecDeque<String> = indegree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(id, _)| id.clone())
        .collect();
    let mut order = Vec::with_capacity(nodes.len());
    while let Some(id) = queue.pop_front() {
        order.push(id.clone());
        if let Some(targets) = outgoing.get(&id) {
            for target in targets {
                let degree = indegree.get_mut(target).expect("target validated");
                *degree -= 1;
                if *degree == 0 {
                    let position = queue
                        .iter()
                        .position(|existing| existing > target)
                        .unwrap_or(queue.len());
                    queue.insert(position, target.clone());
                }
            }
        }
    }
    if order.len() != nodes.len() {
        return Err("workflow_cycle_rejected".to_string());
    }
    Ok(order)
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_WORKFLOW_ID_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'-' | b'_' | b'.'))
}

fn require_slot(value: &str) -> Result<(), String> {
    if valid_id(value) {
        Ok(())
    } else {
        Err("workflow_slot_invalid".to_string())
    }
}

fn safe_relative_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 4096
        && !value.starts_with('/')
        && value
            .split('/')
            .all(|component| !component.is_empty() && component != "." && component != "..")
}

fn digest_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_vec(value)
        .map(|encoded| digest_bytes(&encoded))
        .map_err(|error| format!("workflow_digest_encode_failed: {error}"))
}

fn digest_component(digest: &str) -> &str {
    let value = digest.strip_prefix("sha256:").unwrap_or(digest);
    &value[..value.len().min(32)]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn history_for_state(state: &CaseState) -> Vec<Transition> {
        (1..=state.generation)
            .map(|sequence| Transition {
                schema: crate::transition::TRANSITION_SCHEMA.to_string(),
                transition_id: format!("transition:history:{sequence}"),
                case_id: state.case_id.clone(),
                sequence,
                committed_at_unix_ms: sequence,
                source: crate::transition::TransitionSource::component("workflow-test"),
                scope: None,
                causal_refs: Vec::new(),
                payload: if sequence == 1 {
                    TransitionPayload::CaseOpened {
                        lifecycle: CaseLifecycle::Open,
                    }
                } else {
                    TransitionPayload::ProviderResultRecorded {
                        result_id: format!("provider-result:history:{sequence}"),
                        invocation_id: format!("provider-invocation:history:{sequence}"),
                        provider_id: "provider:history".to_string(),
                        provider_kind: "fixture".to_string(),
                        model_id: "model:history".to_string(),
                        semantic_lineage: None,
                        output: "history filler".to_string(),
                    }
                },
                provenance: Vec::new(),
                summary: None,
            })
            .collect()
    }

    fn bound_state(definition: &WorkflowDefinition) -> (CaseWorkflowBinding, CaseState) {
        let binding = CaseWorkflowBinding::build(
            "tenant:test",
            "case:test",
            definition,
            vec![WorkflowExecutorBinding {
                slot: "model".to_string(),
                participant_id: "participant:model".to_string(),
            }],
            Vec::new(),
            1,
            "principal:p",
            1,
        )
        .unwrap();
        let mut state = CaseState::new("case:test", CaseLifecycle::Open);
        state.tenant_id = Some("tenant:test".to_string());
        state.workflow_binding = Some(binding.clone());
        state.generation = 1;
        (binding, state)
    }

    fn model(node_id: &str) -> WorkflowNode {
        WorkflowNode {
            node_id: node_id.to_string(),
            kind: WorkflowNodeKind::ModelWork {
                executor_slot: "model".to_string(),
                task: "produce a bounded analysis".to_string(),
                completion: WorkflowPredicate::ExecutionProviderResult,
                budgets: WorkflowBudgets::default(),
                resource_slot: None,
            },
        }
    }

    fn input(nodes: Vec<WorkflowNode>, edges: Vec<WorkflowEdge>) -> WorkflowDefinitionInput {
        WorkflowDefinitionInput {
            schema: WORKFLOW_DEFINITION_SCHEMA.to_string(),
            tenant_id: "tenant:test".to_string(),
            workflow_key: "test-flow".to_string(),
            declared_version: "1".to_string(),
            name: "Test flow".to_string(),
            description: String::new(),
            nodes,
            edges,
        }
    }

    #[test]
    fn immutable_identity_and_version_are_content_bound() {
        let first =
            WorkflowDefinition::build(input(vec![model("a")], vec![]), "principal:p", 1).unwrap();
        let same =
            WorkflowDefinition::build(input(vec![model("a")], vec![]), "principal:p", 2).unwrap();
        assert_eq!(first.workflow_definition_id, same.workflow_definition_id);
        assert_ne!(first.integrity_digest, same.integrity_digest);
        let mut v2_input = input(vec![model("a")], vec![]);
        v2_input.declared_version = "2".to_string();
        let v2 = WorkflowDefinition::build(v2_input, "principal:p", 3).unwrap();
        assert_ne!(first.workflow_definition_id, v2.workflow_definition_id);
    }

    #[test]
    fn dag_validation_rejects_cycle_and_dangling_edge() {
        let cycle = input(
            vec![model("a"), model("b")],
            vec![
                WorkflowEdge {
                    from: "a".to_string(),
                    to: "b".to_string(),
                    kind: WorkflowEdgeKind::Always,
                },
                WorkflowEdge {
                    from: "b".to_string(),
                    to: "a".to_string(),
                    kind: WorkflowEdgeKind::Always,
                },
            ],
        );
        assert_eq!(cycle.validate().unwrap_err(), "workflow_cycle_rejected");
        let dangling = input(
            vec![model("a")],
            vec![WorkflowEdge {
                from: "a".to_string(),
                to: "missing".to_string(),
                kind: WorkflowEdgeKind::Always,
            }],
        );
        assert_eq!(dangling.validate().unwrap_err(), "workflow_dangling_edge");
    }

    #[test]
    fn ready_work_is_stable_by_topological_rank_then_node_id() {
        let definition = WorkflowDefinition::build(
            input(vec![model("z-work"), model("a-work")], vec![]),
            "principal:p",
            1,
        )
        .unwrap();
        let (binding, state) = bound_state(&definition);
        let history = history_for_state(&state);
        let first = resolve_workflow(&definition, &binding, &state, &history).unwrap();
        let second = resolve_workflow(&definition, &binding, &state, &history).unwrap();
        assert_eq!(first, second);
        assert_eq!(
            first
                .ready_work
                .iter()
                .map(|work| work.node_id.as_str())
                .collect::<Vec<_>>(),
            vec!["a-work", "z-work"]
        );
    }

    #[test]
    fn frozen_condition_selects_one_branch_and_skips_the_other() {
        let condition = WorkflowNode {
            node_id: "condition".to_string(),
            kind: WorkflowNodeKind::Condition {
                predicate: WorkflowPredicate::CaseLifecycle {
                    lifecycle: CaseLifecycle::Open,
                },
            },
        };
        let definition = WorkflowDefinition::build(
            input(
                vec![condition, model("true-work"), model("false-work")],
                vec![
                    WorkflowEdge {
                        from: "condition".to_string(),
                        to: "true-work".to_string(),
                        kind: WorkflowEdgeKind::OnTrue,
                    },
                    WorkflowEdge {
                        from: "condition".to_string(),
                        to: "false-work".to_string(),
                        kind: WorkflowEdgeKind::OnFalse,
                    },
                ],
            ),
            "principal:p",
            1,
        )
        .unwrap();
        let (binding, mut state) = bound_state(&definition);
        state.workflow_conditions.push(WorkflowConditionResolution {
            schema: WORKFLOW_CONDITION_RESOLUTION_SCHEMA.to_string(),
            resolution_id: "workflow-condition:test".to_string(),
            binding_id: binding.binding_id.clone(),
            workflow_definition_id: definition.workflow_definition_id.clone(),
            node_id: "condition".to_string(),
            result: true,
            predicate_digest: WorkflowPredicate::CaseLifecycle {
                lifecycle: CaseLifecycle::Open,
            }
            .digest()
            .unwrap(),
            evaluated_at_generation: 2,
            evidence_refs: vec!["case:case:test:generation:1".to_string()],
        });
        state.generation = 3;
        let resolution =
            resolve_workflow(&definition, &binding, &state, &history_for_state(&state)).unwrap();
        assert_eq!(
            resolution
                .ready_work
                .iter()
                .map(|work| work.node_id.as_str())
                .collect::<Vec<_>>(),
            vec!["true-work"]
        );
        assert_eq!(
            resolution
                .nodes
                .iter()
                .find(|node| node.node_id == "false-work")
                .unwrap()
                .posture,
            WorkflowNodePosture::Skipped
        );
    }

    #[test]
    fn conditional_join_ignores_the_frozen_unselected_branch() {
        let condition = WorkflowNode {
            node_id: "condition".to_string(),
            kind: WorkflowNodeKind::Condition {
                predicate: WorkflowPredicate::CaseLifecycle {
                    lifecycle: CaseLifecycle::Open,
                },
            },
        };
        let definition = WorkflowDefinition::build(
            input(
                vec![
                    condition,
                    model("true-work"),
                    model("false-work"),
                    model("join"),
                ],
                vec![
                    WorkflowEdge {
                        from: "condition".to_string(),
                        to: "true-work".to_string(),
                        kind: WorkflowEdgeKind::OnTrue,
                    },
                    WorkflowEdge {
                        from: "condition".to_string(),
                        to: "false-work".to_string(),
                        kind: WorkflowEdgeKind::OnFalse,
                    },
                    WorkflowEdge {
                        from: "true-work".to_string(),
                        to: "join".to_string(),
                        kind: WorkflowEdgeKind::Always,
                    },
                    WorkflowEdge {
                        from: "false-work".to_string(),
                        to: "join".to_string(),
                        kind: WorkflowEdgeKind::Always,
                    },
                ],
            ),
            "principal:p",
            1,
        )
        .unwrap();
        let (binding, mut state) = bound_state(&definition);
        state.workflow_conditions.push(WorkflowConditionResolution {
            schema: WORKFLOW_CONDITION_RESOLUTION_SCHEMA.to_string(),
            resolution_id: "workflow-condition:test".to_string(),
            binding_id: binding.binding_id.clone(),
            workflow_definition_id: definition.workflow_definition_id.clone(),
            node_id: "condition".to_string(),
            result: true,
            predicate_digest: WorkflowPredicate::CaseLifecycle {
                lifecycle: CaseLifecycle::Open,
            }
            .digest()
            .unwrap(),
            evaluated_at_generation: 2,
            evidence_refs: vec!["case:case:test:generation:1".to_string()],
        });
        state.workflow_satisfactions.push(WorkflowNodeSatisfaction {
            schema: WORKFLOW_NODE_SATISFACTION_SCHEMA.to_string(),
            satisfaction_id: "workflow-satisfaction:true-work".to_string(),
            binding_id: binding.binding_id.clone(),
            workflow_definition_id: definition.workflow_definition_id.clone(),
            node_id: "true-work".to_string(),
            execution_id: Some("workflow-execution:true-work".to_string()),
            predicate_digest: WorkflowPredicate::ExecutionProviderResult.digest().unwrap(),
            evaluated_at_generation: 3,
            evidence_refs: vec!["provider-result:true-work".to_string()],
        });
        state.generation = 3;

        let resolution =
            resolve_workflow(&definition, &binding, &state, &history_for_state(&state)).unwrap();
        assert_eq!(
            resolution
                .ready_work
                .iter()
                .map(|work| work.node_id.as_str())
                .collect::<Vec<_>>(),
            vec!["join"]
        );
        assert_eq!(
            resolution
                .nodes
                .iter()
                .find(|node| node.node_id == "false-work")
                .unwrap()
                .posture,
            WorkflowNodePosture::Skipped
        );
    }

    #[test]
    fn provider_done_claim_cannot_satisfy_effect_completion() {
        let mut effect_model = model("effect-work");
        if let WorkflowNodeKind::ModelWork { completion, .. } = &mut effect_model.kind {
            *completion = WorkflowPredicate::ExecutionEffectFinalized;
        }
        let definition =
            WorkflowDefinition::build(input(vec![effect_model], vec![]), "principal:p", 1).unwrap();
        let (binding, mut state) = bound_state(&definition);
        let execution_id = "workflow-execution:test".to_string();
        state.workflow_executions.push(WorkflowNodeExecution {
            schema: WORKFLOW_NODE_EXECUTION_SCHEMA.to_string(),
            execution_id: execution_id.clone(),
            binding_id: binding.binding_id.clone(),
            workflow_definition_id: definition.workflow_definition_id.clone(),
            node_id: "effect-work".to_string(),
            case_id: state.case_id.clone(),
            started_at_generation: 2,
            started_at_unix_ms: 2,
        });
        state.generation = 3;
        let provider_claim = Transition {
            schema: crate::transition::TRANSITION_SCHEMA.to_string(),
            transition_id: "transition:provider-claim".to_string(),
            case_id: state.case_id.clone(),
            sequence: 3,
            committed_at_unix_ms: 3,
            source: crate::transition::TransitionSource::component("provider"),
            scope: None,
            causal_refs: vec![execution_id],
            payload: TransitionPayload::ProviderResultRecorded {
                result_id: "provider-result:done".to_string(),
                invocation_id: "provider-invocation:done".to_string(),
                provider_id: "provider:generic".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:test".to_string(),
                semantic_lineage: None,
                output: "Task completed successfully.".to_string(),
            },
            provenance: Vec::new(),
            summary: None,
        };
        let mut history = history_for_state(&state);
        history[2] = provider_claim;
        let resolution = resolve_workflow(&definition, &binding, &state, &history).unwrap();
        let node = resolution
            .nodes
            .iter()
            .find(|node| node.node_id == "effect-work")
            .unwrap();
        assert_eq!(node.posture, WorkflowNodePosture::Active);
        assert_eq!(node.reason, "workflow_execution_active");
        assert!(!resolution.completed);
    }

    #[test]
    fn execution_filesystem_completion_path_is_normalized_and_identity_bound() {
        let first = WorkflowPredicate::ExecutionFilesystemEffectFinalized {
            relative_path: "allowed/step-00.txt".to_string(),
        };
        let second = WorkflowPredicate::ExecutionFilesystemEffectFinalized {
            relative_path: "allowed/step-01.txt".to_string(),
        };
        first.validate().unwrap();
        second.validate().unwrap();
        assert_ne!(first.digest().unwrap(), second.digest().unwrap());
        assert_eq!(
            WorkflowPredicate::ExecutionFilesystemEffectFinalized {
                relative_path: "../outside.txt".to_string(),
            }
            .validate()
            .unwrap_err(),
            "parent traversal is not allowed"
        );
    }

    #[test]
    fn h15_predicate_scope_and_node_compatibility_are_mechanical() {
        let passive_execution = WorkflowNode {
            node_id: "condition".to_string(),
            kind: WorkflowNodeKind::Condition {
                predicate: WorkflowPredicate::ExecutionProviderResult,
            },
        };
        assert_eq!(
            input(vec![passive_execution], vec![])
                .validate()
                .unwrap_err(),
            "workflow_predicate_node_kind_incompatible"
        );

        let misplaced_human = WorkflowNode {
            node_id: "wait".to_string(),
            kind: WorkflowNodeKind::Wait {
                predicate: WorkflowPredicate::HumanInputRecorded,
            },
        };
        assert_eq!(
            input(vec![misplaced_human], vec![]).validate().unwrap_err(),
            "workflow_human_input_predicate_placement_invalid"
        );

        let invalid_goal = WorkflowNode {
            node_id: "goal".to_string(),
            kind: WorkflowNodeKind::EffectGoal {
                predicate: WorkflowPredicate::CaseLifecycle {
                    lifecycle: CaseLifecycle::Open,
                },
            },
        };
        assert_eq!(
            input(vec![invalid_goal], vec![]).validate().unwrap_err(),
            "workflow_predicate_node_kind_incompatible"
        );

        let valid = input(
            vec![
                model("model"),
                WorkflowNode {
                    node_id: "condition".to_string(),
                    kind: WorkflowNodeKind::Condition {
                        predicate: WorkflowPredicate::CaseLifecycle {
                            lifecycle: CaseLifecycle::Open,
                        },
                    },
                },
                WorkflowNode {
                    node_id: "wait".to_string(),
                    kind: WorkflowNodeKind::Wait {
                        predicate: WorkflowPredicate::NodeSatisfied {
                            node_id: "model".to_string(),
                        },
                    },
                },
                WorkflowNode {
                    node_id: "goal".to_string(),
                    kind: WorkflowNodeKind::EffectGoal {
                        predicate: WorkflowPredicate::FinalizedEffect {
                            resource_slot: None,
                            operation_kind: None,
                        },
                    },
                },
            ],
            vec![],
        );
        valid.validate().unwrap();
        assert_eq!(
            WorkflowPredicate::ExecutionEffectFinalized.scope(),
            WorkflowPredicateScope::Execution
        );
        assert_eq!(
            WorkflowPredicate::NodeSatisfied {
                node_id: "model".to_string()
            }
            .scope(),
            WorkflowPredicateScope::Progression
        );
    }

    #[test]
    fn h15_semantic_dependency_cycles_are_rejected_before_persistence() {
        let waits_for_b = WorkflowNode {
            node_id: "a".to_string(),
            kind: WorkflowNodeKind::Condition {
                predicate: WorkflowPredicate::NodeSatisfied {
                    node_id: "b".to_string(),
                },
            },
        };
        let hidden_cycle = input(
            vec![waits_for_b.clone(), model("b")],
            vec![WorkflowEdge {
                from: "a".to_string(),
                to: "b".to_string(),
                kind: WorkflowEdgeKind::Always,
            }],
        );
        assert_eq!(
            hidden_cycle.validate().unwrap_err(),
            "workflow_cycle_rejected"
        );

        let mut waits_for_c = waits_for_b;
        if let WorkflowNodeKind::Condition { predicate } = &mut waits_for_c.kind {
            *predicate = WorkflowPredicate::NodeSatisfied {
                node_id: "c".to_string(),
            };
        }
        let longer_cycle = input(
            vec![waits_for_c, model("b"), model("c")],
            vec![
                WorkflowEdge {
                    from: "a".to_string(),
                    to: "b".to_string(),
                    kind: WorkflowEdgeKind::Always,
                },
                WorkflowEdge {
                    from: "b".to_string(),
                    to: "c".to_string(),
                    kind: WorkflowEdgeKind::Always,
                },
            ],
        );
        assert_eq!(
            longer_cycle.validate().unwrap_err(),
            "workflow_cycle_rejected"
        );

        let valid_prior = input(
            vec![
                model("a"),
                WorkflowNode {
                    node_id: "b".to_string(),
                    kind: WorkflowNodeKind::Wait {
                        predicate: WorkflowPredicate::NodeSatisfied {
                            node_id: "a".to_string(),
                        },
                    },
                },
            ],
            vec![],
        );
        valid_prior.validate().unwrap();
        assert_eq!(
            topological_order(&valid_prior.nodes, &valid_prior.edges).unwrap(),
            vec!["a", "b"]
        );
    }

    #[test]
    fn h15_history_snapshot_rejects_generation_sequence_and_case_mismatch() {
        let definition =
            WorkflowDefinition::build(input(vec![model("a")], vec![]), "principal:p", 1).unwrap();
        let (binding, state) = bound_state(&definition);
        assert_eq!(
            resolve_workflow(&definition, &binding, &state, &[]).unwrap_err(),
            "workflow_history_generation_mismatch"
        );
        let mut history = history_for_state(&state);
        history[0].case_id = "case:other".to_string();
        assert_eq!(
            resolve_workflow(&definition, &binding, &state, &history).unwrap_err(),
            "workflow_history_case_mismatch"
        );
        let mut history = history_for_state(&state);
        history[0].sequence = 2;
        assert_eq!(
            resolve_workflow(&definition, &binding, &state, &history).unwrap_err(),
            "workflow_history_sequence_invalid"
        );
    }

    #[test]
    fn h15_definition_ordering_contract_is_explicit_and_json_keys_are_normalized() {
        let first = WorkflowDefinition::build(
            input(vec![model("a"), model("b")], vec![]),
            "principal:p",
            1,
        )
        .unwrap();
        let reordered = WorkflowDefinition::build(
            input(vec![model("b"), model("a")], vec![]),
            "principal:p",
            1,
        )
        .unwrap();
        assert_ne!(
            first.workflow_definition_id, reordered.workflow_definition_id,
            "node array order is intentionally identity-bearing in v1"
        );

        let left = r#"{"schema":"yai.workflow_definition.v1","tenant_id":"tenant:test","workflow_key":"json-order","declared_version":"1","name":"JSON order","description":"","nodes":[{"node_id":"a","kind":"model_work","executor_slot":"model","task":"analyze","completion":{"predicate":"execution_provider_result"}}],"edges":[]}"#;
        let right = r#"{"edges":[],"nodes":[{"completion":{"predicate":"execution_provider_result"},"task":"analyze","executor_slot":"model","kind":"model_work","node_id":"a"}],"description":"","name":"JSON order","declared_version":"1","workflow_key":"json-order","tenant_id":"tenant:test","schema":"yai.workflow_definition.v1"}"#;
        let left: WorkflowDefinitionInput = serde_json::from_str(left).unwrap();
        let right: WorkflowDefinitionInput = serde_json::from_str(right).unwrap();
        assert_eq!(left, right);
        let left = WorkflowDefinition::build(left, "principal:p", 1).unwrap();
        let right = WorkflowDefinition::build(right, "principal:p", 2).unwrap();
        assert_eq!(left.workflow_definition_id, right.workflow_definition_id);
    }

    #[test]
    fn h15_unknown_definition_algebra_and_ambiguous_edges_fail_closed() {
        let unknown_node = r#"{"schema":"yai.workflow_definition.v1","tenant_id":"tenant:test","workflow_key":"unknown","declared_version":"1","name":"Unknown","nodes":[{"node_id":"a","kind":"agent_node"}],"edges":[]}"#;
        assert!(serde_json::from_str::<WorkflowDefinitionInput>(unknown_node).is_err());
        let unknown_predicate = r#"{"schema":"yai.workflow_definition.v1","tenant_id":"tenant:test","workflow_key":"unknown","declared_version":"1","name":"Unknown","nodes":[{"node_id":"a","kind":"wait","predicate":{"predicate":"free_form"}}],"edges":[]}"#;
        assert!(serde_json::from_str::<WorkflowDefinitionInput>(unknown_predicate).is_err());
        let unknown_edge = r#"{"schema":"yai.workflow_definition.v1","tenant_id":"tenant:test","workflow_key":"unknown","declared_version":"1","name":"Unknown","nodes":[{"node_id":"a","kind":"condition","predicate":{"predicate":"case_lifecycle","lifecycle":"open"}},{"node_id":"b","kind":"model_work","executor_slot":"model","task":"analyze","completion":{"predicate":"execution_provider_result"}}],"edges":[{"from":"a","to":"b","kind":"sometimes"}]}"#;
        assert!(serde_json::from_str::<WorkflowDefinitionInput>(unknown_edge).is_err());

        let ambiguous = input(
            vec![
                WorkflowNode {
                    node_id: "condition".to_string(),
                    kind: WorkflowNodeKind::Condition {
                        predicate: WorkflowPredicate::CaseLifecycle {
                            lifecycle: CaseLifecycle::Open,
                        },
                    },
                },
                model("target"),
            ],
            vec![
                WorkflowEdge {
                    from: "condition".to_string(),
                    to: "target".to_string(),
                    kind: WorkflowEdgeKind::OnTrue,
                },
                WorkflowEdge {
                    from: "condition".to_string(),
                    to: "target".to_string(),
                    kind: WorkflowEdgeKind::OnFalse,
                },
            ],
        );
        assert_eq!(
            ambiguous.validate().unwrap_err(),
            "workflow_ambiguous_parallel_edge"
        );
    }

    #[test]
    fn h15_repeated_identical_provider_outputs_remain_causally_distinct() {
        let definition =
            WorkflowDefinition::build(input(vec![model("a")], vec![]), "principal:p", 1).unwrap();
        let (binding, mut state) = bound_state(&definition);
        let execution_id = "workflow-execution:a".to_string();
        state.workflow_executions.push(WorkflowNodeExecution {
            schema: WORKFLOW_NODE_EXECUTION_SCHEMA.to_string(),
            execution_id: execution_id.clone(),
            binding_id: binding.binding_id.clone(),
            workflow_definition_id: definition.workflow_definition_id.clone(),
            node_id: "a".to_string(),
            case_id: state.case_id.clone(),
            started_at_generation: 2,
            started_at_unix_ms: 2,
        });
        state.generation = 3;
        let mut history = history_for_state(&state);
        for (index, result_id) in ["provider-result:turn-1", "provider-result:turn-2"]
            .into_iter()
            .enumerate()
        {
            history[index + 1].causal_refs = vec![execution_id.clone()];
            history[index + 1].payload = TransitionPayload::ProviderResultRecorded {
                result_id: result_id.to_string(),
                invocation_id: format!("provider-invocation:turn-{}", index + 1),
                provider_id: "provider:fixture".to_string(),
                provider_kind: "fixture".to_string(),
                model_id: "model:fixture".to_string(),
                semantic_lineage: None,
                output: "identical output".to_string(),
            };
        }
        let evaluation = evaluate_predicate(
            &definition,
            &binding,
            &state,
            &history,
            "a",
            Some(&execution_id),
            &WorkflowPredicate::ExecutionProviderResult,
        )
        .unwrap();
        assert!(evaluation.value);
        assert_eq!(evaluation.evidence_refs, vec!["provider-result:turn-1"]);

        let other = evaluate_predicate(
            &definition,
            &binding,
            &state,
            &history,
            "a",
            Some("workflow-execution:other"),
            &WorkflowPredicate::ExecutionProviderResult,
        )
        .unwrap();
        assert!(!other.value);
    }

    #[test]
    fn h15_maximum_definition_scale_is_bounded_and_deterministic() {
        let nodes = (0..MAX_WORKFLOW_NODES)
            .map(|index| model(&format!("node-{index:03}")))
            .collect::<Vec<_>>();
        let mut edges = Vec::new();
        'outer: for from in 0..MAX_WORKFLOW_NODES {
            for to in (from + 1)..MAX_WORKFLOW_NODES {
                edges.push(WorkflowEdge {
                    from: format!("node-{from:03}"),
                    to: format!("node-{to:03}"),
                    kind: WorkflowEdgeKind::Always,
                });
                if edges.len() == MAX_WORKFLOW_EDGES {
                    break 'outer;
                }
            }
        }
        let scale_input = input(nodes, edges);
        let serialized_bytes = serde_json::to_vec(&scale_input).unwrap().len();
        assert!(serialized_bytes < MAX_WORKFLOW_DEFINITION_BYTES);
        let definition = WorkflowDefinition::build(scale_input, "principal:p", 1).unwrap();
        assert_eq!(definition.nodes.len(), 128);
        assert_eq!(definition.edges.len(), 512);
        let (binding, state) = bound_state(&definition);
        let history = history_for_state(&state);
        let first = resolve_workflow(&definition, &binding, &state, &history).unwrap();
        let second = resolve_workflow(&definition, &binding, &state, &history).unwrap();
        assert_eq!(first, second);

        let oversized_nodes = (0..MAX_WORKFLOW_NODES)
            .map(|index| {
                let mut node = model(&format!("node-{index:03}"));
                if let WorkflowNodeKind::ModelWork { task, .. } = &mut node.kind {
                    *task = "x".repeat(MAX_WORKFLOW_TASK_BYTES);
                }
                node
            })
            .collect();
        assert_eq!(
            input(oversized_nodes, vec![]).validate().unwrap_err(),
            "workflow_definition_aggregate_bounds_invalid"
        );
    }

    fn condition_fact(
        definition: &WorkflowDefinition,
        binding: &CaseWorkflowBinding,
        node_id: &str,
        result: bool,
        generation: u64,
    ) -> WorkflowConditionResolution {
        let predicate = match &definition.node(node_id).unwrap().kind {
            WorkflowNodeKind::Condition { predicate } => predicate,
            _ => panic!("condition fixture node kind"),
        };
        WorkflowConditionResolution {
            schema: WORKFLOW_CONDITION_RESOLUTION_SCHEMA.to_string(),
            resolution_id: format!("workflow-condition:{node_id}"),
            binding_id: binding.binding_id.clone(),
            workflow_definition_id: definition.workflow_definition_id.clone(),
            node_id: node_id.to_string(),
            result,
            predicate_digest: predicate.digest().unwrap(),
            evaluated_at_generation: generation,
            evidence_refs: vec![format!("evidence:{node_id}")],
        }
    }

    fn satisfaction_fact(
        definition: &WorkflowDefinition,
        binding: &CaseWorkflowBinding,
        node_id: &str,
        generation: u64,
    ) -> WorkflowNodeSatisfaction {
        WorkflowNodeSatisfaction {
            schema: WORKFLOW_NODE_SATISFACTION_SCHEMA.to_string(),
            satisfaction_id: format!("workflow-satisfaction:{node_id}"),
            binding_id: binding.binding_id.clone(),
            workflow_definition_id: definition.workflow_definition_id.clone(),
            node_id: node_id.to_string(),
            execution_id: Some(format!("workflow-execution:{node_id}")),
            predicate_digest: WorkflowPredicate::ExecutionProviderResult.digest().unwrap(),
            evaluated_at_generation: generation,
            evidence_refs: vec![format!("provider-result:{node_id}")],
        }
    }

    #[test]
    fn h15_nested_and_multi_condition_joins_reduce_only_selected_dependencies() {
        let condition = |node_id: &str| WorkflowNode {
            node_id: node_id.to_string(),
            kind: WorkflowNodeKind::Condition {
                predicate: WorkflowPredicate::CaseLifecycle {
                    lifecycle: CaseLifecycle::Open,
                },
            },
        };
        let definition = WorkflowDefinition::build(
            input(
                vec![
                    condition("a-outer"),
                    condition("b-inner"),
                    condition("b-skipped-condition"),
                    model("c-inner-true"),
                    model("c-inner-false"),
                    model("c-outer-false"),
                    model("z-join"),
                ],
                vec![
                    WorkflowEdge {
                        from: "a-outer".to_string(),
                        to: "b-inner".to_string(),
                        kind: WorkflowEdgeKind::OnTrue,
                    },
                    WorkflowEdge {
                        from: "a-outer".to_string(),
                        to: "b-skipped-condition".to_string(),
                        kind: WorkflowEdgeKind::OnFalse,
                    },
                    WorkflowEdge {
                        from: "b-skipped-condition".to_string(),
                        to: "c-outer-false".to_string(),
                        kind: WorkflowEdgeKind::OnTrue,
                    },
                    WorkflowEdge {
                        from: "b-inner".to_string(),
                        to: "c-inner-true".to_string(),
                        kind: WorkflowEdgeKind::OnTrue,
                    },
                    WorkflowEdge {
                        from: "b-inner".to_string(),
                        to: "c-inner-false".to_string(),
                        kind: WorkflowEdgeKind::OnFalse,
                    },
                    WorkflowEdge {
                        from: "c-inner-true".to_string(),
                        to: "z-join".to_string(),
                        kind: WorkflowEdgeKind::Always,
                    },
                    WorkflowEdge {
                        from: "c-inner-false".to_string(),
                        to: "z-join".to_string(),
                        kind: WorkflowEdgeKind::Always,
                    },
                    WorkflowEdge {
                        from: "c-outer-false".to_string(),
                        to: "z-join".to_string(),
                        kind: WorkflowEdgeKind::Always,
                    },
                ],
            ),
            "principal:p",
            1,
        )
        .unwrap();
        let (binding, mut state) = bound_state(&definition);
        state
            .workflow_conditions
            .push(condition_fact(&definition, &binding, "a-outer", true, 2));
        state
            .workflow_conditions
            .push(condition_fact(&definition, &binding, "b-inner", false, 3));
        state.workflow_satisfactions.push(satisfaction_fact(
            &definition,
            &binding,
            "c-inner-false",
            4,
        ));
        state.generation = 4;
        let resolution =
            resolve_workflow(&definition, &binding, &state, &history_for_state(&state)).unwrap();
        assert_eq!(
            resolution
                .nodes
                .iter()
                .find(|node| node.node_id == "b-skipped-condition")
                .unwrap()
                .posture,
            WorkflowNodePosture::Skipped
        );
        assert_eq!(
            resolution
                .nodes
                .iter()
                .find(|node| node.node_id == "c-inner-true")
                .unwrap()
                .posture,
            WorkflowNodePosture::Skipped
        );
        assert_eq!(
            resolution
                .ready_work
                .iter()
                .map(|work| work.node_id.as_str())
                .collect::<Vec<_>>(),
            vec!["z-join"]
        );
        assert!(!resolution.completed);
    }
}
