use std::{env, fs, path::PathBuf};

use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Group};
use quote::quote;
use syn::{LitStr, parse_macro_input};

#[proc_macro]
pub fn run(input: TokenStream) -> TokenStream {
    let file_path_lit = parse_macro_input!(input as LitStr);
    let file_name = file_path_lit.value();
    let mut path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    path.push(file_name);
    let path_str = path.to_str().unwrap();
    let content =
        fs::read_to_string(&path).unwrap_or_else(|_| panic!("Unable to read file at {:?}", path));

    struct Builder<I: Iterator<Item = u8>>(I);
    pub trait BuilderExt: Iterator<Item = u8> + Sized {
        fn build(self) -> Builder<Self> {
            Builder(self)
        }
    }
    impl<I: Iterator<Item = u8>> BuilderExt for I {}
    impl<I: Iterator<Item = u8>> Iterator for Builder<I> {
        type Item = proc_macro2::TokenStream;
        fn next(&mut self) -> Option<Self::Item> {
            match self.0.next()? {
                b'>' => Some(quote! {ptr = ptr.wrapping_add(1) & (mem_size - 1);}),
                b'<' => Some(quote! {ptr = ptr.wrapping_sub(1) & (mem_size - 1);}),
                b'+' => Some(quote! {*mem.get_unchecked_mut(ptr) += ::std::num::Wrapping(1);}),
                b'-' => Some(quote! {*mem.get_unchecked_mut(ptr) -= ::std::num::Wrapping(1);}),
                b'.' => Some(quote! {let _ = handle.write_all(&[mem.get_unchecked(ptr).0]);}),
                b',' => Some(quote! {let _ = stdin.read_exact(&mut input);
                *mem.get_unchecked_mut(ptr) = ::std::num::Wrapping(input[0]);}),
                b'[' => {
                    let tokens = Group::new(Delimiter::Brace, self.by_ref().collect());
                    Some(quote! {while mem.get_unchecked(ptr).0 != 0 #tokens})
                }
                b']' => None,
                _ => self.next(),
            }
        }
    }

    let bf = content
        .as_bytes()
        .iter()
        .copied()
        .build()
        .collect::<Vec<_>>();
    let expanded = quote! {
        {
            use ::std::io::Write as _;
            const _: &[u8] = include_bytes!(#path_str);
            let mem_size = 1<<15;
            let mut ptr: usize = 0;
            let stdout = ::std::io::stdout();
            let mut handle = stdout.lock();
            let mut stdin = ::std::io::stdin();
            let mut input = [0u8];
            let mut mem = vec![::std::num::Wrapping(0); mem_size];
            unsafe {
                #(#bf)*
            }
        }
    };
    TokenStream::from(expanded)
}
