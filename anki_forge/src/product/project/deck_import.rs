//! Consume a Deck once; imported content lives in the same editable state as
//! subsequent Project additions. Identity snapshots are evidence, not inputs to
//! a second lowering path.

use super::*;
use crate::deck::{Deck, DeckNote};

impl From<Deck> for Project {
    fn from(deck: Deck) -> Self {
        let mut project = Project::new(deck.name().to_string());
        project.stable_id = deck.stable_id().map(str::to_string);
        project.default_deck = Some(deck.name().to_string());
        // Legacy Deck lowering declares all three stock types in this order.
        project.imported_stock_notetypes = supported_stock_notetype_ids().to_vec();
        project.imported_note_count = deck.notes.len();
        project.import_diagnostics = match deck.validate_report() {
            Ok(report) => report
                .diagnostics()
                .iter()
                // Media registrations can be repaired after import. Reference
                // validation belongs to normalization of the current state.
                .filter(|item| item.code != crate::deck::ValidationCode::UnknownMediaRef)
                .map(deck_validation_diagnostic_to_project_diagnostic)
                .collect(),
            Err(error) => vec![Diagnostic {
                code: DiagnosticCode::new("PROJECT.DECK_VALIDATE_FAILED"),
                severity: Severity::Error,
                domain: None,
                stage: None,
                message: error.to_string(),
                source: Some(SourcePath::new("project.deck")),
                help: Some("inspect deck notes before building".into()),
            }],
        };
        project.media = crate::product::MediaRegistry::from_deck_media(deck.media);
        for source in deck.notes {
            let id = source.id().to_string();
            if let Some(snapshot) = source.resolved_identity() {
                project.imported_identities.insert(
                    id.clone(),
                    crate::update_safety::model::ResolvedNoteIdentity {
                        stable_id: id.clone(),
                        current_guid_candidate: id.clone(),
                        recipe_id: snapshot
                            .recipe_id
                            .clone()
                            .unwrap_or_else(|| "product.explicit-stable-id.v1".to_string()),
                        canonical_payload_hash: snapshot
                            .canonical_payload
                            .as_ref()
                            .map(|value| format!("blake3:{}", blake3::hash(value.as_bytes()))),
                        provenance: match snapshot.provenance {
                            crate::deck::IdentityProvenance::ExplicitStableId => "ExplicitStableId",
                            crate::deck::IdentityProvenance::InferredFromNoteFields => {
                                "InferredFromNoteFields"
                            }
                            crate::deck::IdentityProvenance::InferredFromNotetypeFields => {
                                "InferredFromNotetypeFields"
                            }
                            crate::deck::IdentityProvenance::InferredFromStockRecipe => {
                                "InferredFromStockRecipe"
                            }
                        }
                        .into(),
                        used_override: snapshot.used_override,
                    },
                );
            }
            let tags = match &source {
                DeckNote::Basic(note) => note.tags.clone(),
                DeckNote::Cloze(note) => note.tags.clone(),
                DeckNote::ImageOcclusion(note) => note.tags.clone(),
            };
            let mut note = match source {
                DeckNote::Basic(note) => Note::new(STOCK_BASIC_ID)
                    .html("Front", note.front)
                    .html("Back", note.back),
                DeckNote::Cloze(note) => Note::new(STOCK_CLOZE_ID)
                    .html("Text", note.text)
                    .html("Back Extra", note.extra),
                DeckNote::ImageOcclusion(note) => {
                    let occlusion =
                        crate::product::render_image_occlusion_cloze(note.mode, &note.rects)
                            .unwrap_or_else(|error| {
                                project.import_diagnostics.push(Diagnostic {
                                    code: DiagnosticCode::new("PROJECT.DECK_LOWER_FAILED"),
                                    severity: Severity::Error,
                                    domain: None,
                                    stage: None,
                                    message: error.to_string(),
                                    source: Some(SourcePath::new(format!("project.notes[{id:?}]"))),
                                    help: None,
                                });
                                String::new()
                            });
                    Note::new(STOCK_IMAGE_OCCLUSION_ID)
                        .html("Occlusion", occlusion)
                        .html("Image", format!("<img src=\"{}\">", note.image.name()))
                        .html("Header", note.header)
                        .html("Back Extra", note.back_extra)
                        .html("Comments", note.comments)
                }
            }
            .stable_id(id);
            for tag in tags {
                note = note.tag(tag);
            }
            project.notes.push(note);
        }
        project
    }
}
