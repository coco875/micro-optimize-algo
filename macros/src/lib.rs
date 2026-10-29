use proc_macro::TokenStream;
use quote::quote;

// Shared with build-time discovery.
mod metadata;

fn annotated(attr: TokenStream, item: TokenStream, kind: &str) -> TokenStream {
    let result = (|| {
        metadata::parse(attr.into(), kind)?;
        let parsed: syn::Item = syn::parse(item.clone())?;
        let valid = match kind {
            "algorithm" => matches!(parsed, syn::Item::Trait(_)),
            _ => matches!(parsed, syn::Item::Struct(_)),
        };
        if !valid {
            return Err(syn::Error::new_spanned(
                parsed,
                "annotation is on the wrong item kind",
            ));
        }
        if let syn::Item::Struct(ref structure) = parsed {
            let parameters = &structure.generics.params;
            let valid_generics = if kind == "scenario" {
                parameters.len() == 1
                    && matches!(parameters.first(), Some(syn::GenericParam::Type(_)))
            } else {
                parameters.is_empty()
            };
            if !valid_generics {
                return Err(syn::Error::new_spanned(
                    &structure.generics,
                    if kind == "scenario" {
                        "a scenario requires exactly one implementation type parameter"
                    } else {
                        "a variant marker must not be generic"
                    },
                ));
            }
        }
        Ok(item)
    })();
    result.unwrap_or_else(|e: syn::Error| e.into_compile_error().into())
}

#[proc_macro_attribute]
pub fn algorithm(attr: TokenStream, item: TokenStream) -> TokenStream {
    annotated(attr, item, "algorithm")
}

#[proc_macro_attribute]
pub fn scenario(attr: TokenStream, item: TokenStream) -> TokenStream {
    annotated(attr, item, "scenario")
}

#[proc_macro_attribute]
pub fn variant(attr: TokenStream, item: TokenStream) -> TokenStream {
    annotated(attr, item, "variant")
}

fn modules(input: TokenStream, prefix: &str) -> TokenStream {
    let family = syn::parse_macro_input!(input as syn::LitStr);
    let file = format!("/{prefix}_{}.rs", family.value().replace('/', "_"));
    quote!(include!(concat!(env!("OUT_DIR"), #file));).into()
}

#[proc_macro]
pub fn algorithm_modules(input: TokenStream) -> TokenStream {
    modules(input, "scenarios")
}

#[proc_macro]
pub fn implementation_modules(input: TokenStream) -> TokenStream {
    modules(input, "implementations")
}
