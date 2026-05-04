use html_escape::decode_html_entities;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaReferenceCandidate {
    pub owner_kind: String,
    pub owner_id: String,
    pub location_kind: String,
    pub location_name: String,
    pub raw_ref: String,
    pub ref_kind: String,
    pub normalized_local_ref: Option<String>,
    pub skip_reason: Option<String>,
    pub unsafe_reason: Option<String>,
    pub kind: MediaReferenceCandidateKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaReferenceCandidateKind {
    Sound,
    HtmlSrc,
    HtmlObjectData,
    CssUrl,
}

pub fn extract_media_reference_candidates(
    owner_kind: &str,
    owner_id: &str,
    location_kind: &str,
    location_name: &str,
    input: &str,
) -> Vec<MediaReferenceCandidate> {
    let input = strip_html_comments(input);
    let mut refs = Vec::new();

    refs.extend(extract_sound_refs(
        owner_kind,
        owner_id,
        location_kind,
        location_name,
        &input,
    ));
    refs.extend(extract_html_src_refs(
        owner_kind,
        owner_id,
        location_kind,
        location_name,
        &input,
    ));
    refs.extend(extract_html_object_data_refs(
        owner_kind,
        owner_id,
        location_kind,
        location_name,
        &input,
    ));
    refs.extend(extract_css_url_refs(
        owner_kind,
        owner_id,
        location_kind,
        location_name,
        &input,
    ));

    refs
}

fn strip_html_comments(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0;

    while let Some(comment_start) = input[cursor..].find("<!--") {
        let absolute_start = cursor + comment_start;
        output.push_str(&input[cursor..absolute_start]);
        let comment_body_start = absolute_start + "<!--".len();
        match input[comment_body_start..].find("-->") {
            Some(comment_end) => {
                cursor = comment_body_start + comment_end + "-->".len();
            }
            None => {
                cursor = input.len();
                break;
            }
        }
    }

    output.push_str(&input[cursor..]);
    output
}

fn extract_sound_refs(
    owner_kind: &str,
    owner_id: &str,
    location_kind: &str,
    location_name: &str,
    input: &str,
) -> Vec<MediaReferenceCandidate> {
    let mut refs = Vec::new();
    let mut cursor = 0;

    while let Some(start) = input[cursor..].find("[sound:") {
        let value_start = cursor + start + "[sound:".len();
        let Some(relative_end) = input[value_start..].find(']') else {
            break;
        };
        let value_end = value_start + relative_end;
        let raw_ref = decode_html_entities(input[value_start..value_end].trim()).to_string();
        refs.push(local_candidate(
            owner_kind,
            owner_id,
            location_kind,
            location_name,
            &raw_ref,
            MediaReferenceCandidateKind::Sound,
            false,
        ));
        cursor = value_end + 1;
    }

    refs
}

fn extract_html_src_refs(
    owner_kind: &str,
    owner_id: &str,
    location_kind: &str,
    location_name: &str,
    input: &str,
) -> Vec<MediaReferenceCandidate> {
    extract_html_tag_attribute_refs(
        owner_kind,
        owner_id,
        location_kind,
        location_name,
        input,
        "src",
        MediaReferenceCandidateKind::HtmlSrc,
    )
}

fn extract_html_object_data_refs(
    owner_kind: &str,
    owner_id: &str,
    location_kind: &str,
    location_name: &str,
    input: &str,
) -> Vec<MediaReferenceCandidate> {
    let mut refs = Vec::new();
    let mut cursor = 0;

    while let Some(tag) = next_html_start_tag(input, cursor) {
        if tag.name.eq_ignore_ascii_case("object") {
            refs.extend(extract_html_attribute_refs(
                owner_kind,
                owner_id,
                location_kind,
                location_name,
                tag.source,
                "data",
                MediaReferenceCandidateKind::HtmlObjectData,
            ));
        }
        cursor = next_html_scan_cursor(input, &tag);
    }

    refs
}

fn extract_css_url_refs(
    owner_kind: &str,
    owner_id: &str,
    location_kind: &str,
    location_name: &str,
    input: &str,
) -> Vec<MediaReferenceCandidate> {
    let mut refs = Vec::new();
    let input = strip_html_raw_text_elements(input, "script");
    let input = strip_css_block_comments(&input);
    let mut cursor = 0;

    while let Some(url_start) = find_css_url_function(&input, cursor) {
        if !is_css_url_boundary(input.as_bytes(), url_start) {
            cursor = url_start + "url(".len();
            continue;
        }

        let value_start = url_start + "url(".len();
        let Some((raw_ref, next_cursor)) = parse_css_url_value(&input, value_start) else {
            cursor = value_start;
            continue;
        };
        refs.push(local_candidate(
            owner_kind,
            owner_id,
            location_kind,
            location_name,
            &raw_ref,
            MediaReferenceCandidateKind::CssUrl,
            true,
        ));
        cursor = next_cursor;
    }

    refs
}

fn extract_html_tag_attribute_refs(
    owner_kind: &str,
    owner_id: &str,
    location_kind: &str,
    location_name: &str,
    input: &str,
    attr_name: &str,
    kind: MediaReferenceCandidateKind,
) -> Vec<MediaReferenceCandidate> {
    let mut refs = Vec::new();
    let mut cursor = 0;

    while let Some(tag) = next_html_start_tag(input, cursor) {
        refs.extend(extract_html_attribute_refs(
            owner_kind,
            owner_id,
            location_kind,
            location_name,
            tag.source,
            attr_name,
            kind,
        ));
        cursor = next_html_scan_cursor(input, &tag);
    }

    refs
}

fn extract_html_attribute_refs(
    owner_kind: &str,
    owner_id: &str,
    location_kind: &str,
    location_name: &str,
    input: &str,
    attr_name: &str,
    kind: MediaReferenceCandidateKind,
) -> Vec<MediaReferenceCandidate> {
    let mut refs = Vec::new();
    let mut cursor = html_start_tag_attribute_cursor(input);
    let bytes = input.as_bytes();

    while cursor < bytes.len() {
        cursor = skip_ascii_whitespace(input, cursor);
        let Some(first) = bytes.get(cursor) else {
            break;
        };
        if *first == b'>' {
            break;
        }
        if *first == b'/' {
            cursor += 1;
            continue;
        }
        if !is_html_attribute_name_byte(*first) {
            cursor += 1;
            continue;
        }

        let name_start = cursor;
        while cursor < bytes.len() && is_html_attribute_name_byte(bytes[cursor]) {
            cursor += 1;
        }
        let name = &input[name_start..cursor];
        let mut value_start = skip_ascii_whitespace(input, cursor);
        if bytes.get(value_start) != Some(&b'=') {
            cursor = value_start;
            continue;
        }
        value_start = skip_ascii_whitespace(input, value_start + 1);

        let Some((raw_value, next_cursor)) = parse_html_attribute_value(input, value_start) else {
            break;
        };
        if name.eq_ignore_ascii_case(attr_name) {
            let raw_ref = decode_html_entities(raw_value.trim()).to_string();
            refs.push(local_candidate(
                owner_kind,
                owner_id,
                location_kind,
                location_name,
                &raw_ref,
                kind,
                true,
            ));
        }
        cursor = next_cursor;
    }

    refs
}

fn is_html_attribute_boundary(bytes: &[u8], start: usize, end: usize) -> bool {
    let before_ok = start == 0 || !is_html_attribute_name_byte(bytes[start - 1]);
    let after_ok = end >= bytes.len() || !is_html_attribute_name_byte(bytes[end]);
    before_ok && after_ok
}

fn is_css_url_boundary(bytes: &[u8], start: usize) -> bool {
    start == 0 || !is_css_identifier_byte(bytes[start - 1])
}

fn local_candidate(
    owner_kind: &str,
    owner_id: &str,
    location_kind: &str,
    location_name: &str,
    raw_ref: &str,
    kind: MediaReferenceCandidateKind,
    url_semantics: bool,
) -> MediaReferenceCandidate {
    let raw_ref = raw_ref.trim().to_string();
    let ref_kind = kind.ref_kind().to_string();
    let (normalized_local_ref, skip_reason, unsafe_reason) =
        match classify_ref(&raw_ref, url_semantics) {
            ReferenceClassification::Local(value) => (Some(value), None, None),
            ReferenceClassification::Skipped(reason) => (None, Some(reason.to_string()), None),
            ReferenceClassification::Unsafe(reason) => (None, None, Some(reason.to_string())),
        };

    MediaReferenceCandidate {
        owner_kind: owner_kind.to_string(),
        owner_id: owner_id.to_string(),
        location_kind: location_kind.to_string(),
        location_name: location_name.to_string(),
        raw_ref,
        ref_kind,
        normalized_local_ref,
        skip_reason,
        unsafe_reason,
        kind,
    }
}

fn classify_ref(raw_ref: &str, url_semantics: bool) -> ReferenceClassification {
    let trimmed = raw_ref.trim();
    if trimmed.is_empty() {
        return ReferenceClassification::Skipped("empty-ref");
    }
    if contains_dynamic_template(trimmed) {
        return ReferenceClassification::Skipped("dynamic-template");
    }
    if trimmed.starts_with("//") {
        return ReferenceClassification::Skipped("protocol-relative-url");
    }
    if starts_with_ascii_case_insensitive(trimmed, "data:") {
        return ReferenceClassification::Skipped("data-uri");
    }
    if has_url_scheme(trimmed) {
        return ReferenceClassification::Skipped("external-url");
    }

    let local_ref = if url_semantics {
        let url_path = strip_url_query_and_fragment(trimmed);
        match percent_decode_utf8(url_path) {
            Ok(value) => value,
            Err(reason) => return ReferenceClassification::Unsafe(reason),
        }
    } else {
        trimmed.to_string()
    };

    if url_semantics && local_ref.is_empty() {
        return ReferenceClassification::Unsafe("decoded-empty-path");
    }
    if url_semantics && matches!(local_ref.as_str(), "." | "..") {
        return ReferenceClassification::Unsafe("decoded-dot-path");
    }
    if local_ref.is_empty() {
        return ReferenceClassification::Skipped("empty-ref");
    }
    if local_ref.contains(['/', '\\']) {
        return ReferenceClassification::Unsafe("decoded-path-separator");
    }

    ReferenceClassification::Local(local_ref)
}

fn percent_decode_utf8(input: &str) -> Result<String, &'static str> {
    let bytes = input.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut cursor = 0;

    while cursor < bytes.len() {
        if bytes[cursor] == b'%' {
            let Some(high) = bytes.get(cursor + 1).and_then(|byte| hex_value(*byte)) else {
                return Err("invalid-percent-encoding");
            };
            let Some(low) = bytes.get(cursor + 2).and_then(|byte| hex_value(*byte)) else {
                return Err("invalid-percent-encoding");
            };
            decoded.push(high << 4 | low);
            cursor += 3;
        } else {
            decoded.push(bytes[cursor]);
            cursor += 1;
        }
    }

    String::from_utf8(decoded).map_err(|_| "invalid-percent-encoding")
}

