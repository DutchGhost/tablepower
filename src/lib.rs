extern crate proc_macro;

extern crate syn;

#[macro_use]
extern crate quote;

use self::proc_macro::TokenStream;

use self::syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Result, Token, Type,
};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
enum Order {
    Ascending,
    Descending,
}

#[derive(Debug)]
struct Pow10 {
    integer_type: syn::Type,
    name: syn::Ident,
    order: Order,
    exponent: usize,
}

impl Parse for Pow10 {
    fn parse(input: ParseStream) -> Result<Self> {
        let integer_type = input.parse()?;

        let _: Token![,] = input.parse()?;

        let name: syn::Ident = input.parse()?;

        let _: Option<Token![,]> = input.parse()?;

        let order: Option<syn::Ident> = input.parse()?;

        let order_of_table = order.map(|order| {
            if order != "order" {
                return Err(syn::Error::new(order.span(), "Expected `order`"));
            }

            let _: Token![=] = input.parse()?;

            let order_argument: syn::Ident = input.parse()?;

            if order_argument == "ascending" {
                Ok(Order::Ascending)
            } else if order_argument == "descending" {
                Ok(Order::Descending)
            } else {
                Err(syn::Error::new(order.span(), "Invalid order specifier specified!. Did you mean `ascending` or `descending`?"))
            }
 
        }).unwrap_or(Ok(Order::Ascending))?;

        let _: Option<Token![,]> = input.parse()?;

        let exponent: Option<syn::Ident> = input.parse()?;

        let exponent_value = exponent
            .map(|exponent| {
                if exponent != "exponent" {
                    return Err(syn::Error::new(exponent.span(), "Expected `exponent`"));
                }

                let _: Token![=] = input.parse()?;

                let value: syn::LitInt = input.parse()?;

                Ok(value.value())
            })
            .unwrap_or(Ok(10))?;

        Ok(Self {
            integer_type,
            name,
            order: order_of_table,
            exponent: exponent_value as usize,
        })
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

macro_rules! powers {
    ($int:ty, exponent = $exponent:expr) => {{
        struct Powers {
            current: $int,
            exponent: $int,
            done: bool,
        }

        impl Powers {
            fn new(exponent: usize) -> Self {
                Self {
                    current: 1,
                    exponent: exponent as $int,
                    done: false,
                }
            }
        }

        impl Iterator for Powers {
            type Item = $int;

            fn next(&mut self) -> Option<Self::Item> {
                if self.done {
                    return None;
                }

                let ret = self.current;

                let (new, overflowed) = self.current.overflowing_mul(self.exponent);

                if overflowed {
                    self.done = true;
                } else {
                    self.current = new;
                }
                Some(ret)
            }
        }

        Powers::new($exponent)
    }};
}

macro_rules! build_array {
    ($int_type:ty, $name:expr, order = $order:expr, exponent = $exponent:expr) => {{

        let mut v = powers!($int_type, exponent = $exponent).map(|n| n as $int_type).collect::<Vec<_>>();

        if $order == Order::Descending {
            v.reverse();
        }

        let name = $name;
        let len = v.len();

        quote!(const #name: [$int_type; #len] = [#(#v),*];).into()
    }}
}

#[proc_macro]
pub fn table_of(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as Pow10);

    let name = input.name.clone();

    let order = input.order;
    let exponent = input.exponent;

    match &*input.yank_int_type().unwrap() {
        "usize" => build_array!(usize, name, order = order, exponent = exponent),
        "u64" => build_array!(u64, name, order = order, exponent = exponent),
        "u32" => build_array!(u32, name, order = order, exponent = exponent),
        "u16" => build_array!(u16, name, order = order, exponent = exponent),
        "u8" => build_array!(u8, name, order = order, exponent = exponent),

        ty => {
            let s = format!("table_of not implemented for type `{}`", ty);
            syn::Error::new(input.name.span(), s)
                .to_compile_error()
                .into()
        }
    }
}