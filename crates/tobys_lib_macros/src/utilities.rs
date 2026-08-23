use syn::parse::{Parse, ParseStream};

pub(crate) fn parse_zero_or_more<T: Parse>(input: ParseStream<'_>) -> Vec<T> {
    let mut result = Vec::new();
    while let Ok(item) = input.parse() {
        result.push(item);
    }
    result
}
