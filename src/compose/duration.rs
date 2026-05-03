//! Compose-style duration parsing. Output is nanoseconds (engine wire
//! unit). Split out from transform.rs so that file stays under the
//! 250-line cap.

use super::spec::ParseError;

/// Compose-style duration: `1h30m`, `500ms`, `10s`. Bare numbers are
/// interpreted as seconds, matching compose's behavior.
pub(super) fn parse_ns(s: &str) -> Result<i64, ParseError> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err(ParseError("empty duration".into()));
    }
    if let Ok(n) = trimmed.parse::<i64>() {
        return Ok(n.saturating_mul(1_000_000_000));
    }
    let mut total: i64 = 0;
    let mut num = String::new();
    let mut unit = String::new();
    for ch in trimmed.chars() {
        if ch.is_ascii_digit() {
            if !unit.is_empty() {
                total = total.saturating_add(consume_unit(&num, &unit)?);
                num.clear();
                unit.clear();
            }
            num.push(ch);
        } else {
            unit.push(ch);
        }
    }
    total = total.saturating_add(consume_unit(&num, &unit)?);
    Ok(total)
}

fn consume_unit(num: &str, unit: &str) -> Result<i64, ParseError> {
    let n: i64 = num
        .parse()
        .map_err(|_| ParseError(format!("bad duration component `{num}{unit}`")))?;
    let mult: i64 = match unit {
        "ns" => 1,
        "us" | "µs" => 1_000,
        "ms" => 1_000_000,
        "s" => 1_000_000_000,
        "m" => 60_000_000_000,
        "h" => 3_600_000_000_000,
        other => return Err(ParseError(format!("unknown duration unit `{other}`"))),
    };
    Ok(n.saturating_mul(mult))
}
