//! YAI - journal query helpers
//!
//! Purpose:
//!   Filter in-memory journal records for current smoke/query surfaces.
//!
//! Ownership:
//!   Lightweight record filtering over loaded journal residue.
//!
//! Boundary:
//!   Does not own LMDB query planning or future graph/fact query semantics.
//!
//! Status:
//!   active

use crate::compatibility::legacy_summary_value;
use crate::journal::Journal;
use crate::record::{Record, RecordKind};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct QueryFilter {
    pub case_ref: Option<String>,
    pub record_kind: Option<RecordKind>,
    pub subject_ref: Option<String>,
    pub decision_id: Option<String>,
    pub receipt_id: Option<String>,
    pub memory_id: Option<String>,
    pub projection_id: Option<String>,
    pub limit: Option<usize>,
    pub include_summary: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryResult {
    pub records: usize,
    pub matched: usize,
    pub returned: usize,
    pub truncated: bool,
    pub matched_records: Vec<Record>,
}

impl QueryFilter {
    pub fn matches(&self, record: &Record) -> bool {
        if let Some(case_ref) = &self.case_ref {
            if &record.case_ref != case_ref {
                return false;
            }
        }
        if let Some(record_kind) = &self.record_kind {
            if &record.kind != record_kind {
                return false;
            }
        }
        if let Some(subject_ref) = &self.subject_ref {
            if &record.subject_ref != subject_ref {
                return false;
            }
        }
        if let Some(decision_id) = &self.decision_id {
            if &record.decision_id != decision_id {
                return false;
            }
        }
        if let Some(receipt_id) = &self.receipt_id {
            if &record.receipt_id != receipt_id {
                return false;
            }
        }
        if let Some(memory_id) = &self.memory_id {
            if record.id != *memory_id
                && legacy_summary_value(&record.summary, "memory").as_ref() != Some(memory_id)
            {
                return false;
            }
        }
        if let Some(projection_id) = &self.projection_id {
            if record.id != *projection_id
                && legacy_summary_value(&record.summary, "projection").as_ref()
                    != Some(projection_id)
            {
                return false;
            }
        }
        true
    }
}

impl QueryResult {
    pub fn scan(journal: &Journal, filter: &QueryFilter) -> Self {
        let matched_records = journal
            .records()
            .iter()
            .filter(|record| filter.matches(record))
            .cloned()
            .collect::<Vec<_>>();
        let matched = matched_records.len();
        let limit = filter.limit.unwrap_or(matched);
        let returned = matched.min(limit);
        Self {
            records: journal.count(),
            matched,
            returned,
            truncated: returned < matched,
            matched_records: matched_records.into_iter().take(returned).collect(),
        }
    }
}
