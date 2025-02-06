use proc_macro::TokenStream;
use proc_macro::Span;
use std::fs;
use std::path::PathBuf;
use quote::quote;
use serde::Deserialize;
use syn::{parse_macro_input, LitStr};

#[derive(Deserialize)]
struct BlockJson {
    texture: String
}

#[proc_macro]
pub fn block(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let file = input.value();

    let caller_path = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is not set");
    
    let abs_path = PathBuf::from(caller_path).join(file);
    let json_str = fs::read_to_string(abs_path).unwrap();

    let block: BlockJson = serde_json::from_str(&json_str).expect("Failed to parse JSON");
    let texture = block.texture;

    let expanded = quote! {
        $crate::block::Block {
            texture: resources::ResourceKey::<$crate::texture::Texture>::try_from(#texture).expect("Failed to parse the block"),
        }
    };

    expanded.into()
}