use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{map, opt},
    error::ErrorKind,
    IResult,
};

/// A PDF token produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    /// Integer literal: `42`, `-7`
    Integer(i64),
    /// Real number: `3.14`, `-0.5`
    Real(f64),
    /// Literal string: `(hello)`. Stored with decoded content.
    String(Vec<u8>),
    /// Hex string: `<48656C6C6F>`. Stored decoded.
    HexString(Vec<u8>),
    /// Name object: `/Type`. Stored without the leading `/`.
    Name(Vec<u8>),
    /// Array start: `[`
    ArrayStart,
    /// Array end: `]`
    ArrayEnd,
    /// Dictionary start: `<<`
    DictStart,
    /// Dictionary end: `>>`
    DictEnd,
    /// Stream keyword
    Stream,
    /// Null object
    Null,
    /// Boolean: `true` or `false`
    Boolean(bool),
    /// Comment text
    Comment(&'a [u8]),
    /// End of input
    Eof,
}

/// Skip PDF whitespace and comments, returning remaining input.
pub fn skip_ws(input: &[u8]) -> &[u8] {
    let mut i = input;
    loop {
        let len = i
            .iter()
            .position(|&b| !matches!(b, b' ' | b'\t' | b'\r' | b'\n' | b'\x0c' | b'\x00'))
            .unwrap_or(i.len());
        i = &i[len..];

        if let Some(rest) = i.strip_prefix(b"%") {
            if let Some(pos) = rest.iter().position(|&b| b == b'\r' || b == b'\n') {
                i = &rest[pos..];
                if let Some(r) = i.strip_prefix(b"\r\n") {
                    i = r;
                } else {
                    i = &i[1..];
                }
                continue;
            } else {
                i = b"";
                break;
            }
        }
        break;
    }
    i
}

/// Parse an integer: optional sign + digits.
pub fn parse_integer(input: &[u8]) -> IResult<&[u8], i64> {
    let (input, sign) = opt(alt((tag("-"), tag("+"))))(input)?;
    let (input, digits) = take_ascii_digits(input)?;

    let num_str = std::str::from_utf8(digits)
        .map_err(|_| nom::Err::Error(nom::error::make_error(input, ErrorKind::Digit)))?;
    let mut val: i64 = num_str
        .parse()
        .map_err(|_| nom::Err::Error(nom::error::make_error(input, ErrorKind::Digit)))?;
    if sign == Some(&b"-"[..]) {
        val = -val;
    }
    Ok((input, val))
}

/// Parse a real number: optional sign + digits + `.` + optional digits, or `.` + digits.
pub fn parse_real(input: &[u8]) -> IResult<&[u8], f64> {
    let (input, sign) = opt(alt((tag("-"), tag("+"))))(input)?;

    // We need to recognize: digits.digits | digits. | .digits
    let (input, number) = recognize_real_number(input)?;

    let num_str = std::str::from_utf8(number)
        .map_err(|_| nom::Err::Error(nom::error::make_error(input, ErrorKind::Float)))?;
    let mut val: f64 = num_str
        .parse()
        .map_err(|_| nom::Err::Error(nom::error::make_error(input, ErrorKind::Float)))?;
    if sign == Some(&b"-"[..]) {
        val = -val;
    }
    Ok((input, val))
}

fn take_ascii_digits(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let end = input
        .iter()
        .position(|&b| !b.is_ascii_digit())
        .unwrap_or(input.len());
    if end == 0 {
        return Err(nom::Err::Error(nom::error::make_error(
            input,
            ErrorKind::Digit,
        )));
    }
    Ok((&input[end..], &input[..end]))
}

