use quote::quote;

pub fn derive_props(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = syn::parse_macro_input!(input as syn::DeriveInput);
  let ident = input.ident;

  let result = quote! {
    impl ::ruxy::Props for #ident {}
    
    // TODO: It would be nice if we could provide these implementations for all props,
    //       not just the macro-derived ones. For now it's better than nothing, though.
    //       Rust's Orphan Rules won't let us do a generic impl.
    impl<E> ::std::convert::From<#ident> for ::std::result::Result<#ident, E> {
      fn from(props: #ident) -> Self {
        ::std::result::Result::Ok(props)
      }
    }
    
    impl<E> ::std::convert::From<#ident> for ::std::result::Result<::ruxy::LoaderOutput<#ident>, E> {
      fn from(props: #ident) -> Self {
        ::std::result::Result::Ok(props.into())
      }
    }
  };
  
  result.into()
}
