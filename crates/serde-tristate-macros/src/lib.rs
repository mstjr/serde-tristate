use proc_macro::{Delimiter, Group, Punct, Spacing, TokenStream, TokenTree};

const TRISTATE_SERDE_ATTR: &str =
    r#"#[serde(default, skip_serializing_if = "Tristate::is_undefined")]"#;

/// Attribute macro that auto-injects `#[serde(default, skip_serializing_if = "Tristate::is_undefined")]`
/// onto every `Tristate<T>` field. Works on structs and enums with named variant fields.
///
/// ```ignore
/// #[serde_tristate]
/// #[derive(Serialize, Deserialize)]
/// struct Dto {
///     name: Tristate<String>,
///     age: Tristate<u32>,
/// }
/// ```
#[proc_macro_attribute]
pub fn serde_tristate(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut tts: Vec<TokenTree> = input.into_iter().collect();

    if let Some(idx) = tts
        .iter()
        .position(|tt| matches!(tt, TokenTree::Group(g) if g.delimiter() == Delimiter::Brace))
    {
        let g = match &tts[idx] {
            TokenTree::Group(g) => g.clone(),
            _ => unreachable!(),
        };
        let processed = process_body(g.stream());
        let mut new_g = Group::new(Delimiter::Brace, processed);
        new_g.set_span(g.span());
        tts[idx] = TokenTree::Group(new_g);
    }

    tts.into_iter().collect()
}

/// Process a struct body (fields) or enum body (variants).
fn process_body(stream: TokenStream) -> TokenStream {
    let (items, trailing_comma) = split_on_commas(stream);
    let mut out = TokenStream::new();

    for (i, item) in items.iter().enumerate() {
        process_item(item, &mut out);
        if i + 1 < items.len() || trailing_comma {
            out.extend(std::iter::once(TokenTree::Punct(Punct::new(
                ',',
                Spacing::Alone,
            ))));
        }
    }

    out
}

/// Process one item (struct field or enum variant).
/// Enum variants with named fields contain a `{...}` subgroup — recurse into it.
fn process_item(item: &[TokenTree], out: &mut TokenStream) {
    if item.is_empty() {
        return;
    }

    if let Some(brace_pos) = item
        .iter()
        .position(|tt| matches!(tt, TokenTree::Group(g) if g.delimiter() == Delimiter::Brace))
    {
        out.extend(item[..brace_pos].iter().cloned());
        let g = match &item[brace_pos] {
            TokenTree::Group(g) => g.clone(),
            _ => unreachable!(),
        };
        let processed = process_body(g.stream());
        let mut new_g = Group::new(Delimiter::Brace, processed);
        new_g.set_span(g.span());
        out.extend(std::iter::once(TokenTree::Group(new_g)));
        out.extend(item[brace_pos + 1..].iter().cloned());
    } else {
        if is_tristate_field(item) {
            let attr: TokenStream = TRISTATE_SERDE_ATTR.parse().unwrap();
            out.extend(attr);
        }
        out.extend(item.iter().cloned());
    }
}

/// Split on top-level commas, tracking `<>` depth so commas inside generics are skipped.
/// Returns the items and whether the stream had a trailing comma.
fn split_on_commas(stream: TokenStream) -> (Vec<Vec<TokenTree>>, bool) {
    let mut items: Vec<Vec<TokenTree>> = vec![Vec::new()];
    let mut angle_depth: i32 = 0;
    let mut trailing_comma = false;

    for tt in stream {
        match &tt {
            TokenTree::Punct(p) if p.as_char() == '<' => {
                angle_depth += 1;
                items.last_mut().unwrap().push(tt);
                trailing_comma = false;
            }
            TokenTree::Punct(p) if p.as_char() == '>' => {
                angle_depth = (angle_depth - 1).max(0);
                items.last_mut().unwrap().push(tt);
                trailing_comma = false;
            }
            TokenTree::Punct(p) if p.as_char() == ',' && angle_depth == 0 => {
                trailing_comma = true;
                items.push(Vec::new());
            }
            _ => {
                items.last_mut().unwrap().push(tt);
                trailing_comma = false;
            }
        }
    }

    if items.last().map(|v| v.is_empty()).unwrap_or(false) {
        items.pop();
    }

    (items, trailing_comma)
}

/// Detect if a field's type is `Tristate<...>`.
/// Looks for `Tristate` followed by `<` after the first single `:` (not `::`) in the token list.
fn is_tristate_field(tokens: &[TokenTree]) -> bool {
    let mut after_colon = false;
    let mut i = 0;

    while i < tokens.len() {
        if !after_colon {
            if let TokenTree::Punct(p) = &tokens[i] {
                if p.as_char() == ':' {
                    let next_is_colon = matches!(
                        tokens.get(i + 1),
                        Some(TokenTree::Punct(p2)) if p2.as_char() == ':'
                    );
                    if next_is_colon {
                        i += 2; // skip `::`
                        continue;
                    }
                    after_colon = true;
                }
            }
        } else if let TokenTree::Ident(id) = &tokens[i] {
            if id.to_string() == "Tristate" {
                let next_is_lt = matches!(
                    tokens.get(i + 1),
                    Some(TokenTree::Punct(p)) if p.as_char() == '<'
                );
                if next_is_lt {
                    return true;
                }
            }
        }
        i += 1;
    }

    false
}
