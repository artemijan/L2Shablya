extern crate proc_macro;
mod utils;

use crate::utils::ConfigAttributes;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Field, FieldMutability, Fields, Item, ItemStruct, Type, Visibility};
use syn::token::Colon;

#[allow(clippy::missing_panics_doc)]
#[proc_macro_attribute]
pub fn config_file(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attribute argument (e.g., a `&'static str`)
    let mut attrs = ConfigAttributes::new();
    let parser = syn::meta::parser(|meta| attrs.parse(&meta));
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
    let parser = syn::meta::parser(|meta| attrs.parse(&meta));
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



/// # Panics
/// - `PacketEnum` can only be derived for enums
/// - Each variant must be a tuple struct with a single field
#[proc_macro_derive(PacketEnum)]
pub fn derive_packet_repository(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;

    let Data::Enum(data_enum) = &input.data else {
        return syn::Error::new_spanned(
            enum_name,
            "PacketRepository can only be derived for enums",
        )
        .to_compile_error()
        .into();
    };

    // Build match arms and collect field types for the trait bounds
    let mut match_arms = Vec::new();
    let mut field_types = Vec::new();

    for variant in &data_enum.variants {
        let variant_name = &variant.ident;
        let field_ty = match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                &fields.unnamed.first().unwrap().ty
            }
            _ => {
                return syn::Error::new_spanned(
                    &variant.fields,
                    "Each variant must be a tuple struct with a single field",
                )
                .to_compile_error()
                .into();
            }
        };

        match_arms.push(quote! {
            #enum_name::#variant_name(msg) => actor
                .tell(msg)
                .try_send()
                .map_err(|e| ::anyhow::anyhow!("{:?}", e)),
        });

        field_types.push(field_ty);
    }

    let expanded = quote! {
        #[automatically_derived]
        impl #enum_name {
            pub async fn accept<T>(self, actor: ::kameo::prelude::ActorRef<T>) -> ::anyhow::Result<()>
            where
                T: ::kameo::prelude::Actor
                    #( + ::kameo::prelude::Message<#field_types, Reply = ::anyhow::Result<()>> )*
            {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
#[proc_macro_derive(SendablePacket)]
pub fn derive_packet(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    let (_, generics) = (input.vis, input.generics);

    let struct_name_str = struct_name.to_string();

    let expanded = quote! {
        impl #generics l2_core::shared_packets::common::SendablePacket for #struct_name #generics {
            fn get_buffer(self) -> l2_core::shared_packets::write::SendablePacketBuffer {
                self.buffer
            }
            fn name(&self) -> &'static str {
                #struct_name_str
            }
        }
    };

    TokenStream::from(expanded)
}