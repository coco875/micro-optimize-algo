use std::collections::BTreeMap;

pub fn parse(
    tokens: proc_macro2::TokenStream,
    kind: &str,
) -> syn::Result<BTreeMap<String, String>> {
    use syn::parse::Parser;
    let values =
        syn::punctuated::Punctuated::<syn::MetaNameValue, syn::Token![,]>::parse_terminated
            .parse2(tokens)?;
    let allowed: &[&str] = match kind {
        "algorithm" => &["group", "reference"],
        "variant" => &["group", "id"],
        "scenario" => &["group", "id", "name", "tags"],
        _ => unreachable!(),
    };
    let mut result = BTreeMap::new();
    for value in values {
        let key = value
            .path
            .get_ident()
            .map(ToString::to_string)
            .unwrap_or_default();
        if !allowed.contains(&key.as_str()) {
            return Err(syn::Error::new_spanned(value, "unknown annotation key"));
        }
        let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(ref text),
            ..
        }) = value.value
        else {
            return Err(syn::Error::new_spanned(
                value.value,
                "expected a string literal",
            ));
        };
        if text.value().trim().is_empty() || result.insert(key, text.value()).is_some() {
            return Err(syn::Error::new_spanned(
                value,
                "empty value or duplicate key",
            ));
        }
    }
    let required: &[&str] = if kind == "algorithm" {
        &["group", "reference"]
    } else {
        &["group"]
    };
    for required in required {
        if !result.contains_key(*required) {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("missing {required}"),
            ));
        }
    }
    Ok(result)
}