fn recognize_real_number(input: &[u8]) -> IResult<&[u8], &[u8]> {
    // Count leading digits
    let digit_end = input
        .iter()
        .position(|&b| !b.is_ascii_digit())
        .unwrap_or(input.len());

    if digit_end < input.len() && input[digit_end] == b'.' {
        // digits.digits or digits.
        let after_dot = &input[digit_end + 1..];
        let frac_end = after_dot
            .iter()
            .position(|&b| !b.is_ascii_digit())
            .unwrap_or(after_dot.len());
        let total = digit_end + 1 + frac_end;
        if total == digit_end + 1 && digit_end == 0 {
            return Err(nom::Err::Error(nom::error::make_error(
                input,
                ErrorKind::Float,
            )));
        }
        Ok((&input[total..], &input[..total]))
    } else if digit_end == 0 && input.starts_with(b".") {
        // .digits
        let after_dot = &input[1..];
        let frac_end = after_dot
            .iter()
            .position(|&b| !b.is_ascii_digit())
            .unwrap_or(after_dot.len());
        if frac_end == 0 {
            return Err(nom::Err::Error(nom::error::make_error(
                input,
                ErrorKind::Float,
            )));
        }
        let total = 1 + frac_end;
        Ok((&input[total..], &input[..total]))
    } else {
        Err(nom::Err::Error(nom::error::make_error(
            input,
            ErrorKind::Float,
        )))
    }
}

/// Parse a literal string `(hello (nested) world)`.
pub fn parse_literal_string(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let (input, _) = char('(')(input)?;
    let mut result = Vec::new();
    let mut depth: i32 = 1;
    let mut i = 0;

    while i < input.len() {
        match input[i] {
            b'(' => {
                depth += 1;
                result.push(b'(');
                i += 1;
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Ok((&input[i + 1..], result));
                }
                result.push(b')');
                i += 1;
            }
            b'\\' => {
                if i + 1 >= input.len() {
                    result.push(b'\\');
                    break;
                }
                let escaped = match input[i + 1] {
                    b'n' => b'\n',
                    b'r' => b'\r',
                    b't' => b'\t',
                    b'b' => 0x08,
                    b'f' => 0x0C,
                    b'(' => b'(',
                    b')' => b')',
                    b'\\' => b'\\',
                    c if (b'0'..=b'7').contains(&c) => {
                        let mut val = (c - b'0') as u32;
                        let mut j = i + 2;
                        for _ in 0..2 {
                            if j < input.len() && input[j] >= b'0' && input[j] <= b'7' {
                                val = val * 8 + (input[j] - b'0') as u32;
                                j += 1;
                            } else {
                                break;
                            }
                        }
                        i = j - 1;
                        (val & 0xFF) as u8
                    }
                    b'\r' => {
                        i += 2;
                        if i < input.len() && input[i] == b'\n' {
                            i += 1;
                        }
                        continue;
                    }
                    b'\n' => {
                        i += 2;
                        continue;
                    }
                    _ => input[i + 1],
                };
                result.push(escaped);
                i += 2;
            }
            b'\r' => {
                result.push(b'\n');
                i += 1;
                if i < input.len() && input[i] == b'\n' {
                    i += 1;
                }
            }
            _ => {
                result.push(input[i]);
                i += 1;
            }
        }
    }
    Err(nom::Err::Error(nom::error::make_error(
        input,
        ErrorKind::Tag,
    )))
}

/// Parse a hex string `<48656C6C6F>`.
pub fn parse_hex_string(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let (input, _) = char('<')(input)?;
    let mut result = Vec::new();
    let mut hex_buf = String::new();
    let mut i = 0;

    while i < input.len() {
        match input[i] {
            b'>' => {
                if !hex_buf.is_empty() {
                    hex_buf.push('0');
                    result.push(u8::from_str_radix(&hex_buf, 16).unwrap());
                    hex_buf.clear();
                }
                return Ok((&input[i + 1..], result));
            }
            b => {
                let c = b as char;
                if c.is_ascii_hexdigit() {
                    hex_buf.push(c.to_ascii_lowercase());
                    if hex_buf.len() == 2 {
                        result.push(u8::from_str_radix(&hex_buf, 16).unwrap());
                        hex_buf.clear();
                    }
                }
                // Skip whitespace in hex strings
            }
        }
        i += 1;
    }
    Err(nom::Err::Error(nom::error::make_error(
        input,
        ErrorKind::Tag,
    )))
}

