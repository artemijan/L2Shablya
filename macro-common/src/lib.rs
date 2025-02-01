extern crate proc_macro;
mod utils;

use crate::utils::ConfigAttributes;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Item, ItemStruct};

#[allow(clippy::missing_panics_doc)]
#[proc_macro_attribute]
pub fn config_file(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attribute argument (e.g., a `&'static str`)
    let mut attrs = ConfigAttributes::new();
    let parser = syn::meta::parser(|meta| attrs.parse(meta));
    // Parse the item (e.g., a struct)
    let item = parse_macro_input!(item as Item);
    parse_macro_input!(attr with parser);
    // Generate the `post_load` method if `post_load_message` is provided
    let post_load_method = attrs.post_load_method_code_gen();
    // Generate the implementation of `Loadable`
    let expanded = match item {
        Item::Struct(item_struct) => {
            let name = &item_struct.ident;
            utils::generate_code_for_config_file(
                attrs.post_load,
                &item_struct,
                name,
                &post_load_method,
                &attrs.path,
            )
        }
        Item::Enum(item_enum) => {
            let name = &item_enum.ident;
            utils::generate_code_for_config_file(
                attrs.post_load,
                &item_enum,
                name,
                &post_load_method,
                &attrs.path,
            )
        }
        _ => panic!("`#[loadable]` can only be applied to structs or enums"),
    };

    // Return the generated code as a token stream
    TokenStream::from(expanded)
}
#[proc_macro_attribute]
pub fn config_dir(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut attrs = ConfigAttributes::new();
    let parser = syn::meta::parser(|meta| attrs.parse(meta));
    // Parse the item (e.g., a struct)
    parse_macro_input!(attr with parser);
    // Generate the `post_load` method if `post_load_message` is provided
    let post_load_method = attrs.post_load_method_code_gen();
    let path = attrs.path.clone();
    // Parse the item (e.g., a struct)
    let item = parse_macro_input!(item as ItemStruct);

    // Get the name of the struct
    let name = &item.ident;

    // Generate the implementation of `Loadable`
    let expanded = if attrs.post_load {
        quote! {
            #item
            impl l2_core::config::traits::ConfigDirLoader for #name {
                const DATA_DIR: &'static str = #path;
            }
        }
    } else {
        quote! {
            #item

            impl l2_core::config::traits::Loadable for #name {
                #post_load_method
            }
            impl l2_core::config::traits::ConfigDirLoader for #name {
                const DATA_DIR: &'static str = #path;
            }
        }
    };

    // Return the generated code as a token stream
    TokenStream::from(expanded)
}

#[proc_macro_derive(SendablePacketImpl)]
pub fn derive_sendable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    // Generate the implementation
    let expanded = quote! {
        impl l2_core::shared_packets::common::SendablePacket for #name {
            fn get_bytes(&mut self, with_padding:bool) -> &mut [u8] {
                self.buffer.get_data_mut(with_padding)
            }
        }
    };

    TokenStream::from(expanded)
}
