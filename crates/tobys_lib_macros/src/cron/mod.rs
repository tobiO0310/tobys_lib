use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    LitInt, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

use crate::utilities::parse_zero_or_more_with_separator;

#[derive(Debug, Clone, Copy)]
enum Multiple {
    Single(usize),
    Range(usize, usize),
}
impl Parse for Multiple {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let v = input.parse::<LitInt>()?.base10_parse()?;
        if input.parse::<Token![-]>().is_ok() {
            Ok(Self::Range(v, input.parse::<LitInt>()?.base10_parse()?))
        } else {
            Ok(Self::Single(v))
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum CronSection<const MIN: u8, const MAX: u8> {
    EveryTime,
    At(u8),
    Multiple([bool; 60]),
    EveryNth(u8),
    StartingAtXEveryNth(u8, u8),
}
#[expect(
    clippy::unwrap_used,
    clippy::as_conversions,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation
)]
impl<const MIN: u8, const MAX: u8> Parse for CronSection<MIN, MAX> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.parse::<Token![*]>().is_ok() {
            if input.parse::<Token![/]>().is_ok() {
                if let Ok(v) = input.parse::<LitInt>() {
                    let val = v.base10_parse()?;
                    if MIN > val || MAX < val {
                        return Err(syn::Error::new(
                            v.span(),
                            format!("{val} is not within {MIN} and {MAX}"),
                        ));
                    }
                    Ok(Self::EveryNth(val))
                } else {
                    Err(input.error("Unknown format"))
                }
            } else {
                Ok(Self::EveryTime)
            }
        } else {
            let multiple =
                parse_zero_or_more_with_separator::<Multiple, Token![,]>(input);
            if multiple.is_empty() {
                return Err(input.error("Unknown format"));
            } else if let Multiple::Single(v) = multiple.first().unwrap() {
                if input.parse::<Token![/]>().is_err() {
                    return Ok(Self::At(*v as u8));
                }
                return Ok(Self::StartingAtXEveryNth(
                    *v as u8,
                    input.parse::<LitInt>()?.base10_parse()?,
                ));
            }

            let mut arr = [false; 60];
            for multi in multiple {
                match multi {
                    Multiple::Single(v) => arr[v] = true,
                    Multiple::Range(min, max) => {
                        for item in
                            arr.iter_mut().take(max.saturating_add(1)).skip(min)
                        {
                            *item = true;
                        }
                    }
                }
            }

            Ok(Self::Multiple(arr))
        }
    }
}
impl<const MIN: u8, const MAX: u8> ToTokens for CronSection<MIN, MAX> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            Self::EveryTime =>
                quote! { ::tobys_lib::cron::CronSection::EveryTime },

            Self::At(v) => quote! { ::tobys_lib::cron::CronSection::At(#v) },
            Self::Multiple(v) => {
                quote! { ::tobys_lib::cron::CronSection::Multiple([#(#v),*])}
            },
            Self::EveryNth(v) =>
                quote! { ::tobys_lib::cron::CronSection::EveryNth(#v) },

            Self::StartingAtXEveryNth(x, n) =>
                quote! { ::tobys_lib::cron::CronSection::StartingAtXEveryNth(#x, #n)},

        });
    }
}

#[derive(Debug, Clone, Copy)]
struct CronSyntax(
    CronSection<0, 59>,
    CronSection<0, 23>,
    CronSection<1, 31>,
    CronSection<1, 12>,
    CronSection<0, 7>,
);

impl Parse for CronSyntax {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self(
            input.parse()?,
            input.parse()?,
            input.parse()?,
            input.parse()?,
            input.parse()?,
        ))
    }
}
impl ToTokens for CronSyntax {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let CronSyntax(minute, hour, day_of_month, month, day_of_week) = self;

        tokens.extend(quote! {
            const { ::tobys_lib::cron::CronTime::new_unchecked(&[
                #minute,
                #hour,
                #day_of_month,
                #month,
                #day_of_week
            ]) }
        });
    }
}

pub(crate) fn create_time_impl(input: TokenStream) -> TokenStream {
    let c = parse_macro_input!(input as CronSyntax);
    quote! { #c }.into()
}

// * * * * * || {}
struct JobSyntax {
    cron_syntax: CronSyntax,
    job: syn::ExprClosure,
}
impl Parse for JobSyntax {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let cron_syntax = input.parse()?;
        let job = input.parse()?;

        Ok(Self { cron_syntax, job })
    }
}
impl ToTokens for JobSyntax {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let syntax = &self.cron_syntax;
        let job = &self.job;
        tokens.extend(quote! {
            ::tobys_lib::cron::Job::new(#syntax, #job)
        });
    }
}

struct CreateJobsSyntax(Vec<JobSyntax>);
impl Parse for CreateJobsSyntax {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self(parse_zero_or_more_with_separator::<_, Token![;]>(
            input,
        )))
    }
}
impl ToTokens for CreateJobsSyntax {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let jobs = self.0.iter();

        tokens.extend(quote! {
            [#(#jobs),*]
        });
    }
}

pub(crate) fn create_jobs_impl(input: TokenStream) -> TokenStream {
    let c = parse_macro_input!(input as CreateJobsSyntax);
    quote! { #c }.into()
}