/// Parse a name object `/Type` or `/Name#20WithSpaces`.
pub fn parse_name_raw(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let (input, _) = char('/')(input)?;
    let mut result = Vec::new();
    let mut i = 0;

    while i < input.len() {
        let b = input[i];
        if matches!(
            b,
            b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%'
        ) {
            break;
        }
        if matches!(b, b' ' | b'\t' | b'\r' | b'\n' | b'\x0c' | b'\x00') {
            break;
        }
        if b == b'#' {
            if i + 2 < input.len() {
                let hex = &input[i + 1..i + 3];
                if let Ok(s) = std::str::from_utf8(hex) {
                    if let Ok(val) = u8::from_str_radix(s, 16) {
                        result.push(val);
                        i += 3;
                        continue;
                    }
                }
            }
            break;
        }
        result.push(b);
        i += 1;
    }

    if result.is_empty() {
        return Err(nom::Err::Error(nom::error::make_error(
            input,
            ErrorKind::Tag,
        )));
    }
    Ok((&input[i..], result))
}

/// Parse the next token from the input.
pub fn next_token(input: &[u8]) -> IResult<&[u8], Token<'_>> {
    let input = skip_ws(input);

    if input.is_empty() {
        return Ok((input, Token::Eof));
    }

    // Try keywords first
    if input.starts_with(b"<<") {
        return Ok((&input[2..], Token::DictStart));
    }
    if input.starts_with(b">>") {
        return Ok((&input[2..], Token::DictEnd));
    }

    // Check for stream keyword (must be followed by whitespace/EOL)
    if input.starts_with(b"stream") {
        let after = &input[6..];
        if after.is_empty() || after[0] == b'\r' || after[0] == b'\n' {
            return Ok((&input[6..], Token::Stream));
        }
    }

    if input.starts_with(b"true") && is_token_boundary(input, 4) {
        return Ok((&input[4..], Token::Boolean(true)));
    }
    if input.starts_with(b"false") && is_token_boundary(input, 5) {
        return Ok((&input[5..], Token::Boolean(false)));
    }
    if input.starts_with(b"null") && is_token_boundary(input, 4) {
        return Ok((&input[4..], Token::Null));
    }

    if input[0] == b'[' {
        return Ok((&input[1..], Token::ArrayStart));
    }
    if input[0] == b']' {
        return Ok((&input[1..], Token::ArrayEnd));
    }

    // Comment
    if input[0] == b'%' {
        let end = input
            .iter()
            .position(|&b| b == b'\r' || b == b'\n')
            .unwrap_or(input.len());
        return Ok((&input[end..], Token::Comment(&input[..end])));
    }

    // Name
    if input[0] == b'/' {
        return map(parse_name_raw, Token::Name)(input);
    }

    // Hex string (starts with < but not <<)
    if input[0] == b'<' && (input.len() < 2 || input[1] != b'<') {
        return map(parse_hex_string, Token::HexString)(input);
    }

    // Literal string
    if input[0] == b'(' {
        return map(parse_literal_string, Token::String)(input);
    }

    // Number (integer or real)
    if input[0].is_ascii_digit() || input[0] == b'-' || input[0] == b'+' || input[0] == b'.' {
        // Peek to see if it's real or integer
        let start = if input[0] == b'-' || input[0] == b'+' {
            1
        } else {
            0
        };
        let rest = &input[start..];
        let has_dot = rest
            .iter()
            .take_while(|&&b| b.is_ascii_digit() || b == b'.')
            .any(|&b| b == b'.');
        if has_dot {
            return map(parse_real, Token::Real)(input);
        } else {
            // Could be integer, but could also be a real that starts with digit then .
            // Let's try integer first
            if let Ok(result) = map(parse_integer, Token::Integer)(input) {
                return Ok(result);
            }
            return map(parse_real, Token::Real)(input);
        }
    }

    // Bare keyword (regular characters that aren't a recognized token type)
    // In PDF, these include: R, obj, endobj, stream, endstream, xref, trailer, startxref, etc.
    // We parse them as Name tokens (without the / prefix).
    if input[0].is_ascii_alphabetic() {
        let end = input
            .iter()
            .position(|&b| !(b.is_ascii_alphanumeric() || b == b'_'))
            .unwrap_or(input.len());
        if end > 0 {
            return Ok((&input[end..], Token::Name(input[..end].to_vec())));
        }
    }

    Err(nom::Err::Error(nom::error::make_error(
        input,
        ErrorKind::NoneOf,
    )))
}

