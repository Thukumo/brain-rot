use proc_macro::TokenStream;

#[proc_macro]
pub fn hello(input: TokenStream) -> TokenStream {
    input
}
