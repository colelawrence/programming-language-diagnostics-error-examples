#![crate_type = "proc-macro"]

extern crate proc_macro2;

use proc_macro2::{TokenStream, TokenTree};

/// Convenience macro for generating codegen attributes
///
/// Example:
/// ```rs
/// #[protocol("wasm")]
/// struct Point(f64, f64);
/// // Expands to:
/// #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
/// #[codegen(tags = "protocol-wasm")]
/// struct Point(f64, f64);
/// ```
#[proc_macro_attribute]
pub fn protocol(
    input: proc_macro::TokenStream,
    following: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let following = proc_macro2::TokenStream::from(following);
    let mut input_it = input.into_iter();
    let protocol_name = input_it
        .next()
        .expect("protocol attribute must have a value");
    let name = match protocol_name {
        TokenTree::Literal(lit) => {
            let value = lit.to_string();
            value[1..value.len() - 1].to_string()
        }
        other => {
            panic!("Unexpected token in protocol attribute: {:?}", other);
        }
    };

    let tag = format!("protocol-{name}");
    let attrs = input_it.collect::<TokenStream>();
    let codegen_attr = quote::quote! {#[codegen(tags = #tag #attrs)]};
    let mut output = quote::quote! {
        #[allow(non_snake_case)]
        #[derive(derive_codegen::Codegen, serde::Serialize, serde::Deserialize, Debug, Clone)]
        #codegen_attr
    };

    // If the item is an enum && not externally tagged, add #[serde(tag = "type")]
    if following.to_string().contains("enum ") && !attrs.to_string().contains("externally_tagged") {
        output.extend(quote::quote! {
            #[serde(tag = "type")]
        });
    }

    output.extend(following);
    proc_macro::TokenStream::from(output)
}
