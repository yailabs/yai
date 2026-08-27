use crate::compatibility::legacy_summary_value;
use crate::journal::Journal;
use crate::record::RecordKind;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReconcileSummary {
    pub records: usize,
    pub divergences: usize,
    pub reconciliations: usize,
    pub critical: usize,
    pub warnings: usize,
}

impl ReconcileSummary {
    pub fn from_journal(journal: &Journal) -> Self {
        let mut summary = Self {
            records: journal.count(),
            divergences: 0,
            reconciliations: 0,
            critical: 0,
            warnings: 0,
        };

        for record in journal.records() {
            if record.kind == RecordKind::Divergence {
                summary.divergences += 1;
                match legacy_summary_value(&record.summary, "severity").as_deref() {
                    Some("critical") => summary.critical += 1,
                    Some("warning") => summary.warnings += 1,
                    _ => {}
                }
            } else if record.kind == RecordKind::Reconciliation {
                summary.reconciliations += 1;
            }
        }

        summary
    }
}
