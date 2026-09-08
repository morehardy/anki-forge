pub fn strip_html_preserving_media_filenames(input: &str) -> String {
    // No tag or comment can close without '>'; keep literal text and entities
    // without allocating a suffix index, even for arbitrarily many '<' bytes.
    if !input.contains('>') {
        return decode_html_entities_for_anki_text(input);
    }
    let mut output = String::with_capacity(input.len());
    let mut index = 0;
    let mut scanner = HtmlScanner::default();
    let mut comments_exhausted = false;

    while index < input.len() {
        if !comments_exhausted && input[index..].starts_with("<!--") {
            let end = input[index + 4..].find("-->");
            #[cfg(test)]
            tests::record_scan(end.map_or(input.len() - index - 4, |end| end + 3));
            if let Some(end) = end {
                index += 4 + end + 3;
                continue;
            }
            comments_exhausted = true;
        }

        let ch = input[index..]
            .chars()
            .next()
            .expect("index is within string bounds");
        if ch == '<' {
            let Some(tag_end) = scanner.tag_end(input, index) else {
                output.push(ch);
                index += ch.len_utf8();
                continue;
            };
            let tag = &input[index..=tag_end];
            if let Some((tag_name, closing)) = html_tag_name(tag) {
                if !closing && is_raw_text_html_tag(tag_name) {
                    if let Some(raw_text_end) = scanner.raw_text_end(input, tag_end + 1, tag_name) {
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

/// Valid non-overlapping tags use a direct scan without allocating an index.
/// If malformed input or raw-text lookahead revisits scanned bytes, build a
/// sparse suffix index once. A query then takes O(log(number of '<' bytes));
/// quote state is preserved instead of treating every failed suffix as literal.
#[derive(Default)]
struct HtmlScanner {
    scanned_to: usize,
    indexed_ends: Option<Vec<(usize, usize)>>,
    raw_failed_through: [usize; 2],
}

impl HtmlScanner {
    fn tag_end(&mut self, input: &str, start: usize) -> Option<usize> {
        if self.indexed_ends.is_none() && start + 1 < self.scanned_to {
            // For each quote state, track the first unquoted '>' in the suffix.
            // ASCII delimiters cannot occur inside a UTF-8 continuation byte.
            let mut ends = [input.len(); 3];
            let mut index = Vec::new();
            for (offset, byte) in input.bytes().enumerate().rev() {
                #[cfg(test)]
                tests::record_scan(1);
                match byte {
                    b'>' => ends[0] = offset,
                    b'\'' => ends.swap(0, 1),
                    b'"' => ends.swap(0, 2),
                    b'<' => index.push((offset, ends[0])),
                    _ => {}
                }
            }
            index.reverse();
            self.indexed_ends = Some(index);
        }
        if let Some(index) = &self.indexed_ends {
            let entry = index
                .binary_search_by_key(&start, |&(offset, _)| offset)
                .ok()?;
            let end = index[entry].1;
            return (end < input.len()).then_some(end);
        }

        let mut quote = None;
        for (offset, byte) in input.bytes().enumerate().skip(start + 1) {
            #[cfg(test)]
            tests::record_scan(1);
            match quote {
                Some(active) if byte == active => quote = None,
                Some(_) => {}
                None if byte == b'"' || byte == b'\'' => quote = Some(byte),
                None if byte == b'>' => {
                    self.scanned_to = offset + 1;
                    return Some(offset);
                }
                None => {}
            }
        }
        self.scanned_to = input.len();
        None
    }

    fn raw_text_end(&mut self, input: &str, from: usize, tag_name: &str) -> Option<usize> {
        let (kind, prefix): (usize, &[u8]) = if tag_name.eq_ignore_ascii_case("script") {
            (0, b"</script")
        } else {
            (1, b"</style")
        };
        if from <= self.raw_failed_through[kind] {
            return None;
        }
        let mut search_from = from;
        while search_from < input.len() {
            let candidate = input[search_from..]
                .match_indices('<')
                .find(|&(relative, _)| {
                    input
                        .as_bytes()
                        .get(search_from + relative..search_from + relative + prefix.len())
                        .is_some_and(|bytes| bytes.eq_ignore_ascii_case(prefix))
                });
            let Some((relative, _)) = candidate else {
                #[cfg(test)]
                tests::record_scan(input.len() - search_from);
                self.raw_failed_through[kind] = input.len();
                return None;
            };
            #[cfg(test)]
            tests::record_scan(relative + prefix.len());
            let close_start = search_from + relative;
            let Some(close_end) = self.tag_end(input, close_start) else {
                // A later opener may begin after this malformed close. Only
                // cache the interval actually ruled out by the legacy parser.
                self.raw_failed_through[kind] = close_start;
                return None;
            };
            let delimiter = input[close_start + prefix.len()..].chars().next();
            if delimiter.is_some_and(|ch| ch.is_whitespace() || matches!(ch, '>' | '/')) {
                return Some(close_end + 1);
            }
            search_from = close_start + 2;
        }
        self.raw_failed_through[kind] = input.len();
        None
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    thread_local! {
        static SCANNED_BYTES: Cell<usize> = const { Cell::new(0) };
    }

    pub(super) fn record_scan(bytes: usize) {
        SCANNED_BYTES.with(|count| count.set(count.get() + bytes));
    }

    #[test]
    fn malformed_html_keeps_quote_comment_and_raw_text_boundaries() {
        for (input, expected) in [
            (r#"<"prefix <b>tail"#, r#"<"prefix tail"#),
            ("a<!-- unclosed <b>front</b>", "afront"),
            ("<script>keep</scriptx>tail", "keeptail"),
            (
                r#"<script>A</script" <script>B</script>C"#,
                r#"A</script" C"#,
            ),
            ("<style>x</styleX <z></STYLE>tail", "tail"),
            ("<script>…</SCRIPT / >end", "end"),
            ("<img title='>' src='a&#46;png'>尾", " a.png 尾"),
        ] {
            assert_eq!(
                strip_html_preserving_media_filenames(input),
                expected,
                "{input:?}"
            );
        }
    }

    #[test]
    fn unclosed_tags_do_not_rescan_each_remaining_suffix() {
        for input in ["<".repeat(4096), format!("{}\">", "<".repeat(4096))] {
            SCANNED_BYTES.with(|count| count.set(0));
            assert_eq!(strip_html_preserving_media_filenames(&input), input);
            let scanned = SCANNED_BYTES.with(Cell::get);
            assert!(
                scanned <= input.len() * 4,
                "scanned {scanned} bytes for {} input bytes",
                input.len()
            );
        }
    }

    #[test]
    fn repeated_comments_and_raw_text_near_misses_have_bounded_scans() {
        for input in [
            "<!--".repeat(1024),
            "<script>".repeat(1024),
            "<style>".repeat(1024),
            format!("<script>{}</script>end", "</scriptx ".repeat(1024)),
        ] {
            SCANNED_BYTES.with(|count| count.set(0));
            strip_html_preserving_media_filenames(&input);
            let scanned = SCANNED_BYTES.with(Cell::get);
            assert!(
                scanned <= input.len() * 8,
                "scanned {scanned} bytes for {} input bytes",
                input.len()
            );
        }
    }
}
