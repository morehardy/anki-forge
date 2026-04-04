use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectorError {
    Empty,
    ArrayIndexNotAllowed,
    InvalidPredicate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectorResolveError {
    Unmatched,
    Ambiguous,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selector {
    pub kind: String,
    pub predicates: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectorTarget {
    pub kind: String,
    pub keys: BTreeMap<String, String>,
}

impl SelectorTarget {
    pub fn new<K, V, I>(kind: impl Into<String>, pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let mut keys = BTreeMap::new();
        for (key, value) in pairs {
            keys.insert(key.into(), value.into());
        }

        Self {
            kind: kind.into(),
            keys,
        }
    }
}

pub fn parse_selector(raw: &str) -> Result<Selector, SelectorError> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(SelectorError::Empty);
    }

    if contains_array_index(raw) {
        return Err(SelectorError::ArrayIndexNotAllowed);
    }

    if raw.contains('/') {
        return Err(SelectorError::InvalidPredicate);
    }

    let Some(open_bracket) = raw.find('[') else {
        return validate_kind(raw).map(|kind| Selector {
            kind,
            predicates: Vec::new(),
        });
    };

    let kind = raw[..open_bracket].trim();
    validate_kind(kind)?;

    let predicate_block = raw[open_bracket..].trim();
    let Some(inner) = predicate_block
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return Err(SelectorError::InvalidPredicate);
    };

    if inner.trim().is_empty() {
        return Err(SelectorError::InvalidPredicate);
    }
    if inner.contains('[') || inner.contains(']') {
        return Err(SelectorError::InvalidPredicate);
    }

    let predicates = parse_predicate_list(inner)?;
    Ok(Selector {
        kind: kind.to_string(),
        predicates,
    })
}

pub fn resolve_selector(
    selector: &Selector,
    targets: &[SelectorTarget],
) -> Result<usize, SelectorResolveError> {
    let mut matched_index = None;

    for (index, target) in targets.iter().enumerate() {
        if target.kind != selector.kind {
            continue;
        }

        if !selector
            .predicates
            .iter()
            .all(|(key, value)| target.keys.get(key) == Some(value))
        {
            continue;
        }

        if matched_index.is_some() {
            return Err(SelectorResolveError::Ambiguous);
        }
        matched_index = Some(index);
    }

    matched_index.ok_or(SelectorResolveError::Unmatched)
}

fn contains_array_index(raw: &str) -> bool {
    for (open_offset, ch) in raw.char_indices() {
        if ch != '[' {
            continue;
        }

        let Some(close_offset) = raw[open_offset + 1..].find(']') else {
            break;
        };
        let inner = raw[open_offset + 1..open_offset + 1 + close_offset].trim();
        if !inner.is_empty() && inner.chars().all(|ch| ch.is_ascii_digit()) {
            return true;
        }
    }

    false
}

fn validate_kind(kind: &str) -> Result<String, SelectorError> {
    if kind.is_empty() || !is_identifier(kind) {
        return Err(SelectorError::InvalidPredicate);
    }

    Ok(kind.to_string())
}

fn parse_predicate_list(raw: &str) -> Result<Vec<(String, String)>, SelectorError> {
    let mut predicates = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;

    for ch in raw.chars() {
        match quote {
            Some(active_quote) if ch == active_quote => {
                current.push(ch);
                quote = None;
            }
            Some(_) => current.push(ch),
            None if ch == '\'' || ch == '"' => {
                quote = Some(ch);
                current.push(ch);
            }
            None if ch == ',' => {
                push_predicate(&mut predicates, &current)?;
                current.clear();
            }
            None => current.push(ch),
        }
    }

    if quote.is_some() {
        return Err(SelectorError::InvalidPredicate);
    }

    push_predicate(&mut predicates, &current)?;

    if predicates.is_empty() {
        return Err(SelectorError::InvalidPredicate);
    }

    Ok(predicates)
}

fn push_predicate(
    predicates: &mut Vec<(String, String)>,
    raw: &str,
) -> Result<(), SelectorError> {
    let item = raw.trim();
    if item.is_empty() {
        return Err(SelectorError::InvalidPredicate);
    }

    let Some((key, value)) = item.split_once('=') else {
        return Err(SelectorError::InvalidPredicate);
    };
    let key = key.trim();
    let value = value.trim();
    if !is_identifier(key) {
        return Err(SelectorError::InvalidPredicate);
    }
    let value = parse_quoted_value(value)?;

    if predicates.iter().any(|(existing_key, _)| existing_key == key) {
        return Err(SelectorError::InvalidPredicate);
    }

    predicates.push((key.to_string(), value));
    Ok(())
}

fn parse_quoted_value(raw: &str) -> Result<String, SelectorError> {
    let quote = raw.chars().next().ok_or(SelectorError::InvalidPredicate)?;
    if quote != '\'' && quote != '"' {
        return Err(SelectorError::InvalidPredicate);
    }
    if raw.len() < 2 || !raw.ends_with(quote) {
        return Err(SelectorError::InvalidPredicate);
    }

    Ok(raw[1..raw.len() - 1].to_string())
}

fn is_identifier(raw: &str) -> bool {
    let mut chars = raw.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }

    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}
