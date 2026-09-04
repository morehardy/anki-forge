use std::collections::{BTreeMap, BTreeSet};

use unicode_normalization::UnicodeNormalization;

const NATIVE_DECK_SEPARATOR: char = '\u{1f}';

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedDeck {
    pub(crate) id: i64,
    pub(crate) native_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeckRegistry {
    rows: Vec<ResolvedDeck>,
    ids_by_human_name: BTreeMap<String, i64>,
}

impl DeckRegistry {
    pub(crate) fn from_human_names(names: impl IntoIterator<Item = String>) -> Self {
        let requested_names: BTreeSet<_> = names.into_iter().collect();
        let mut native_names = BTreeSet::from(["Default".to_string()]);
        let mut native_names_by_request = BTreeMap::new();

        for human_name in requested_names {
            let native_name = human_deck_name_to_native(&human_name);
            add_native_name_and_parents(&native_name, &mut native_names);
            native_names_by_request.insert(human_name, native_name);
        }

        let mut ids_by_native_name = BTreeMap::from([("Default".to_string(), 1_i64)]);
        let mut next_id = 2_i64;
        for native_name in native_names {
            if native_name == "Default" {
                continue;
            }
            ids_by_native_name.insert(native_name, next_id);
            next_id += 1;
        }

        let mut rows = ids_by_native_name
            .iter()
            .map(|(native_name, id)| ResolvedDeck {
                id: *id,
                native_name: native_name.clone(),
            })
            .collect::<Vec<_>>();
        rows.sort_by_key(|deck| deck.id);

        let mut ids_by_human_name = ids_by_native_name
            .iter()
            .map(|(native_name, id)| (native_deck_name_to_human(native_name), *id))
            .collect::<BTreeMap<_, _>>();
        for (requested_name, native_name) in native_names_by_request {
            let id = ids_by_native_name[&native_name];
            ids_by_human_name.insert(requested_name, id);
        }

        Self {
            rows,
            ids_by_human_name,
        }
    }

    pub(crate) fn rows(&self) -> &[ResolvedDeck] {
        &self.rows
    }

    pub(crate) fn id_for_human_name(&self, name: &str) -> Option<i64> {
        self.ids_by_human_name.get(name).copied()
    }
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

fn add_native_name_and_parents(name: &str, names: &mut BTreeSet<String>) {
    let mut current = String::new();
    for component in name.split(NATIVE_DECK_SEPARATOR) {
        if !current.is_empty() {
            current.push(NATIVE_DECK_SEPARATOR);
        }
        current.push_str(component);
        names.insert(current.clone());
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
}
