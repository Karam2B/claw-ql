#![allow(unused)]
#![warn(unused_must_use)]
#![allow(non_camel_case_types)]

use std::{
    collections::{HashMap, HashSet},
    ops::Not,
};

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote, quote_spanned};
use syn::{
    Error as SErr, Expr, Ident, custom_keyword, parenthesized,
    parse::{Parse, ParseBuffer},
    parse2,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{self, Comma, Dot, Paren},
};

custom_keyword!(SELECT);
custom_keyword!(LINK);
custom_keyword!(FROM);
custom_keyword!(BASE);
custom_keyword!(WHERE);
custom_keyword!(RETURN);

pub struct statement {
    kw: SELECT,
    from_kw: FROM,
    from: Ident,
    links: Option<(LINK, Vec<Ident>)>,
    where_clause: Option<(WHERE, Vec<where_expression>)>,
    return_list: Option<(RETURN, Punctuated<Ident, Comma>)>,
    aliases: HashMap<Ident, Ident>,
}

pub struct where_expression {
    scope: Option<Ident>,
    this: Ident,
    fn_ident: Ident,
    expr: Expr,
}

fn parse(input: syn::parse::ParseStream) -> syn::Result<statement> {
    let mut aliases = HashMap::<Ident, Ident>::new();
    let kw = input.parse::<SELECT>()?;

    let from_kw = input.parse()?;
    let from: Ident = input.parse()?;

    if input.peek(Ident) {
        aliases.insert(input.parse::<Ident>()?, from.clone());
    }

    let links = if input.peek(LINK) {
        let kw = input.parse::<LINK>()?;
        let mut idents = vec![input.parse::<Ident>()?];
        while input.peek(Comma) {
            let c = input.parse::<Comma>()?;
            idents.push(input.parse()?);
        }
        Some((kw, idents))
    } else {
        None
    };

    let where_clause = if input.peek(WHERE) {
        let kw = input.parse::<WHERE>()?;
        let mut predicates: Vec<where_expression> = vec![];
        loop {
            let first = input.parse::<Ident>()?;
            let dot = input.parse::<Dot>()?;
            let second = input.parse::<Ident>()?;
            let possible_third = if input.peek(Dot) {
                let dot = input.parse::<Dot>()?;
                let dot = input.parse::<Ident>()?;
                Some(dot)
            } else {
                None
            };

            let (pren, rest) = parse_parens(&input)?;
            let expr = <syn::Expr as Parse>::parse(&rest)?;

            let s = if let Some(fn_ident) = possible_third {
                predicates.push(where_expression {
                    scope: Some(first),
                    this: second,
                    fn_ident,
                    expr,
                })
            } else {
                predicates.push(where_expression {
                    scope: None,
                    this: first,
                    fn_ident: second,
                    expr,
                })
            };

            if rest.is_empty().not() {
                return err(rest.span(), "expected nothing but found {..}");
            }

            if input.peek(Comma) {
                input.parse::<Comma>()?;
                continue;
            } else if input.is_empty() {
                break;
            } else if input.peek(RETURN) {
                break;
            } else {
                return Err(SErr::new(
                    input.span(),
                    "expected either , or return but found something else",
                ));
            }
        }

        Some((kw, predicates))
    } else {
        None
    };

    let return_list = if input.peek(RETURN) {
        let kw = input.parse::<RETURN>()?;
        let mut list: Punctuated<Ident, Comma> = Default::default();
        while input.is_empty().not() {
            if input.peek(FROM) {
                break;
            }
            let ident = input.parse::<Ident>()?;
            if input.peek(Comma) {
                list.push(ident);
                list.push_punct(input.parse()?);
                continue;
            } else {
                break;
            }
        }
        Some((kw, list))
    } else {
        None
    };

    Ok(statement {
        kw,
        from,
        links,
        where_clause,
        from_kw,
        return_list,
        aliases,
    })
}

struct Ctx<'a> {
    aliases: &'a HashMap<Ident, Ident>,
    from: &'a Ident,
}

// fn where_to_token(
//     ctx: Ctx,
//     where_clause: &Option<(WHERE, Vec<where_expression>)>,
// ) -> Result<TokenStream, SErr> {
//     if let Some(wc) = where_clause {
//         let mut ino = Vec::<TokenStream>::new();
//         for where_expression {
//             scope,
//             this,
//             fn_ident,
//             expr,
//         } in wc.1.iter()
//         {
//             let span = this.span();

