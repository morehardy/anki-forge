use std::collections::{BTreeMap, BTreeSet};

use unicase::UniCase;
use unicode_normalization::UnicodeNormalization;

const NATIVE_DECK_SEPARATOR: char = '\u{1f}';

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedDeck {
    pub(crate) id: i64,
    pub(crate) native_name: String,
}

impl ResolvedDeck {
    pub(crate) fn human_name(&self) -> String {
        native_deck_name_to_human(&self.native_name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeckRegistry {
    rows: Vec<ResolvedDeck>,
    indices_by_human_name: BTreeMap<String, usize>,
}

impl DeckRegistry {
    pub(crate) fn from_human_names(names: impl IntoIterator<Item = String>) -> Self {
        let native_names_by_request = names
            .into_iter()
            .map(|human_name| {
                let native_name = human_deck_name_to_native(&human_name);
                (human_name, native_name)
            })
            .collect::<BTreeMap<_, _>>();
        let requested_native_names = native_names_by_request.values().cloned().collect();

        let mut rows = vec![ResolvedDeck {
            id: 1,
            native_name: "Default".into(),
        }];
        let mut indices_by_native_name = BTreeMap::from([(UniCase::new("Default".to_string()), 0)]);
        for native_name in canonical_deck_names(&requested_native_names) {
            if native_name == "Default" {
                continue;
            }
            let index = rows.len();
            indices_by_native_name.insert(UniCase::new(native_name.clone()), index);
            rows.push(ResolvedDeck {
                id: index as i64 + 1,
                native_name,
            });
        }

        let mut indices_by_human_name = rows
            .iter()
            .enumerate()
            .map(|(index, deck)| (deck.human_name(), index))
            .collect::<BTreeMap<_, _>>();
        for (requested_name, native_name) in native_names_by_request {
            let index = indices_by_native_name[&UniCase::new(native_name)];
            indices_by_human_name.insert(requested_name, index);
        }

        Self {
            rows,
            indices_by_human_name,
        }
    }

    pub(crate) fn rows(&self) -> &[ResolvedDeck] {
        &self.rows
    }

    pub(crate) fn id_for_human_name(&self, name: &str) -> Option<i64> {
        self.deck_for_human_name(name).map(|deck| deck.id)
    }

    pub(crate) fn deck_for_human_name(&self, name: &str) -> Option<&ResolvedDeck> {
        self.indices_by_human_name
            .get(name)
            .map(|index| &self.rows[*index])
    }
}

fn canonical_deck_names(requested_names: &BTreeSet<String>) -> BTreeSet<String> {
    // Anki's SQLite unicase collation uses Unicode case folding via UniCase.
    // Reserve Default, then prefer explicitly requested spellings over implicit parents.
    let mut spellings =
        BTreeMap::from([(UniCase::new("Default".to_string()), "Default".to_string())]);
    for name in requested_names {
        spellings
            .entry(UniCase::new(name.clone()))
            .or_insert_with(|| name.clone());
    }
    for name in requested_names {
        add_native_name_and_parents(name, &mut spellings);
    }

    // Resolve parents first so every child uses the parent's chosen spelling.
    let mut names = spellings.into_values().collect::<Vec<_>>();
    names.sort_by_key(|name| name.split(NATIVE_DECK_SEPARATOR).count());
    let mut canonical_names = BTreeMap::<UniCase<String>, String>::new();
    for name in names {
        let canonical = if let Some((parent, leaf)) = name.rsplit_once(NATIVE_DECK_SEPARATOR) {
            let parent = &canonical_names[&UniCase::new(parent.to_string())];
            format!("{parent}{NATIVE_DECK_SEPARATOR}{leaf}")
        } else {
            name.clone()
        };
        canonical_names.insert(UniCase::new(name), canonical);
    }
    canonical_names.into_values().collect()
}

pub(crate) fn human_deck_name_to_native(name: &str) -> String {
    name.split("::")
        .map(normalize_deck_name_component)
        .collect::<Vec<_>>()
        .join("\u{1f}")
}

pub(crate) fn native_deck_name_to_human(name: &str) -> String {
    name.replace(NATIVE_DECK_SEPARATOR, "::")
}

fn normalize_deck_name_component(component: &str) -> String {
    let normalized = component
        .nfc()
        .filter(|character| !character.is_ascii_control())
        .collect::<String>();
    let trimmed =
        normalized.trim_matches(|character: char| character.is_whitespace() || character == ':');

    if trimmed.is_empty() {
        "blank".into()
    } else {
        trimmed.into()
    }
}

fn add_native_name_and_parents(name: &str, names: &mut BTreeMap<UniCase<String>, String>) {
    let mut current = String::new();
    for component in name.split(NATIVE_DECK_SEPARATOR) {
        if !current.is_empty() {
            current.push(NATIVE_DECK_SEPARATOR);
        }
        current.push_str(component);
        names
            .entry(UniCase::new(current.clone()))
            .or_insert_with(|| current.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_names_use_native_separator_and_component_normalization() {
        assert_eq!(human_deck_name_to_native("English"), "English");
        assert_eq!(
            human_deck_name_to_native("English::Listening"),
            "English\u{1f}Listening"
        );
        assert_eq!(
            human_deck_name_to_native(" English :::: Listening "),
            "English\u{1f}blank\u{1f}Listening"
        );
        assert_eq!(human_deck_name_to_native("Cafe\u{301}"), "Caf\u{e9}");
        assert_eq!(
            human_deck_name_to_native("fo\u{1f}o::ba\nr"),
            "foo\u{1f}bar"
        );
    }

    #[test]
    fn registry_adds_each_parent_and_resolves_requested_leaf() {
        let registry = DeckRegistry::from_human_names([
            "English::Listening::Beginner".to_string(),
            "English::Reading".to_string(),
        ]);

        let rows = registry
            .rows()
            .iter()
            .map(|deck| (deck.id, deck.native_name.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(
            rows,
            vec![
                (1, "Default"),
                (2, "English"),
                (3, "English\u{1f}Listening"),
                (4, "English\u{1f}Listening\u{1f}Beginner"),
                (5, "English\u{1f}Reading"),
            ]
        );
        assert_eq!(
            registry.id_for_human_name("English::Listening::Beginner"),
            Some(4)
        );
        assert_eq!(registry.id_for_human_name("English::Reading"), Some(5));
    }

    #[test]
    fn registry_keeps_parent_spellings_and_ids_independent_of_request_order() {
        let names = [
            "foo::child::Leaf",
            "Foo",
            "Foo::Child",
            "default::Child",
            "DEFAULT",
            " Café ::Reading",
            "Cafe\u{301}::reading",
        ]
        .map(str::to_string);
        let registry = DeckRegistry::from_human_names(names.clone());
        let reversed = DeckRegistry::from_human_names(names.into_iter().rev());
        assert_eq!(registry, reversed);
        assert_eq!(registry.id_for_human_name("DEFAULT"), Some(1));
        assert_eq!(
            registry
                .deck_for_human_name("default::Child")
                .unwrap()
                .human_name(),
            "Default::Child"
        );
        assert_eq!(
            registry
                .deck_for_human_name("foo::child::Leaf")
                .unwrap()
                .human_name(),
            "Foo::Child::Leaf"
        );
        assert_eq!(
            registry.id_for_human_name(" Café ::Reading"),
            registry.id_for_human_name("Cafe\u{301}::reading")
        );
    }
}
