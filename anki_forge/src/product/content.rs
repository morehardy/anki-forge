pub fn escape_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    #[test]
    fn escape_html_escapes_text_once_in_source_order() {
        assert_eq!(
            super::escape_html("AT&T <b>\"phone\"</b> 'ok'"),
            "AT&amp;T &lt;b&gt;&quot;phone&quot;&lt;/b&gt; &#39;ok&#39;"
        );
    }
}