enum ReferenceClassification {
    Local(String),
    Skipped(&'static str),
    Unsafe(&'static str),
}

impl MediaReferenceCandidateKind {
    fn ref_kind(self) -> &'static str {
        match self {
            MediaReferenceCandidateKind::Sound => "sound",
            MediaReferenceCandidateKind::HtmlSrc => "html_src",
            MediaReferenceCandidateKind::HtmlObjectData => "html_object_data",
            MediaReferenceCandidateKind::CssUrl => "css_url",
        }
    }
}

fn parse_html_attribute_value(input: &str, value_start: usize) -> Option<(&str, usize)> {
    let bytes = input.as_bytes();
    let first = *bytes.get(value_start)?;

    if first == b'"' || first == b'\'' {
        let quote = first;
        let mut cursor = value_start + 1;
        while cursor < bytes.len() {
            if bytes[cursor] == quote {
                return Some((&input[value_start + 1..cursor], cursor + 1));
            }
            cursor += 1;
        }
        return Some((&input[value_start + 1..], input.len()));
    }

    let mut cursor = value_start;
    while cursor < bytes.len() {
        let byte = bytes[cursor];
        if byte.is_ascii_whitespace() || byte == b'>' {
            break;
        }
        cursor += 1;
    }
    Some((&input[value_start..cursor], cursor))
}

fn html_start_tag_attribute_cursor(input: &str) -> usize {
    let bytes = input.as_bytes();
    let mut cursor = usize::from(bytes.first() == Some(&b'<'));
    while cursor < bytes.len() && is_html_tag_name_byte(bytes[cursor]) {
        cursor += 1;
    }
    cursor
}

fn parse_css_url_value(input: &str, value_start: usize) -> Option<(String, usize)> {
    let bytes = input.as_bytes();
    let mut cursor = skip_ascii_whitespace(input, value_start);
    let first = *bytes.get(cursor)?;

    if first == b'"' || first == b'\'' {
        let quote = first;
        cursor += 1;
        let raw_start = cursor;
        while cursor < bytes.len() {
            if bytes[cursor] == b'\\' {
                cursor = cursor.saturating_add(2);
                continue;
            }
            if bytes[cursor] == quote {
                let raw_ref = input[raw_start..cursor].to_string();
                cursor = skip_ascii_whitespace(input, cursor + 1);
                if bytes.get(cursor) == Some(&b')') {
                    return Some((raw_ref, cursor + 1));
                }
                return None;
            }
            cursor += 1;
        }
        return None;
    }

    let raw_start = cursor;
    while cursor < bytes.len() && bytes[cursor] != b')' {
        cursor += 1;
    }
    if cursor >= bytes.len() {
        return None;
    }

    Some((input[raw_start..cursor].trim().to_string(), cursor + 1))
}

struct HtmlStartTag<'a> {
    name: &'a str,
    source: &'a str,
    start: usize,
    end: usize,
}

