#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphEdge {
    pub from_ref: String,
    pub to_ref: String,
    pub edge_kind: String,
}

impl GraphEdge {
    pub fn new(
        from_ref: impl Into<String>,
        to_ref: impl Into<String>,
        edge_kind: impl Into<String>,
    ) -> Self {
        Self {
            from_ref: from_ref.into(),
            to_ref: to_ref.into(),
            edge_kind: edge_kind.into(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GraphSummary {
    pub graph_edges: usize,
    pub case_binds_subject: usize,
    pub op_targets_subject: usize,
    pub decision_controls_op: usize,
    pub receipt_records_effect: usize,
    pub receipt_updates_subject: usize,
}

impl GraphSummary {
    pub fn from_summaries<'a>(summaries: impl Iterator<Item = &'a str>) -> Self {
        let mut summary = Self::default();
        for value in summaries {
            if !value.contains("edge:") {
                continue;
            }
            summary.graph_edges += 1;
            if value.contains("edge:case_binds_subject") {
                summary.case_binds_subject += 1;
            } else if value.contains("edge:op_targets_subject") {
                summary.op_targets_subject += 1;
            } else if value.contains("edge:decision_controls_op") {
                summary.decision_controls_op += 1;
            } else if value.contains("edge:receipt_records_effect") {
                summary.receipt_records_effect += 1;
            } else if value.contains("edge:receipt_updates_subject") {
                summary.receipt_updates_subject += 1;
            }
        }
        summary
    }
}
pub mod experience;
pub mod knowledge;
