use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn when(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn irql_requires(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn irql_requires_max(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