fn next_html_start_tag(input: &str, cursor: usize) -> Option<HtmlStartTag<'_>> {
    let bytes = input.as_bytes();
    let mut cursor = cursor;

    while let Some(relative_tag_start) = input[cursor..].find('<') {
        let tag_start = cursor + relative_tag_start;
        let tag_name_start = tag_start + 1;
        let Some(first_tag_name_byte) = bytes.get(tag_name_start) else {
            break;
        };
        if !first_tag_name_byte.is_ascii_alphabetic() {
            cursor = tag_name_start + 1;
            continue;
        }

        let mut tag_name_end = tag_name_start + 1;
        while tag_name_end < bytes.len() && is_html_tag_name_byte(bytes[tag_name_end]) {
            tag_name_end += 1;
        }

        let Some(tag_end) = find_html_tag_end(input, tag_name_end) else {
            break;
        };
        return Some(HtmlStartTag {
            name: &input[tag_name_start..tag_name_end],
            source: &input[tag_start..=tag_end],
            start: tag_start,
            end: tag_end,
        });
    }

    None
}

fn next_html_scan_cursor(input: &str, tag: &HtmlStartTag<'_>) -> usize {
    if is_html_raw_text_tag(tag.name) && !is_self_closing_start_tag(tag.source) {
        find_html_raw_text_end(input, tag.end + 1, tag.name)
    } else {
        tag.end + 1
    }
}

