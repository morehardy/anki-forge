use authoring_core::{extract_media_reference_candidates, MediaReferenceCandidateKind};

type ReferenceSummary<'a> = (
    &'a str,
    &'a str,
    Option<&'a str>,
    Option<&'a str>,
    Option<&'a str>,
);

#[test]
fn extracts_sound_html_object_and_css_refs() {
    let refs = scan(
        r#"
        [sound:bell.mp3]
        <img src="hero.png">
        <source src='clip.webm'>
        <object data=diagram.svg></object>
        <style>.card { background-image: url("bg%20one.png?version=1#frag"); }</style>
        "#,
    );

    assert_eq!(
        ref_summaries(&refs),
        vec![
            ("sound", "bell.mp3", Some("bell.mp3"), None, None),
            ("html_src", "hero.png", Some("hero.png"), None, None),
            ("html_src", "clip.webm", Some("clip.webm"), None, None),
            (
                "html_object_data",
                "diagram.svg",
                Some("diagram.svg"),
                None,
                None,
            ),
            (
                "css_url",
                "bg%20one.png?version=1#frag",
                Some("bg one.png"),
                None,
                None,
            ),
        ]
    );
    assert_eq!(
        refs.iter().map(|item| item.kind).collect::<Vec<_>>(),
        vec![
            MediaReferenceCandidateKind::Sound,
            MediaReferenceCandidateKind::HtmlSrc,
            MediaReferenceCandidateKind::HtmlSrc,
            MediaReferenceCandidateKind::HtmlObjectData,
            MediaReferenceCandidateKind::CssUrl,
        ]
    );
}

