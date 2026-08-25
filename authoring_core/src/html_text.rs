pub fn strip_html_preserving_media_filenames(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut index = 0;

    while index < input.len() {
        if input[index..].starts_with("<!--") {
            if let Some(end) = input[index + 4..].find("-->") {
                index += 4 + end + 3;
                continue;
            }
        }

        let ch = input[index..]
            .chars()
            .next()
            .expect("index is within string bounds");
        if ch == '<' {
            let Some(tag_end) = find_html_tag_end(input, index) else {
                output.push(ch);
                index += ch.len_utf8();
                continue;
            };
            let tag = &input[index..=tag_end];
            if let Some((tag_name, closing)) = html_tag_name(tag) {
                if !closing && is_raw_text_html_tag(tag_name) {
                    if let Some(raw_text_end) =
                        find_raw_text_html_tag_end(input, tag_end + 1, tag_name)
                    {
                        index = raw_text_end;
                        continue;
                    }
                }
                if !closing {
                    if let Some(filename) = media_filename_from_tag(tag) {
                        output.push(' ');
                        output.push_str(&filename);
                        output.push(' ');
                    }
                }
            }
            index = tag_end + 1;
        } else {
            output.push(ch);
            index += ch.len_utf8();
        }
    }

    decode_html_entities_for_anki_text(&output)
}

fn find_html_tag_end(input: &str, start: usize) -> Option<usize> {
    let mut quote = None;
    let mut index = start + 1;
    while index < input.len() {
        let ch = input[index..].chars().next()?;
        match quote {
            Some(active_quote) if ch == active_quote => quote = None,
            Some(_) => {}
            None if ch == '"' || ch == '\'' => quote = Some(ch),
            None if ch == '>' => return Some(index),
            None => {}
        }
        index += ch.len_utf8();
    }
    None
}

fn find_raw_text_html_tag_end(input: &str, from: usize, tag_name: &str) -> Option<usize> {
    let closing_prefix = format!("</{}", tag_name.to_ascii_lowercase());
    let mut search_from = from;
    while search_from < input.len() {
        let lower_remaining = input[search_from..].to_ascii_lowercase();
        let Some(relative_start) = lower_remaining.find(&closing_prefix) else {
            break;
        };
        let close_start = search_from + relative_start;
        let Some(close_end) = find_html_tag_end(input, close_start) else {
            break;
        };
        let closing_tag = &input[close_start..=close_end];
        if let Some((closing_name, true)) = html_tag_name(closing_tag) {
            if closing_name.eq_ignore_ascii_case(tag_name) {
                return Some(close_end + 1);
            }
        }
        search_from = close_start + 2;
    }
    None
}

fn html_tag_name(tag: &str) -> Option<(&str, bool)> {
    if !tag.starts_with('<') {
        return None;
    }
    let mut index = skip_html_whitespace(tag, 1);
    let closing = tag[index..].starts_with('/');
    if closing {
        index += 1;
        index = skip_html_whitespace(tag, index);
    }
    let name_start = index;
    while index < tag.len() {
        let ch = tag[index..].chars().next()?;
        if ch.is_whitespace() || matches!(ch, '>' | '/') {
            break;
        }
        index += ch.len_utf8();
    }
    (name_start != index).then_some((&tag[name_start..index], closing))
}

fn media_filename_from_tag(tag: &str) -> Option<String> {
    let Some((tag_name, false)) = html_tag_name(tag) else {
        return None;
    };
    if !matches!(
        tag_name.to_ascii_lowercase().as_str(),
        "img" | "audio" | "video" | "source" | "object"
    ) {
        return None;
    }
    extract_html_attr(tag, "src").or_else(|| extract_html_attr(tag, "data"))
}

fn is_raw_text_html_tag(tag_name: &str) -> bool {
    tag_name.eq_ignore_ascii_case("script") || tag_name.eq_ignore_ascii_case("style")
}

fn extract_html_attr(tag: &str, attr: &str) -> Option<String> {
    let mut index = 0;
    while index < tag.len() {
        index = skip_html_whitespace(tag, index);
        if index >= tag.len() || tag.as_bytes()[index] == b'>' {
            break;
        }
        let name_start = index;
        while index < tag.len() {
            let ch = tag[index..].chars().next()?;
            if ch.is_whitespace() || matches!(ch, '=' | '>' | '/') {
                break;
            }
            index += ch.len_utf8();
        }
        if name_start == index {
            index += tag[index..].chars().next()?.len_utf8();
            continue;
        }
        let name = &tag[name_start..index];
        index = skip_html_whitespace(tag, index);
        if index >= tag.len() || tag.as_bytes()[index] != b'=' {
            continue;
        }
        index += 1;
        index = skip_html_whitespace(tag, index);
        if index >= tag.len() {
            break;
        }
        let first = tag[index..].chars().next()?;
        let raw = match first {
            '"' | '\'' => {
                let content_start = index + first.len_utf8();
                let end = tag[content_start..].find(first)?;
                index = content_start + end + first.len_utf8();
                &tag[content_start..content_start + end]
            }
            _ => {
                let value_start = index;
                while index < tag.len() {
                    let ch = tag[index..].chars().next()?;
                    if ch.is_whitespace() || ch == '>' {
                        break;
                    }
                    index += ch.len_utf8();
                }
                &tag[value_start..index]
            }
        };
        if name.eq_ignore_ascii_case(attr) {
            return Some(decode_html_entities_for_anki_text(raw));
        }
    }
    None
}

fn decode_html_entities_for_anki_text(value: &str) -> String {
    if !value.contains('&') {
        return value.to_string();
    }
    html_escape::decode_html_entities(value).replace('\u{a0}', " ")
}

fn skip_html_whitespace(input: &str, mut index: usize) -> usize {
    while index < input.len() {
        let ch = input[index..]
            .chars()
            .next()
            .expect("index is within string bounds");
        if !ch.is_whitespace() {
            break;
        }
        index += ch.len_utf8();
    }
    index
}