fn find_html_raw_text_end(input: &str, start: usize, tag_name: &str) -> usize {
    let close_prefix = format!("</{tag_name}");
    let mut cursor = start;

    while let Some(close_start) = find_ascii_case_insensitive(input, &close_prefix, cursor) {
        let name_start = close_start + "</".len();
        let name_end = name_start + tag_name.len();
        if is_html_attribute_boundary(input.as_bytes(), name_start, name_end) {
            return find_html_tag_end(input, name_end).map_or(input.len(), |tag_end| tag_end + 1);
        }
        cursor = name_end;
    }

    input.len()
}

fn is_html_raw_text_tag(name: &str) -> bool {
    name.eq_ignore_ascii_case("script") || name.eq_ignore_ascii_case("style")
}

fn is_self_closing_start_tag(source: &str) -> bool {
    source
        .strip_suffix('>')
        .unwrap_or(source)
        .trim_end()
        .ends_with('/')
}

fn strip_html_raw_text_elements(input: &str, tag_name: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut scan_cursor = 0;
    let mut copy_cursor = 0;

    while let Some(tag) = next_html_start_tag(input, scan_cursor) {
        if tag.name.eq_ignore_ascii_case(tag_name) && !is_self_closing_start_tag(tag.source) {
            output.push_str(&input[copy_cursor..tag.start]);
            copy_cursor = find_html_raw_text_end(input, tag.end + 1, tag.name);
            scan_cursor = copy_cursor;
        } else {
            scan_cursor = tag.end + 1;
        }
    }

    output.push_str(&input[copy_cursor..]);
    output
}

