use syn::parse::{Parse, ParseStream};

/// Parse zero or more of the required type
pub(crate) fn parse_zero_or_more<T: Parse>(input: ParseStream<'_>) -> Vec<T> {
    let mut result = Vec::new();
    while let Ok(item) = input.parse() {
        result.push(item);
    }
    result
}

/// Parse zero or more of the required with a separator.
pub(crate) fn parse_zero_or_more_with_separator<T: Parse, Sep: Parse>(
    input: ParseStream<'_>,
) -> Vec<T> {
    let mut result = Vec::new();
    if let Ok(item) = input.parse() {
        // parse first (if exists, else return empty)
        result.push(item);
    } else {
        return result;
    }
    // parse more, separated by Sep
    loop {
        if input.parse::<Sep>().is_err() {
            break;
        }
        if let Ok(item) = input.parse() {
            result.push(item);
        } else {
            break;
        }
    }
    result
}
