extern crate proc_macro;

extern crate syn;

#[macro_use]
extern crate quote;

use self::proc_macro::TokenStream;

use self::syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Result, Token, Type,
};

#[derive(Debug)]
struct Pow10 {
    integer_type: syn::Type,
    name: syn::Ident,
}

impl Parse for Pow10 {
    fn parse(input: ParseStream) -> Result<Self> {
        let integer_type = input.parse()?;

        let _: Token![,] = input.parse()?;

        let name: syn::Ident = input.parse()?;

        Ok(Self { integer_type, name })
    }
}

impl Pow10 {
    fn yank_int_type(&self) -> Option<String> {
        match (*self).integer_type {
            Type::Path(ref inner) => {
                let x = inner.path.segments.first()?;
                Some(quote!(#x).to_string())
            }
            _ => None,
        }
    }
}

// @TODO: Replace this with pow10 macro.
trait Integer {
    const LEN: usize;
}

impl Integer for usize {
    const LEN: usize = 20;
}

impl Integer for u64 {
    const LEN: usize = 20;
}

impl Integer for u32 {
    const LEN: usize = 10;
}

impl Integer for u16 {
    const LEN: usize = 5;
}

impl Integer for u8 {
    const LEN: usize = 3;
}

macro_rules! build_array {
    ($int_type:ty, $name:expr) => {{

        let len = <$int_type>::LEN;

        let v = Powers::new().take(len).map(|n| n as $int_type).collect::<Vec<_>>();

        let name = $name;

        quote!(const #name: [$int_type; #len] = [#(#v),*];).into()
    }}
}

#[proc_macro]
pub fn table_of(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as Pow10);

    let name = input.name.clone();

    match &*input.yank_int_type().unwrap() {
        "usize" => build_array!(usize, name),
        "u64" => build_array!(u64, name),
        "u32" => build_array!(u32, name),
        "u16" => build_array!(u16, name),
        "u8" => build_array!(u8, name),

        ty => {
            let s = format!("table_of not implemented for type `{}`", ty);
            syn::Error::new(input.name.span(), s)
                .to_compile_error()
                .into()
        }
    }
}

struct Powers {
    current: usize,
    done: bool,
}

impl Powers {
    fn new() -> Self {
        Self {
            current: 1,
            done: false,
        }
    }
}

impl Iterator for Powers {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let ret = self.current;

        let (new, overflowed) = self.current.overflowing_mul(10);

        if overflowed {
            self.done = true;
        } else {
            self.current = new;
        }
        Some(ret)
    }
}