fn find_html_tag_end(input: &str, start: usize) -> Option<usize> {
    let bytes = input.as_bytes();
    let mut cursor = start;
    let mut quote = None;

    while cursor < bytes.len() {
        let byte = bytes[cursor];
        if let Some(active_quote) = quote {
            if byte == active_quote {
                quote = None;
            }
        } else if byte == b'"' || byte == b'\'' {
            quote = Some(byte);
        } else if byte == b'>' {
            return Some(cursor);
        }
        cursor += 1;
    }

    None
}

fn find_ascii_case_insensitive(input: &str, needle: &str, start: usize) -> Option<usize> {
    if needle.is_empty() || start >= input.len() {
        return None;
    }

    let haystack = input.as_bytes();
    let needle = needle.as_bytes();
    if needle.len() > haystack.len() {
        return None;
    }

    (start..=haystack.len() - needle.len()).find(|candidate| {
        haystack[*candidate..*candidate + needle.len()].eq_ignore_ascii_case(needle)
    })
}

fn find_css_url_function(input: &str, start: usize) -> Option<usize> {
    let bytes = input.as_bytes();
    let mut cursor = start;
    let mut quote = None;
    let needle = b"url(";

    while cursor < bytes.len() {
        if let Some(active_quote) = quote {
            if bytes[cursor] == b'\\' {
                cursor = cursor.saturating_add(2);
                continue;
            }
            if bytes[cursor] == active_quote {
                quote = None;
            }
            cursor += 1;
            continue;
        }

        if bytes[cursor] == b'"' || bytes[cursor] == b'\'' {
            quote = Some(bytes[cursor]);
            cursor += 1;
            continue;
        }
        if bytes
            .get(cursor..cursor + needle.len())
            .is_some_and(|candidate| candidate.eq_ignore_ascii_case(needle))
        {
            return Some(cursor);
        }
        cursor += 1;
    }

    None
}

fn skip_ascii_whitespace(input: &str, start: usize) -> usize {
    let bytes = input.as_bytes();
    let mut cursor = start;
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    cursor
}

fn strip_url_query_and_fragment(input: &str) -> &str {
    let query = input.find('?');
    let fragment = input.find('#');
    match (query, fragment) {
        (Some(left), Some(right)) => &input[..left.min(right)],
        (Some(index), None) | (None, Some(index)) => &input[..index],
        (None, None) => input,
    }
}

fn strip_css_block_comments(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0;

    while let Some(comment_start) = input[cursor..].find("/*") {
        let absolute_start = cursor + comment_start;
        output.push_str(&input[cursor..absolute_start]);
        let comment_body_start = absolute_start + "/*".len();
        match input[comment_body_start..].find("*/") {
            Some(comment_end) => {
                cursor = comment_body_start + comment_end + "*/".len();
            }
            None => {
                cursor = input.len();
                break;
            }
        }
    }

    output.push_str(&input[cursor..]);
    output
}

fn contains_dynamic_template(input: &str) -> bool {
    input.contains("{{")
        || input.contains("}}")
        || input.contains("{%")
        || input.contains("%}")
        || input.contains("${")
        || input.contains("<%")
        || input.contains("%>")
}

fn has_url_scheme(input: &str) -> bool {
    let Some(colon) = input.find(':') else {
        return false;
    };
    let scheme = &input[..colon];
    let mut chars = scheme.bytes();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_alphabetic()
        && chars.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'.' | b'-'))
}

fn starts_with_ascii_case_insensitive(input: &str, prefix: &str) -> bool {
    input
        .as_bytes()
        .get(..prefix.len())
        .is_some_and(|value| value.eq_ignore_ascii_case(prefix.as_bytes()))
}

fn is_html_attribute_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'_' | b'-')
}

fn is_html_tag_name_byte(byte: u8) -> bool {
    is_html_attribute_name_byte(byte)
}

fn is_css_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
