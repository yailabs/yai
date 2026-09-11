//! Deterministic, profile-bounded extraction. No policy compiler, OCR, NER or LLM.
use super::*;
use serde_json::Value;

struct Builder<'a> {
    source: &'a KnowledgeSource,
    units: Vec<KnowledgeUnit>,
}
impl Builder<'_> {
    fn add(
        &mut self,
        parent: Option<String>,
        kind: &str,
        location: SourceLocation,
        text: String,
        posture: KnowledgePosture,
    ) -> Result<usize, String> {
        if text.len() > MAX_UNIT_BYTES || self.units.len() >= MAX_KNOWLEDGE_UNITS {
            return Err("document_unit_or_text_bound_needs_processing".into());
        }
        let id = identity(
            "knowledge-unit",
            &(
                &self.source.id,
                &self.source.extraction_id,
                &location,
                kind,
                &text,
                &posture,
            ),
        );
        self.units.push(KnowledgeUnit {
            id,
            source: self.source.id.clone(),
            parent,
            kind: kind.into(),
            location,
            text,
            posture,
            entity: None,
            predicate: None,
            value: None,
            references: vec![],
            topics: vec![],
        });
        Ok(self.units.len() - 1)
    }
    fn json(
        &mut self,
        value: &Value,
        pointer: &str,
        parent: Option<String>,
        container: Option<&SourceLocation>,
        entity: Option<&str>,
        depth: usize,
    ) -> Result<(), String> {
        if depth > 32 {
            return Err("document_json_depth_bound".into());
        }
        let explicit = value
            .get("id")
            .and_then(Value::as_str)
            .filter(|s| exact_entity(s));
        let entity = explicit.or(entity);
        let posture = if matches!(self.source.backing, SourceBacking::Observation { .. }) {
            KnowledgePosture::RecordedObservation
        } else {
            KnowledgePosture::SourceStated
        };
        let scalar = !value.is_array() && !value.is_object();
        let text = if scalar {
            format!("{} {} = {}", entity.unwrap_or("source"), pointer, value)
        } else {
            format!(
                "{} {}",
                if value.is_array() { "array" } else { "object" },
                pointer
            )
        };
        let at = self.add(
            parent,
            if scalar {
                "documentary_value"
            } else {
                "json_structure"
            },
            SourceLocation::JsonPointer {
                pointer: pointer.into(),
                container: container.cloned().map(Box::new),
            },
            text,
            if scalar {
                posture
            } else {
                KnowledgePosture::DeterministicStructure
            },
        )?;
        let unit = &mut self.units[at];
        unit.entity = entity.map(str::to_string);
        if scalar {
            unit.predicate = Some(if entity.is_some() && pointer.contains("/claims/") {
                pointer.rsplit("/claims/").next().unwrap().into()
            } else {
                pointer.into()
            });
            unit.value = Some(value.clone());
        }
        if let Some(o) = value.as_object() {
            unit.references = strings(o.get("references"));
            unit.topics = strings(o.get("topics"));
        }
        let parent = Some(self.units[at].id.clone());
        match value {
            Value::Object(o) => {
                for (k, v) in o {
                    self.json(
                        v,
                        &format!("{}/{}", pointer, k.replace('~', "~0").replace('/', "~1")),
                        parent.clone(),
                        container,
                        entity,
                        depth + 1,
                    )?;
                }
            }
            Value::Array(a) => {
                for (i, v) in a.iter().enumerate() {
                    self.json(
                        v,
                        &format!("{pointer}/{i}"),
                        parent.clone(),
                        container,
                        entity,
                        depth + 1,
                    )?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
fn exact_entity(s: &str) -> bool {
    (s.starts_with("urn:") || s.starts_with("entity:"))
        && s.len() > 5
        && s.len() <= 256
        && !s.chars().any(|c| c.is_whitespace() || c.is_control())
}
fn strings(v: Option<&Value>) -> Vec<String> {
    let mut result: Vec<_> = v
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|s| !s.is_empty() && s.len() <= 256 && !s.chars().any(char::is_control))
        .map(str::to_string)
        .collect();
    result.sort();
    result.dedup();
    result
}
fn location(at: &str) -> Result<SourceLocation, String> {
    if let Some(s) = at.strip_prefix("pdf:page=") {
        let (page, line) = s
            .split_once(":extracted-line=")
            .ok_or("pdf_coordinate_invalid")?;
        let n = line.parse().map_err(|_| "pdf_coordinate_invalid")?;
        Ok(SourceLocation::PdfExtractedLines {
            page: page.parse().map_err(|_| "pdf_coordinate_invalid")?,
            first: n,
            last: n,
        })
    } else {
        let n = at
            .strip_prefix("markdown:line=")
            .ok_or("line_coordinate_invalid")?
            .parse()
            .map_err(|_| "line_coordinate_invalid")?;
        Ok(SourceLocation::TextLines { first: n, last: n })
    }
}
fn span(first: &SourceLocation, last: &SourceLocation) -> Result<SourceLocation, String> {
    match (first, last) {
        (SourceLocation::TextLines { first, .. }, SourceLocation::TextLines { last, .. }) => {
            Ok(SourceLocation::TextLines {
                first: *first,
                last: *last,
            })
        }
        (
            SourceLocation::PdfExtractedLines { page, first, .. },
            SourceLocation::PdfExtractedLines { page: p, last, .. },
        ) if page == p => Ok(SourceLocation::PdfExtractedLines {
            page: *page,
            first: *first,
            last: *last,
        }),
        _ => Err("structured_block_cross_page_needs_processing".into()),
    }
}
pub(super) fn extract(
    source: &mut KnowledgeSource,
    bytes: &[u8],
) -> Result<Vec<KnowledgeUnit>, String> {
    let path = source.path.to_ascii_lowercase();
    let json = source.media_type == "application/json"
        || path.ends_with(".json")
        || matches!(source.backing, SourceBacking::Observation { .. });
    let pdf = source.media_type == "application/pdf" || path.ends_with(".pdf");
    let text = source.media_type.starts_with("text/")
        || [
            ".md", ".txt", ".rs", ".c", ".h", ".py", ".toml", ".yaml", ".yml",
        ]
        .iter()
        .any(|ext| path.ends_with(ext));
    if !json && !pdf && !text {
        return Err("unsupported_source_profile".into());
    }
    source.extractor = if json {
        "yai.strict_json_structure.v1"
    } else if pdf {
        "yai.text_pdf.v1:lopdf-0.44.0"
    } else {
        "yai.utf8_lines_and_structured_blocks.v1"
    }
    .into();
    source.extraction_id = identity(
        "source-extraction",
        &(&source.digest, &source.extractor, &source.media_type),
    );
    let mut b = Builder {
        source,
        units: vec![],
    };
    let root = b.add(
        None,
        "source_document",
        SourceLocation::WholeSource,
        source.path.clone(),
        KnowledgePosture::DeterministicStructure,
    )?;
    let root = Some(b.units[root].id.clone());
    if json {
        let v = crate::governance::parse_strict_json(bytes)
            .map_err(|_| "malformed_or_ambiguous_json_needs_processing")?;
        b.json(&v, "", root.clone(), None, None, 0)?;
        // SQLite metadata is the exact named-query observation, not a live DB
        // introspection by the knowledge layer. Only explicit name/sql columns.
        if source.source_kind == "sqlite_schema_observation" {
            if let (Some(columns), Some(rows)) = (v["columns"].as_array(), v["rows"].as_array()) {
                if let (Some(name), Some(sql)) = (
                    columns.iter().position(|c| c == "name"),
                    columns.iter().position(|c| c == "sql"),
                ) {
                    for (i, row) in rows.iter().enumerate() {
                        if let (Some(name), Some(sql)) = (row[name].as_str(), row[sql].as_str()) {
                            let encoded: String = name
                                .bytes()
                                .map(|b| {
                                    if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.')
                                    {
                                        (b as char).to_string()
                                    } else {
                                        format!("%{b:02X}")
                                    }
                                })
                                .collect();
                            let entity =
                                format!("urn:yai:sqlite:{}:{}", source.resource_id, encoded);
                            let n = b.add(
                                root.clone(),
                                "database_table",
                                SourceLocation::JsonPointer {
                                    pointer: format!("/rows/{i}"),
                                    container: None,
                                },
                                format!("{entity} sql = {sql}"),
                                KnowledgePosture::RecordedObservation,
                            )?;
                            b.units[n].entity = Some(entity);
                            b.units[n].predicate = Some("sql".into());
                            b.units[n].value = Some(Value::String(sql.into()));
                        }
                    }
                }
            }
        }
    } else {
        if pdf != bytes.starts_with(b"%PDF-") {
            return Err("declared_text_pdf_signature_mismatch".into());
        }
        let (_, lines) = crate::governance::extract_document_lines(bytes)
            .map_err(|_| "bounded_text_extraction_needs_processing_no_ocr")?;
        let document_parent = root.clone();
        let mut parent = root;
        let mut headings: Vec<(usize, String)> = Vec::new();
        let mut block: Option<(SourceLocation, String, &str)> = None;
        for (at, line) in lines {
            let at = location(&at)?;
            let trimmed = line.trim();
            if let Some((start, body, end)) = &mut block {
                if trimmed == *end {
                    let container = span(start, &at)?;
                    let v = crate::governance::parse_strict_json(body.as_bytes())
                        .map_err(|_| "malformed_structured_block_needs_processing")?;
                    b.json(&v, "", parent.clone(), Some(&container), None, 0)?;
                    block = None;
                } else {
                    body.push_str(&line);
                    body.push('\n');
                }
                continue;
            }
            if [
                "```yai-knowledge-json",
                "```yai-policy-json",
                "YAI-KNOWLEDGE-JSON-BEGIN",
                "YAI-POLICY-JSON-BEGIN",
            ]
            .contains(&trimmed)
            {
                block = Some((
                    at,
                    String::new(),
                    if trimmed.starts_with("```") {
                        "```"
                    } else if trimmed.starts_with("YAI-POLICY") {
                        "YAI-POLICY-JSON-END"
                    } else {
                        "YAI-KNOWLEDGE-JSON-END"
                    },
                ));
                continue;
            }
            if trimmed.is_empty() {
                continue;
            }
            // Only a bounded Markdown ATX heading profile supplies hierarchy.
            // Equal-level headings are siblings, not a chain of containment;
            // code comments/preprocessor lines are ordinary source text.
            let level = trimmed.chars().take_while(|c| *c == '#').count();
            let heading = (path.ends_with(".md") || source.media_type == "text/markdown")
                && (1..=6).contains(&level)
                && (trimmed.len() == level || trimmed.as_bytes()[level].is_ascii_whitespace());
            if heading {
                while headings.last().is_some_and(|(prior, _)| *prior >= level) {
                    headings.pop();
                }
                parent = headings
                    .last()
                    .map(|(_, id)| id.clone())
                    .or(document_parent.clone());
            }
            let n = b.add(
                parent.clone(),
                if heading { "heading" } else { "text_block" },
                at,
                line.clone(),
                if heading {
                    KnowledgePosture::DeterministicStructure
                } else {
                    KnowledgePosture::SourceStated
                },
            )?;
            if heading {
                b.units[n].topics = vec![trimmed.trim_start_matches('#').trim().into()];
                headings.push((level, b.units[n].id.clone()));
                parent = Some(b.units[n].id.clone());
            }
        }
        if block.is_some() {
            return Err("unterminated_structured_block_needs_processing".into());
        }
    }
    Ok(b.units)
}
