use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Path};
use packet::Packet;

#[proc_macro_derive(Packet, attributes(packet_type))]
pub fn proc_packet_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let type_name = &input.ident;

    let packet_type = input.attrs.iter()
        .find_map(|attr|
            attr.path()
                .is_ident("packet_type")
                .then(|| attr.parse_args::<Path>().ok())
        )
        .flatten()
        .expect("Missing 'packet_type' attribute");

    let ty = packet_type.segments.first().unwrap().ident.clone();

    let expanded = quote! {
        impl Packet<#ty> for #type_name {
            const TYPE: #ty = #packet_type;
        }
    };

    expanded.into()
}