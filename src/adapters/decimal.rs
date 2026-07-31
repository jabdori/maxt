//! Exact decimal parsing for exchange payloads.
//!
//! Plain and scientific notation are accepted without rounding.

use rust_decimal::Decimal;

/// Maximum decimal-point shift expanded from scientific notation.
///
/// This bounds allocation before [`Decimal`] validates representability.
const MAX_POINT_SHIFT: i64 = 64;

/// Parses a decimal exactly, including scientific notation.
///
/// Scientific notation is expanded as text because arithmetic conversion may
/// round. Whitespace is ignored and unrepresentable values return an error.
pub(crate) fn exact(text: &str) -> Result<Decimal, rust_decimal::Error> {
    let text = text.trim();

    match Decimal::from_str_exact(text) {
        Ok(value) => Ok(value),
        // Retry only after expanding valid scientific notation.
        Err(err) => match without_exponent(text) {
            Some(plain) => Decimal::from_str_exact(&plain),
            None => Err(err),
        },
    }
}

/// Expands scientific notation without changing its digits or scale.
///
/// Returns `None` for invalid syntax or an expansion beyond the allocation
/// bound.
fn without_exponent(text: &str) -> Option<String> {
    let marker = text.find(['e', 'E'])?;
    let (mantissa, exponent) = text.split_at(marker);
    let exponent: i64 = exponent.get(1..)?.parse().ok()?;

    let (sign, unsigned) = match mantissa.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", mantissa.strip_prefix('+').unwrap_or(mantissa)),
    };
    let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    if whole.is_empty() && fraction.is_empty() {
        return None;
    }
    if !whole
        .bytes()
        .chain(fraction.bytes())
        .all(|b| b.is_ascii_digit())
    {
        return None;
    }

    // Negative positions require leading fractional zeroes.
    let digits = format!("{whole}{fraction}");
    let point = exponent.checked_add(i64::try_from(whole.len()).ok()?)?;
    if !(-MAX_POINT_SHIFT..=MAX_POINT_SHIFT).contains(&point) {
        return None;
    }

    Some(match usize::try_from(point) {
        Err(_) => format!(
            "{sign}0.{}{digits}",
            "0".repeat(point.unsigned_abs() as usize)
        ),
        Ok(0) => format!("{sign}0.{digits}"),
        Ok(point) if point >= digits.len() => {
            format!("{sign}{digits}{}", "0".repeat(point - digits.len()))
        }
        Ok(point) => format!("{sign}{}.{}", &digits[..point], &digits[point..]),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(text: &str) -> Decimal {
        exact(text).expect("a representable decimal")
    }

    #[test]
    fn an_exponent_names_the_same_number_as_the_digits_it_stands_for() {
        for (exponent_form, plain_form) in [
            ("8.428e-05", "0.00008428"),
            ("8.428E-5", "0.00008428"),
            ("1e5", "100000"),
            ("1.5E+3", "1500"),
            ("-1.2e-3", "-0.0012"),
            ("1230e-2", "12.30"),
            ("0.5e1", "5"),
            ("12.5e0", "12.5"),
            ("1e-28", "0.0000000000000000000000000001"),
        ] {
            assert_eq!(ok(exponent_form), ok(plain_form), "{exponent_form}");
        }
    }

    #[test]
    fn a_value_too_precise_to_hold_is_refused_in_either_spelling() {
        let plain = "123456.78901234567890123456789012";
        let exponent = "1.2345678901234567890123456789012e5";

        assert!(Decimal::from_str_exact(plain).is_err());
        assert!(exact(plain).is_err());
        assert!(
            exact(exponent).is_err(),
            "the exponent spelling of an unrepresentable number was rounded, not refused"
        );
    }

    #[test]
    fn a_value_too_small_to_hold_is_refused_rather_than_flattened_to_zero() {
        for text in ["1e-30", "0.000000000000000000000000000001"] {
            assert!(exact(text).is_err(), "{text}");
        }
    }

    #[test]
    fn an_exponent_no_number_could_survive_is_refused_before_it_is_expanded() {
        for text in ["1e2000000000", "1e-2000000000", "1e9223372036854775807"] {
            assert!(exact(text).is_err(), "{text}");
        }
    }

    #[test]
    fn text_that_is_not_a_number_stays_an_error() {
        for text in ["", "e5", "1e", "1e1e1", "abc", "1.2.3", "0x10", "1e2.5"] {
            assert!(exact(text).is_err(), "{text}");
        }
    }

    #[test]
    fn surrounding_whitespace_is_not_part_of_the_number() {
        assert_eq!(ok("  1.25  "), ok("1.25"));
        assert_eq!(ok("\t8.428e-05\n"), ok("0.00008428"));
    }

    #[test]
    fn the_scale_an_exchange_sent_survives_the_rewrite() {
        // Preserve the scale encoded by trailing zeroes.
        assert_eq!(ok("1.2300e2").to_string(), "123.00");
        assert_eq!(ok("100e-2").to_string(), "1.00");
    }
}
