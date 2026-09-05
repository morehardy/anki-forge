use super::{load_registry, run_registry_gates};
use crate::manifest::{load_manifest, resolve_asset_path};
use anyhow::{ensure, Context, Result};
use proc_macro2::{TokenStream, TokenTree};
use std::{collections::BTreeMap, fs, path::Path};
use syn::visit::{self, Visit};

/// Check built-in diagnostic literals in both production Rust source trees.
///
/// Scan all code-shaped literals, including constants, helper arguments and
/// macros, rather than maintaining a list of diagnostic constructor names.
/// Documentation/comments and explicitly test-only functions/modules are not
/// production code. All other feature/platform branches are scanned.
/// Built-in codes must be whole literals; adapters may forward existing codes.
pub fn run_source_registry_gates(manifest_path: &Path, source_root: &Path) -> Result<()> {
    run_registry_gates(manifest_path)?;
    let manifest = load_manifest(manifest_path)?;
    let registry = load_registry(resolve_asset_path(&manifest, "error_registry")?)?;
    let statuses: BTreeMap<_, _> = registry
        .codes
        .iter()
        .map(|code| (code.id.as_str(), code.status.as_str()))
        .collect();
    let mut problems = Vec::new();
    for relative in ["anki_forge/src", "contract_tools/src"] {
        let root = source_root.join(relative);
        ensure!(
            root.is_dir(),
            "production source directory is missing: {}",
            root.display()
        );
        check_directory(&root, source_root, &statuses, &mut problems)?;
    }
    problems.sort();
    problems.dedup();
    ensure!(
        problems.is_empty(),
        "production diagnostic registry coverage failed:\n{}",
        problems.join("\n")
    );
    Ok(())
}

fn check_directory(
    path: &Path,
    root: &Path,
    statuses: &BTreeMap<&str, &str>,
    problems: &mut Vec<String>,
) -> Result<()> {
    for entry in
        fs::read_dir(path).with_context(|| format!("read source directory {}", path.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let kind = entry.file_type()?;
        ensure!(
            !kind.is_symlink(),
            "source scan does not follow symlinks: {}",
            path.display()
        );
        if kind.is_dir() {
            check_directory(&path, root, statuses, problems)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let raw = fs::read_to_string(&path)?;
            let syntax = syn::parse_file(&raw)
                .with_context(|| format!("parse Rust source {}", path.display()))?;
            if test_only(&syntax.attrs) {
                continue;
            }
            let mut literals = CodeLiterals::default();
            literals.visit_file(&syntax);
            for (code, line) in literals.codes {
                let reason = match statuses.get(code.as_str()) {
                    None => "not registered",
                    Some(&"removed") => "registered as removed",
                    _ => continue,
                };
                problems.push(format!(
                    "{}:{line}: {code} is {reason}",
                    path.strip_prefix(root)?.display()
                ));
            }
        }
    }
    Ok(())
}

#[derive(Default)]
struct CodeLiterals {
    codes: Vec<(String, usize)>,
}

impl CodeLiterals {
    fn tokens(&mut self, tokens: TokenStream) {
        for token in tokens {
            match token {
                TokenTree::Group(group) => self.tokens(group.stream()),
                TokenTree::Literal(literal) => {
                    if let Ok(string) = syn::parse_str::<syn::LitStr>(&literal.to_string()) {
                        self.record(&string.value(), literal.span().start().line);
                    }
                }
                _ => {}
            }
        }
    }

    fn record(&mut self, value: &str, line: usize) {
        let legacy = value
            .strip_prefix("AF")
            .is_some_and(|suffix| !suffix.is_empty() && suffix.bytes().all(|b| b.is_ascii_digit()));
        let dotted = value.contains('.')
            && value.split('.').all(|part| {
                part.as_bytes().first().is_some_and(u8::is_ascii_uppercase)
                    && part
                        .bytes()
                        .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
            });
        if legacy || dotted {
            self.codes.push((value.to_owned(), line));
        }
    }
}

impl<'ast> Visit<'ast> for CodeLiterals {
    fn visit_attribute(&mut self, _: &'ast syn::Attribute) {}

    fn visit_lit_str(&mut self, literal: &'ast syn::LitStr) {
        self.record(&literal.value(), literal.span().start().line);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        self.tokens(node.tokens.clone());
    }

    fn visit_item(&mut self, node: &'ast syn::Item) {
        let attrs = match node {
            syn::Item::Const(v) => &v.attrs,
            syn::Item::Enum(v) => &v.attrs,
            syn::Item::ExternCrate(v) => &v.attrs,
            syn::Item::Fn(v) => &v.attrs,
            syn::Item::ForeignMod(v) => &v.attrs,
            syn::Item::Impl(v) => &v.attrs,
            syn::Item::Macro(v) => &v.attrs,
            syn::Item::Mod(v) => &v.attrs,
            syn::Item::Static(v) => &v.attrs,
            syn::Item::Struct(v) => &v.attrs,
            syn::Item::Trait(v) => &v.attrs,
            syn::Item::TraitAlias(v) => &v.attrs,
            syn::Item::Type(v) => &v.attrs,
            syn::Item::Union(v) => &v.attrs,
            syn::Item::Use(v) => &v.attrs,
            _ => return visit::visit_item(self, node),
        };
        if !test_only(attrs) {
            visit::visit_item(self, node);
        }
    }

    fn visit_impl_item(&mut self, node: &'ast syn::ImplItem) {
        let attrs = match node {
            syn::ImplItem::Const(v) => &v.attrs,
            syn::ImplItem::Fn(v) => &v.attrs,
            syn::ImplItem::Type(v) => &v.attrs,
            syn::ImplItem::Macro(v) => &v.attrs,
            _ => return visit::visit_impl_item(self, node),
        };
        if !test_only(attrs) {
            visit::visit_impl_item(self, node);
        }
    }

    fn visit_trait_item(&mut self, node: &'ast syn::TraitItem) {
        let attrs = match node {
            syn::TraitItem::Const(v) => &v.attrs,
            syn::TraitItem::Fn(v) => &v.attrs,
            syn::TraitItem::Type(v) => &v.attrs,
            syn::TraitItem::Macro(v) => &v.attrs,
            _ => return visit::visit_trait_item(self, node),
        };
        if !test_only(attrs) {
            visit::visit_trait_item(self, node);
        }
    }
}

fn test_only(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("test")
            || (attr.path().is_ident("cfg")
                && attr
                    .parse_args::<syn::Meta>()
                    .is_ok_and(|meta| cfg_without_test(&meta) == Some(false)))
    })
}

// Unknown features/platforms may be enabled in production; only discard a
// branch when it is impossible with cfg(test) = false.
fn cfg_without_test(meta: &syn::Meta) -> Option<bool> {
    match meta {
        syn::Meta::Path(path) if path.is_ident("test") => Some(false),
        syn::Meta::List(list) => {
            let args = list
                .parse_args_with(
                    syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
                )
                .ok()?;
            let values: Vec<_> = args.iter().map(cfg_without_test).collect();
            if list.path.is_ident("all") {
                if values.contains(&Some(false)) {
                    Some(false)
                } else if values.iter().all(|v| *v == Some(true)) {
                    Some(true)
                } else {
                    None
                }
            } else if list.path.is_ident("any") {
                if values.contains(&Some(true)) {
                    Some(true)
                } else if values.iter().all(|v| *v == Some(false)) {
                    Some(false)
                } else {
                    None
                }
            } else if list.path.is_ident("not") && values.len() == 1 {
                values[0].map(|v| !v)
            } else {
                None
            }
        }
        _ => None,
    }
}
