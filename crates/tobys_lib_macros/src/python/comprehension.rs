use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{
    Expr, Pat, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

use crate::utilities::parse_zero_or_more;

#[cfg_attr(feature = "extra-traits", derive(Debug, Eq, PartialEq))]
struct Comprehension {
    mapping: Mapping,
    for_if_clause: ForIfClause,
    additional_for_if_clauses: Vec<ForIfClause>,
}
#[cfg_attr(feature = "extra-traits", derive(Debug, Eq, PartialEq))]
struct Mapping(Expr);
#[cfg_attr(feature = "extra-traits", derive(Debug, Eq, PartialEq))]
struct ForIfClause {
    pattern: Pattern,
    target_list: Expr,
    conditions: Vec<Condition>,
}
#[cfg_attr(feature = "extra-traits", derive(Debug, Eq, PartialEq))]
struct Pattern(Pat);
#[cfg_attr(feature = "extra-traits", derive(Debug, Eq, PartialEq))]
struct Condition(Expr);

impl Parse for Comprehension {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            mapping: input.parse()?,
            for_if_clause: input.parse()?,
            additional_for_if_clauses: parse_zero_or_more(input),
        })
    }
}
impl Parse for Mapping {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        input.parse().map(Self)
    }
}
impl Parse for ForIfClause {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        _ = input.parse::<Token![for]>()?;
        let pattern = input.parse()?;
        _ = input.parse::<Token![in]>()?;
        let target_list = input.parse()?;
        let conditions = parse_zero_or_more(input);

        Ok(Self {
            pattern,
            target_list,
            conditions,
        })
    }
}
impl Parse for Pattern {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Pat::parse_single(input).map(Self)
    }
}
impl Parse for Condition {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        _ = input.parse::<Token![if]>()?;
        input.parse().map(Self)
    }
}

impl ToTokens for Comprehension {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let first = {
            let Mapping(mapping) = &self.mapping;
            let ForIfClause {
                pattern,
                target_list,
                conditions,
            } = &self.for_if_clause;

            let conditions = conditions.iter().map(|c| {
                let inner = &c.0;
                quote! { #inner }
            });
            quote! {
                ::core::iter::IntoIterator::into_iter(#target_list).filter_map(|#pattern| {
                    (true #(&& (#conditions))*).then(|| #mapping)
                })
            }
        };

        let output =
            self.additional_for_if_clauses
                .iter()
                .fold(first, |acc_output, curr| {
                    let ForIfClause {
                        pattern,
                        target_list,
                        conditions,
                    } = &curr;
                    quote! {
                        ::core::iter::IntoIterator::into_iter(#target_list).filter_map(|#pattern| {
                            (true #(&& (#conditions))*).then(|| #acc_output)
                        })
                        .flatten()
                    }
                });

        tokens.extend(output);
    }
}
impl ToTokens for Pattern {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.0.to_tokens(tokens);
    }
}
impl ToTokens for Condition {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.0.to_tokens(tokens);
    }
}

pub(crate) fn comprehension_impl(input: TokenStream) -> TokenStream {
    let c = parse_macro_input!(input as Comprehension);
    quote! { #c }.into()
}
