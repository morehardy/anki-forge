pub mod basic;
pub mod cloze;
pub mod parser;
pub mod render_nonempty;

use crate::domain::card::CardMeta;
use crate::domain::package_spec::PackageSpec;

#[must_use]
pub fn expand_cards(spec: &PackageSpec) -> Vec<CardMeta> {
    spec.notes
        .iter()
        .enumerate()
        .map(|(note_index, _)| CardMeta {
            template_ord: u16::try_from(note_index).unwrap_or(u16::MAX),
            ..CardMeta::default()
        })
        .collect()
}