fn is_token_boundary(input: &[u8], len: usize) -> bool {
    if input.len() <= len {
        return true;
    }
    let b = input[len];
    matches!(
        b,
        b' ' | b'\t'
            | b'\r'
            | b'\n'
            | b'\x0c'
            | b'\x00'
            | b'('
            | b')'
            | b'<'
            | b'>'
            | b'['
            | b']'
            | b'{'
            | b'}'
            | b'/'
            | b'%'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer() {
        assert_eq!(parse_integer(b"42 rest"), Ok((&b" rest"[..], 42)));
        assert_eq!(parse_integer(b"-7 rest"), Ok((&b" rest"[..], -7)));
        assert_eq!(parse_integer(b"+3 rest"), Ok((&b" rest"[..], 3)));
    }

    #[test]
    fn test_real() {
        let (rest, val) = parse_real(b"3.14 rest").unwrap();
        #[allow(clippy::approx_constant)]
        let expected = 3.14;
        assert!((val - expected).abs() < f64::EPSILON);
        assert_eq!(rest, b" rest");

        let (rest, val) = parse_real(b"-0.5 rest").unwrap();
        assert!((val - (-0.5)).abs() < f64::EPSILON);
        assert_eq!(rest, b" rest");

        let (rest, val) = parse_real(b".5 rest").unwrap();
        assert!((val - 0.5).abs() < f64::EPSILON);
        assert_eq!(rest, b" rest");
    }

    #[test]
    fn test_literal_string() {
        let (rest, content) = parse_literal_string(b"(hello) rest").unwrap();
        assert_eq!(content, b"hello");
        assert_eq!(rest, b" rest");
    }

    #[test]
    fn test_hex_string() {
        let (rest, content) = parse_hex_string(b"<48656C6C6F> rest").unwrap();
        assert_eq!(content, b"Hello");
        assert_eq!(rest, b" rest");
    }

    #[test]
    fn test_name_raw() {
        let (rest, name) = parse_name_raw(b"/Type rest").unwrap();
        assert_eq!(name, b"Type");
        assert_eq!(rest, b" rest");

        let (rest, name) = parse_name_raw(b"/Name#20WithSpaces rest").unwrap();
        assert_eq!(name, b"Name WithSpaces");
        assert_eq!(rest, b" rest");
    }

    #[test]
    fn test_skip_ws() {
        assert_eq!(skip_ws(b"  \t\n  hello"), b"hello");
        assert_eq!(skip_ws(b"% comment\nhello"), b"hello");
        assert_eq!(skip_ws(b"hello"), b"hello");
    }

    #[test]
    fn test_next_token() {
        let (rest, tok) = next_token(b"42").unwrap();
        assert_eq!(tok, Token::Integer(42));
        assert!(rest.is_empty());

        let (_, tok) = next_token(b"<<").unwrap();
        assert_eq!(tok, Token::DictStart);

        let (_, tok) = next_token(b"/Type").unwrap();
        match tok {
            Token::Name(n) => assert_eq!(n, b"Type"),
            _ => panic!("Expected Name token"),
        }

        let (_, tok) = next_token(b"true").unwrap();
        assert_eq!(tok, Token::Boolean(true));

        let (_, tok) = next_token(b"null").unwrap();
        assert_eq!(tok, Token::Null);
    }
}
