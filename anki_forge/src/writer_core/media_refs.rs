pub fn extract_media_references(field: &str) -> Vec<String> {
    let mut refs = extract_sound_media_references(field);
    refs.extend(extract_html_media_references(field, "src"));
    refs.extend(extract_html_media_references(field, "data"));
    refs
}

fn extract_sound_media_references(field: &str) -> Vec<String> {
    let mut refs = vec![];
    let mut remaining = field;

    while let Some(start) = remaining.find("[sound:") {
        let after = &remaining[start + "[sound:".len()..];
        if let Some(end) = after.find(']') {
            refs.push(decode_html_entities(&after[..end]));
            remaining = &after[end + 1..];
        } else {
            break;
        }
    }

    refs
}

fn extract_html_media_references(field: &str, attribute: &str) -> Vec<String> {
    let mut refs = vec![];
    let marker = format!("{attribute}=");
    let mut remaining = field;

    while let Some(start) = remaining.find(&marker) {
        let after = &remaining[start + marker.len()..];
        let Some(first_char) = after.chars().next() else {
            break;
        };

        let (raw_ref, rest) = match first_char {
            '"' | '\'' => {
                let content = &after[first_char.len_utf8()..];
                if let Some(end) = content.find(first_char) {
                    (&content[..end], &content[end + first_char.len_utf8()..])
                } else {
                    break;
                }
            }
            _ => {
                let end = after
                    .find(|ch: char| ch.is_whitespace() || ch == '>')
                    .unwrap_or(after.len());
                (&after[..end], &after[end..])
            }
        };

        refs.push(decode_html_entities(raw_ref));
        remaining = rest;
    }

    refs
}

fn decode_html_entities(value: &str) -> String {
    if !value.contains('&') {
        return value.to_string();
    }

    html_escape::decode_html_entities(value).into_owned()
}