#[test]
fn classifies_external_and_data_uri_as_skipped() {
    let refs = scan(
        r#"
        [sound:https://example.test/bell.mp3]
        <img src="//cdn.example.test/hero.png">
        <img src="data:image/png;base64,AAAA">
        <object data="{{ dynamic_media }}"></object>
        <style>.x { background: url(mailto:media@example.test); }</style>
        "#,
    );

    assert_eq!(
        ref_summaries(&refs),
        vec![
            (
                "sound",
                "https://example.test/bell.mp3",
                None,
                Some("external-url"),
                None,
            ),
            (
                "html_src",
                "//cdn.example.test/hero.png",
                None,
                Some("protocol-relative-url"),
                None,
            ),
            (
                "html_src",
                "data:image/png;base64,AAAA",
                None,
                Some("data-uri"),
                None,
            ),
            (
                "html_object_data",
                "{{ dynamic_media }}",
                None,
                Some("dynamic-template"),
                None,
            ),
            (
                "css_url",
                "mailto:media@example.test",
                None,
                Some("external-url"),
                None,
            ),
        ]
    );
}

#[test]
fn percent_decodes_local_url_path_and_rejects_decoded_separators() {
    let refs = scan(
        r#"
        <img src="space%20name.png?cache=1#front">
        <img src="folder%2Fescape.png">
        <style>.x { background: url(back%5Cslash.png); }</style>
        "#,
    );

    assert_eq!(
        ref_summaries(&refs),
        vec![
            (
                "html_src",
                "space%20name.png?cache=1#front",
                Some("space name.png"),
                None,
                None,
            ),
            (
                "html_src",
                "folder%2Fescape.png",
                None,
                None,
                Some("decoded-path-separator"),
            ),
            (
                "css_url",
                "back%5Cslash.png",
                None,
                None,
                Some("decoded-path-separator"),
            ),
        ]
    );
}

#[test]
fn sound_refs_do_not_use_url_percent_decoding() {
    let refs = scan("[sound:space%20name.mp3] [sound:folder%2Fescape.mp3]");

    assert_eq!(
        ref_summaries(&refs),
        vec![
            (
                "sound",
                "space%20name.mp3",
                Some("space%20name.mp3"),
                None,
                None,
            ),
            (
                "sound",
                "folder%2Fescape.mp3",
                Some("folder%2Fescape.mp3"),
                None,
                None,
            ),
        ]
    );
}

#[test]
fn html_refs_handle_entities_case_unquoted_attributes_and_comments() {
    let refs = scan(
        r#"
        <!-- [sound:ignored.mp3] <img src="ignored.png"> -->
        <IMG SRC=hero&amp;icon.png>
        <object DATA=diagram&amp;v.svg></object>
        <img data-src="ignored-data-attr.png">
        <img xsrc="ignored-boundary.png">
        "#,
    );

    assert_eq!(
        ref_summaries(&refs),
        vec![
            (
                "html_src",
                "hero&icon.png",
                Some("hero&icon.png"),
                None,
                None,
            ),
            (
                "html_object_data",
                "diagram&v.svg",
                Some("diagram&v.svg"),
                None,
                None,
            ),
        ]
    );
}

#[test]
fn css_url_refs_require_url_function_boundary() {
    let refs = scan(
        r#"
        .fake { background: myurl(fake.png); }
        .fake2 { background: my-url(fake2.png); }
        .real { background: url(real.png); }
        .nested { background-image: image-set(url("set.png") 1x); }
        "#,
    );

    assert_eq!(
        ref_summaries(&refs),
        vec![
            ("css_url", "real.png", Some("real.png"), None, None),
            ("css_url", "set.png", Some("set.png"), None, None),
        ]
    );
}

#[test]
fn html_src_refs_only_scan_tag_attributes() {
    let refs = scan(
        r#"
        src = "plain.png"
        <script>const src = "script.png";</script>
        <source srcset="one.png 1x, two.png 2x">
        <img src="real.png">
        "#,
    );

    assert_eq!(
        ref_summaries(&refs),
        vec![("html_src", "real.png", Some("real.png"), None, None)]
    );
}

#[test]
fn html_src_refs_ignore_tag_like_text_inside_raw_text_elements() {
    let refs = scan(
        r#"
        <script>const html = "<img src='script-ghost.png'>";</script>
        <style>.ghost::after { content: "<img src='style-ghost.png'>"; background: url(style-bg.png); }</style>
        <img src="real.png">
        "#,
    );

    assert_eq!(
        ref_summaries(&refs),
        vec![
            ("html_src", "real.png", Some("real.png"), None, None),
            ("css_url", "style-bg.png", Some("style-bg.png"), None, None),
        ]
    );
}

#[test]
fn html_attrs_ignore_fake_refs_inside_quoted_attribute_values() {
    let refs = scan(
        r#"
        <img alt="src=ghost.png" src="real.png">
        <object title="data=ghost.svg" data="real.svg"></object>
        "#,
    );

    assert_eq!(
        ref_summaries(&refs),
        vec![
            ("html_src", "real.png", Some("real.png"), None, None),
            ("html_object_data", "real.svg", Some("real.svg"), None, None),
        ]
    );
}

#[test]
fn css_url_refs_ignore_block_comments() {
    let refs = scan(
        r#"
        .commented { background: /* url(comment.png) */ none; }
        /* .also-commented { background: url(comment-two.png); } */
        .real { background: url(real.png); }
        "#,
    );

    assert_eq!(
        ref_summaries(&refs),
        vec![("css_url", "real.png", Some("real.png"), None, None)]
    );
}

#[test]
fn css_url_refs_ignore_script_text_and_css_strings() {
    let refs = scan(
        r#"
        <script>const css = "url(script-fake.png)";</script>
        <style>.x::before { content: "url(style-string-fake.png)"; background: url(style-real.png); }</style>
        .field::after { content: 'url(field-string-fake.png)'; background: url(field-real.png); }
        "#,
    );

    assert_eq!(
        ref_summaries(&refs),
        vec![
            (
                "css_url",
                "style-real.png",
                Some("style-real.png"),
                None,
                None,
            ),
            (
                "css_url",
                "field-real.png",
                Some("field-real.png"),
                None,
                None,
            ),
        ]
    );
}

#[test]
fn invalid_percent_escapes_are_unsafe() {
    let refs = scan(
        r#"
        <img src="bad%.png">
        <img src="bad%2.png">
        <style>.x { background: url(bad%ZZ.png); }</style>
        "#,
    );

    assert_eq!(
        ref_summaries(&refs),
        vec![
            (
                "html_src",
                "bad%.png",
                None,
                None,
                Some("invalid-percent-encoding"),
            ),
            (
                "html_src",
                "bad%2.png",
                None,
                None,
                Some("invalid-percent-encoding"),
            ),
            (
                "css_url",
                "bad%ZZ.png",
                None,
                None,
                Some("invalid-percent-encoding"),
            ),
        ]
    );
}

#[test]
fn decoded_empty_dot_and_dot_dot_paths_are_unsafe() {
    let refs = scan(
        r##"
        <img src="?cache=1">
        <img src="#fragment">
        <img src=".">
        <img src="%2E">
        <style>.x { background: url(..); background-image: url(%2E%2E); }</style>
        "##,
    );

    assert_eq!(
        ref_summaries(&refs),
        vec![
            (
                "html_src",
                "?cache=1",
                None,
                None,
                Some("decoded-empty-path"),
            ),
            (
                "html_src",
                "#fragment",
                None,
                None,
                Some("decoded-empty-path"),
            ),
            ("html_src", ".", None, None, Some("decoded-dot-path")),
            ("html_src", "%2E", None, None, Some("decoded-dot-path")),
            ("css_url", "..", None, None, Some("decoded-dot-path")),
            ("css_url", "%2E%2E", None, None, Some("decoded-dot-path")),
        ]
    );
}

fn scan(input: &str) -> Vec<authoring_core::MediaReferenceCandidate> {
    extract_media_reference_candidates("note", "note-1", "field", "Front", input)
}

fn ref_summaries(refs: &[authoring_core::MediaReferenceCandidate]) -> Vec<ReferenceSummary<'_>> {
    refs.iter()
        .map(|item| {
            (
                item.ref_kind.as_str(),
                item.raw_ref.as_str(),
                item.normalized_local_ref.as_deref(),
                item.skip_reason.as_deref(),
                item.unsafe_reason.as_deref(),
            )
        })
        .collect()
}
