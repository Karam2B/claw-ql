#![allow(unused)]
#![warn(unused_must_use)]
#![allow(non_camel_case_types)]

use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
    ops::Not,
};

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote, quote_spanned};
use syn::{
    Error as SErr, Expr, Ident, parenthesized,
    parse::{Parse, ParseBuffer, ParseStream},
    parse2,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{self, Comma, Dot, Paren},
};

use crate::sql_mod::{
    kws::{FROM, SELECT, WITH},
    link_mod::LinkSegment,
    wheres_mod::{WhereScope, WhereSegment},
};

pub mod kws {
    use syn::custom_keyword;

    custom_keyword!(SELECT);
    custom_keyword!(LINK);
    custom_keyword!(FROM);
    custom_keyword!(BASE);
    custom_keyword!(WHERE);
    custom_keyword!(RETURN);
    custom_keyword!(WITH);
}

trait MyParse: Sized {
    type Scope;
    fn parse_scope(scope: Self::Scope, input: ParseStream) -> syn::Result<Self>;
    fn parse(this: PhantomData<Self>, scope: Self::Scope, input: ParseStream) -> syn::Result<Self> {
        Self::parse_scope(scope, input)
    }
}

pub struct MainStatement {
    main: MainEnum,
    with: Ident,
}

enum MainEnum {
    FetchOne {
        base: Ident,
        wheres: WhereSegment,
        links: LinkSegment,
    },
}

impl MainEnum {
    fn options() -> Vec<&'static str> {
        vec!["SELECT FROM", "INSERT INTO"]
    }
}

pub fn parse_main_statement(input: ParseStream) -> syn::Result<MainStatement> {
    let main = if input.peek(SELECT) {
        input.parse::<SELECT>()?;
        input.parse::<FROM>()?;

        let base = input.parse::<Ident>()?;

        let links = MyParse::parse(PhantomData::<LinkSegment>, (), input)?;
        let wheres = MyParse::parse(
            PhantomData::<WhereSegment>,
            WhereScope { base: base.clone() },
            input,
        )?;

        MainEnum::FetchOne {
            base,
            wheres,
            links,
        }
    } else {
        return match input.cursor().token_tree() {
            Some(i) => Err(syn::Error::new(
                i.0.span(),
                format!(
                    "expected '{}' found '{}'",
                    MainEnum::options().join(", "),
                    i.0.to_string()
                ),
            )),
            None => return Err(input.error("unexpected token")),
        };
    };

    let with = if input.peek(WITH) {
        input.parse::<WITH>()?;
        input.parse::<Ident>()?
    } else {
        Ident::new("pool", Span::call_site())
    };

    if input.is_empty().not() {
        let tt = input.cursor().token_tree().unwrap();
        return Err(syn::Error::new(
            tt.0.span(),
            format!("end of input found '{}'", tt.0.to_string()),
        ));
    }

    Ok(MainStatement { main, with })
}

impl Parse for MainStatement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        parse_main_statement(input)
    }
}

pub fn main_statment_to_token(input: MainStatement) -> TokenStream {
    let main = match &input.main {
        MainEnum::FetchOne {
            base,
            wheres,
            links,
        } => {
            quote! {
                FetchOne {
                    base: #base,
                    links: #links,
                    wheres: #wheres,
                }
            }
        }
    };
    let with = &input.with;
    quote!({
        use ::claw_ql::prelude::sql::*;
        let op = is_valid_syntax(#main, infer_db(&#with));

        Operation::exec(op, #with)
    })
}

mod link_mod {
    use crate::sql_mod::{MyParse, kws::LINK};
    use quote::{ToTokens, quote};
    use syn::Ident;

    #[derive(Clone)]
    pub struct LinkSegment {
        inner: Vec<Ident>,
    }

    impl MyParse for LinkSegment {
        type Scope = ();
        fn parse_scope(scope: Self::Scope, input: syn::parse::ParseStream) -> syn::Result<Self> {
            let mut links = vec![];
            while input.peek(LINK) {
                input.parse::<LINK>()?;
                let i = input.parse::<Ident>()?;
                links.push(i);
            }

            return Ok(LinkSegment { inner: links });
        }
    }

    impl ToTokens for LinkSegment {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let idents = self.inner.clone();
            tokens.extend(quote! {
                (#(#idents,)*)
            });
        }
    }
}

mod wheres_mod {
    use proc_macro2::TokenStream;
    use quote::ToTokens;
    use quote::quote;
    use quote::quote_spanned;
    use syn::token::Where;
    use syn::{Ident, token::Dot};

    use crate::{
        local_lib::parse_parens,
        sql_mod::{MyParse, kws::WHERE},
    };

    pub struct WhereSegment {
        ands: Vec<TokenStream>,
    }

    pub struct WhereScope {
        pub base: Ident,
    }
    impl MyParse for WhereSegment {
        type Scope = WhereScope;
        fn parse_scope(scope: Self::Scope, input: syn::parse::ParseStream) -> syn::Result<Self> {
            // if input.peek(WH);

            if input.peek(WHERE) {
                input.parse::<WHERE>()?;
                let i1 = input.parse::<Ident>()?;
                input.parse::<Dot>()?;
                let i2 = input.parse::<Ident>()?;
                let (scope, member, method) = if input.peek(Dot) {
                    input.parse::<Dot>()?;
                    let i3 = input.parse::<Ident>()?;

                    let members_mod =
                        Ident::new(&format!("{}_members", i2.to_string().as_str()), i2.span());

                    (members_mod, i2, i3)
                } else {
                    let members_mod = Ident::new(
                        &format!("{}_members", scope.base.to_string().as_str()),
                        scope.base.span(),
                    );

                    let member =
                        Ident::new(&format!("{}_members", i1.to_string().as_str()), i1.span());

                    (members_mod, i1, i2)
                };

                let (_, input) = parse_parens(input)?;

                let expr = input.parse::<syn::Expr>()?;

                let member_spanned = quote_spanned! {member.span()=> member(#scope::#member)};

                return Ok(Self {
                    ands: vec![quote!(member(#scope::#member).#method(#expr))],
                });
            }
            Ok(Self { ands: vec![] })
        }
    }

    impl ToTokens for WhereSegment {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let s = &self.ands;
            tokens.extend(quote! ((#(#s,)*)))
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use quote::{ToTokens, quote};
    use syn::spanned::Spanned;

    use crate::sql_mod::{MainStatement, main_statment_to_token};

    #[test]
    fn simple() {
        let expect = quote!(
            SELECT FROM todo
            LINK category
            WHERE title.eq("first_todo")

        );

        let expect = match syn::parse2::<MainStatement>(expect) {
            Ok(ok) => main_statment_to_token(ok),
            Err(e) => e.to_compile_error(),
        };

        let to_be = quote!({
            use ::claw_ql::prelude::sql::*;
            let op = is_valid_syntax(
                FetchOne {
                    base: todo,
                    link: (category,),
                    wheres: (member(todo_members::title).eq("first_todo"),),
                },
                infer_db(&pool),
            );

            Operation::exec(op, pool)
        });

        pretty_assertions::assert_eq!(expect.to_string(), to_be.to_string());
    }
}

// reduce indentation!

// utils
fn parse_parens<'a>(input: syn::parse::ParseStream<'a>) -> syn::Result<(Paren, ParseBuffer<'a>)> {
    syn::__private::parse_parens(input).map(|e| return (e.token, e.content))
}

fn err<T, S: Spanned, D: ToString>(span: S, str: D) -> Result<T, syn::Error> {
    return Err(syn::Error::new(span.span(), str.to_string()));
}