//             let _from = if let Some(alias) = scope {
//                 let s = if let Some(found) = ctx.aliases.get(alias) {
//                     found
//                 } else {
//                     return err(
//                         alias,
//                         format!(
//                             "cannot find this aliase, all are {}",
//                             ctx.aliases
//                                 .keys()
//                                 .map(|e| e.to_string())
//                                 .collect::<Vec<_>>()
//                                 .join(", ")
//                         ),
//                     );
//                 };
//                 Ident::new(&format!("{}_members", s.to_string()), s.span())
//             } else {
//                 Ident::new(
//                     &format!("{}_members", ctx.from.to_string()),
//                     ctx.from.span(),
//                 )
//             };
//             ino.push(quote_spanned!(span=>
//                 Box::new(#fn_ident (#_from::#this, #expr)) as Box<dyn WhereExpression>,
//             ));
//         }
//         Ok(quote! {
//             {
//                 let mut v = Vec::<Box<dyn WhereExpression>>::new();

//                 #(v.push(#ino);)*

//                 v
//             }
//         })
//     } else {
//         Ok(quote! {
//             Vec::<Box<dyn WhereExpression>>::new()
//         })
//     }
// }
fn where_to_token(
    ctx: Ctx,
    where_clause: &Option<(WHERE, Vec<where_expression>)>,
) -> Result<TokenStream, SErr> {
    if let Some(wc) = where_clause {
        let mut ino = Vec::<TokenStream>::new();
        for where_expression {
            scope,
            this,
            fn_ident,
            expr,
        } in wc.1.iter()
        {
            let span = this.span();

            let _from = if let Some(alias) = scope {
                let s = if let Some(found) = ctx.aliases.get(alias) {
                    found
                } else {
                    return err(
                        alias,
                        format!(
                            "cannot find this aliase, all are {}",
                            ctx.aliases
                                .keys()
                                .map(|e| e.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                    );
                };
                Ident::new(&format!("{}_members", s.to_string()), s.span())
            } else {
                Ident::new(
                    &format!("{}_members", ctx.from.to_string()),
                    ctx.from.span(),
                )
            };
            ino.push(quote_spanned!(span=>
                #fn_ident (#_from::#this, #expr)
            ));
        }
        Ok(quote! { (#(#ino,)*) })
    } else {
        Ok(quote! { () })
    }
}

fn to_token(
    statement {
        kw,
        from_kw,
        from,
        links,
        where_clause,
        return_list,
        aliases,
    }: &statement,
    token: &mut TokenStream,
) -> Result<(), SErr> {
    let _from = Ident::new(&format!("_{}", from.to_string()), from.span());

    let where_clause_ = where_to_token(Ctx { aliases, from }, where_clause)?;

    let links = if let Some((_, links)) = links {
        quote!( (#(#links,)*))
    } else {
        quote! { () }
    };

    let mut out = token.extend(quote!({
        // use prelude::*;
        Select {
            from: #from,
            wheres: #where_clause_,
            limit: (),
            link: #links,
            returning: ()
        }
    }));

    Ok(())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use quote::{ToTokens, quote};
    use syn::spanned::Spanned;

    use crate::sql_mod::statement;

    #[test]
    fn main() {
        let input = quote!(SELECT FROM todo LINK has_category WHERE title.eq(s) RETURN title);

        let err = syn::parse2::<statement>(input).err().unwrap().to_string();
        assert_eq!(err, String::from("should be empty for now"));

        let input = quote!(
            SELECT FROM todo
            LINK has_category
            WHERE title.eq()
            RETURN title
            LIMIT 3
        );

        let expect = syn::parse2::<statement>(input)
            .unwrap()
            .into_token_stream()
            .to_string();

        let to_be = quote!(
            const {
                use prelude::*;
                Select {
                    from: todo,
                    wheres: (),
                }
            }
        )
        .to_string();

        pretty_assertions::assert_eq!(expect, to_be);
    }
}

// reduce indentation!
impl Parse for statement {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        parse(input)
    }
}
impl ToTokens for statement {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Err(er) = to_token(self, tokens) {
            tokens.extend(er.to_compile_error());
        }
    }
}

// utils
fn parse_parens<'a>(input: syn::parse::ParseStream<'a>) -> syn::Result<(Paren, ParseBuffer<'a>)> {
    syn::__private::parse_parens(input).map(|e| return (e.token, e.content))
}

fn err<T, S: Spanned, D: ToString>(span: S, str: D) -> Result<T, syn::Error> {
    return Err(syn::Error::new(span.span(), str.to_string()));
}
