fn decode_entity(entity: &str) -> Option<char> {
    match entity {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" | "#39" => Some('\''),
        "#34" => Some('"'),
        _ => {
            let digits = entity.strip_prefix('#')?;
            if let Some(hex) = digits.strip_prefix('x').or_else(|| digits.strip_prefix('X')) {
                char::from_u32(u32::from_str_radix(hex, 16).ok()?)
            } else {
                char::from_u32(digits.parse::<u32>().ok()?)
            }
        }
    }
}

pub fn clean_bbcode(raw: &str) -> String {
    let mut text = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(open) = rest.find('[') {
        text.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        match after.find(']') {
            Some(close) => {
                rest = &after[close + 1..];
            }
            None => {
                text.push('[');
                rest = after;
            }
        }
    }
    text.push_str(rest);
    let mut decoded = String::with_capacity(text.len());
    let mut rest = text.as_str();
    while let Some(open) = rest.find('&') {
        decoded.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        match after.find(';') {
            Some(close) if close < 16 => {
                let entity = &after[..close];
                match decode_entity(entity) {
                    Some(c) => decoded.push(c),
                    None => {
                        decoded.push('&');
                        decoded.push_str(entity);
                        decoded.push(';');
                    }
                }
                rest = &after[close + 1..];
            }
            _ => {
                decoded.push('&');
                rest = after;
            }
        }
    }
    decoded.push_str(rest);
    decoded
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markup_strips_but_text_stays() {
        assert_eq!(
            clean_bbcode("A [b]bold[/b] tale with [spoiler]secret[/spoiler]."),
            "A bold tale with secret."
        );
    }

    #[test]
    fn tags_with_parameters_strip() {
        assert_eq!(
            clean_bbcode("[url=https://example.com]link[/url] and [color=red]hue[/color]"),
            "link and hue"
        );
    }

    #[test]
    fn unclosed_bracket_stays_literal() {
        assert_eq!(clean_bbcode("a [b broken"), "a [b broken");
        assert_eq!(clean_bbcode("5 > 3 and 2 < 4"), "5 > 3 and 2 < 4");
    }

    #[test]
    fn entities_decode() {
        assert_eq!(
            clean_bbcode("Fish &amp; Chips &lt;b&gt; &#39;hi&#39; &#x41;"),
            "Fish & Chips <b> 'hi' A"
        );
    }

    #[test]
    fn unknown_entities_stay_literal() {
        assert_eq!(clean_bbcode("&bogus; &;"), "&bogus; &;");
    }

    #[test]
    fn whitespace_collapses() {
        assert_eq!(clean_bbcode("line one\nline   two"), "line one line two");
        assert_eq!(clean_bbcode(""), "");
    }
}
