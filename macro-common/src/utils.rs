use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::meta::ParseNestedMeta;
use syn::parse::Result;
use syn::LitStr;

pub struct ConfigAttributes {
    pub path: LitStr,
    pub post_load: bool,
    pub post_load_message: Option<LitStr>,
}

impl ConfigAttributes {
    pub fn new() -> Self {
        Self {
            path: LitStr::new("", Span::call_site()),
            post_load: false,
            post_load_message: None,
        }
    }
    pub fn post_load_method_code_gen(&self) -> TokenStream {
        if let Some(ref message) = self.post_load_message {
            quote! {
                fn post_load(&self) {
                    tracing::info!(#message);
                }
            }
        } else {
            quote! {}
        }
    }
    pub fn parse(&mut self, meta: ParseNestedMeta) -> Result<()> {
        if meta.path.is_ident("path") {
            self.path = meta.value()?.parse()?;
            if self.path.value().is_empty() {
                return Err(meta.error("Please specify path to configuration"));
            }
            Ok(())
        } else if meta.path.is_ident("msg") {
            self.post_load_message = meta.value()?.parse()?;
            Ok(())
        } else if meta.path.is_ident("post_load") {
            self.post_load = true;
            Ok(())
        } else {
            Err(meta.error("unsupported property"))
        }
    }
}

pub fn generate_code_for_config_file<T: ToTokens, K: ToTokens>(
    post_load: bool,
    item: &T,
    name: &Ident,
    post_load_method: &K,
    path: &LitStr,
) -> TokenStream {
    if post_load {
        quote! {
            #item
            impl l2_core::config::traits::ConfigFileLoader for #name {
                const DATA_FILE: &'static str = #path;
            }
        }
    } else {
        quote! {
            #item
            impl l2_core::config::traits::Loadable for #name {
                #post_load_method
            }
            impl l2_core::config::traits::ConfigFileLoader for #name {
                const DATA_FILE: &'static str = #path;
            }
        }
    }
}
