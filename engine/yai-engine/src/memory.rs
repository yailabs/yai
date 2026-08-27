use crate::compatibility::legacy_summary_value;
use crate::journal::Journal;
use crate::record::RecordKind;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemorySummary {
    pub records: usize,
    pub memory_candidates: usize,
    pub operational: usize,
    pub decision: usize,
    pub subject: usize,
    pub error: usize,
    pub recovery: usize,
}

impl MemorySummary {
    pub fn from_journal(journal: &Journal) -> Self {
        let mut summary = Self {
            records: journal.count(),
            memory_candidates: 0,
            operational: 0,
            decision: 0,
            subject: 0,
            error: 0,
            recovery: 0,
        };

        for record in journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::MemoryCandidate)
        {
            summary.memory_candidates += 1;
            match legacy_summary_value(&record.summary, "memory").as_deref() {
                Some("operational") => summary.operational += 1,
                Some("decision") => summary.decision += 1,
                Some("subject") => summary.subject += 1,
                Some("error") => summary.error += 1,
                Some("recovery") => summary.recovery += 1,
                _ => {}
            }
        }

        summary
    }
}

pub fn derive_memory_note(journal: &Journal) -> String {
    format!("memory:candidate records:{}", journal.count())
}
