//! Parsing and normalization of name-list source files.
//!
//! A source file looks like this:
//!
//! ```text
//! # comments start with a hash
//! [dwarves]
//! thorin, BALIN
//! dwalin
//!
//! [elves]
//!    legolas
//! ```
//!
//! Blank lines and comment lines are ignored. Everything else must belong
//! to a `[category]` section. Within a section, one or more comma
//! separated names may appear per line; each one is trimmed, has its
//! internal whitespace collapsed, and is title-cased.

use std::collections::BTreeMap;
use std::fmt;

/// A parse or validation failure, located precisely in the source text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatError {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}: {}", self.line, self.column, self.message)
    }
}

impl std::error::Error for FormatError {}

/// The result of parsing a source file: names grouped by category.
///
/// Categories are stored in a `BTreeMap` so that rendering is always in a
/// stable, alphabetical order regardless of the order names were declared
/// in.
#[derive(Debug, Default, Clone)]
pub struct NameList {
    pub categories: BTreeMap<String, Vec<String>>,
}

/// Parse a source file into a [`NameList`].
///
/// Parsing does not stop at the first problem. It collects every error it
/// finds so a caller can report them all at once, the way a compiler does.
pub fn parse(input: &str) -> Result<NameList, Vec<FormatError>> {
    let mut categories: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut current: Option<String> = None;
    let mut errors = Vec::new();

    for (line_idx, raw_line) in input.lines().enumerate() {
        let line_no = line_idx + 1;
        // Trailing whitespace never matters; leading whitespace does, since
        // we report columns against it.
        let line = raw_line.trim_end();
        let trimmed = line.trim_start();
        let indent = line.chars().count() - trimmed.chars().count();
        let start_col = indent + 1;

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with('[') {
            match parse_header(trimmed, line_no, start_col) {
                Ok(name) => {
                    categories.entry(name.clone()).or_default();
                    current = Some(name);
                }
                Err(e) => errors.push(e),
            }
            continue;
        }

        let Some(category) = current.clone() else {
            errors.push(FormatError {
                line: line_no,
                column: start_col,
                message: "name given before any [category] header".to_string(),
            });
            continue;
        };

        let mut col = start_col;
        for piece in trimmed.split(',') {
            let piece_start_col = col;
            col += piece.chars().count() + 1; // +1 for the comma we split on

            let name = normalize_name(piece);
            if name.is_empty() {
                errors.push(FormatError {
                    line: line_no,
                    column: piece_start_col,
                    message: "empty name entry (check for a stray comma)".to_string(),
                });
                continue;
            }
            categories.get_mut(&category).unwrap().push(name);
        }
    }

    if errors.is_empty() {
        Ok(NameList { categories })
    } else {
        Err(errors)
    }
}

fn parse_header(trimmed: &str, line_no: usize, start_col: usize) -> Result<String, FormatError> {
    if !trimmed.ends_with(']') {
        return Err(FormatError {
            line: line_no,
            column: start_col + trimmed.chars().count(),
            message: "category header is missing a closing ']'".to_string(),
        });
    }

    let inner = &trimmed[1..trimmed.len() - 1];
    let name = inner.trim();
    if name.is_empty() {
        return Err(FormatError {
            line: line_no,
            column: start_col + 1,
            message: "category header has no name between '[' and ']'".to_string(),
        });
    }
    Ok(name.to_string())
}

/// Collapse internal whitespace and title-case a single name.
///
/// Apostrophes and hyphens start a new word for capitalization purposes
/// (so "o'brien" becomes "O'Brien" and "mary-jane" becomes "Mary-Jane"),
/// but unlike whitespace they are never collapsed or trimmed themselves.
fn normalize_name(raw: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for ch in raw.trim().chars() {
        if ch.is_whitespace() {
            if !result.is_empty() && !result.ends_with(' ') {
                result.push(' ');
            }
            capitalize_next = true;
            continue;
        }
        if ch == '\'' || ch == '-' {
            result.push(ch);
            capitalize_next = true;
            continue;
        }
        if capitalize_next {
            result.extend(ch.to_uppercase());
            capitalize_next = false;
        } else {
            result.extend(ch.to_lowercase());
        }
    }
    result
}

/// Render a [`NameList`] back into canonical source form: categories in
/// alphabetical order, names within each category sorted and deduplicated.
pub fn render(list: &NameList) -> String {
    let mut out = String::new();
    for (category, names) in &list.categories {
        out.push('[');
        out.push_str(category);
        out.push_str("]\n");

        let mut sorted: Vec<&String> = names.iter().collect();
        sorted.sort();
        sorted.dedup();
        for name in sorted {
            out.push_str(name);
            out.push('\n');
        }
        out.push('\n');
    }
    out
}

/// Parse and immediately render, for callers that only want the cleaned
/// text and don't need the intermediate [`NameList`].
pub fn format_source(input: &str) -> Result<String, Vec<FormatError>> {
    parse(input).map(|list| render(&list))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_case_and_whitespace() {
        assert_eq!(normalize_name("  john   SMITH "), "John Smith");
        assert_eq!(normalize_name("mary"), "Mary");
        assert_eq!(normalize_name(""), "");
        assert_eq!(normalize_name("   "), "");
    }

    #[test]
    fn title_cases_apostrophes_and_hyphens() {
        assert_eq!(normalize_name("o'brien"), "O'Brien");
        assert_eq!(normalize_name("D'ANGELO"), "D'Angelo");
        assert_eq!(normalize_name("mary-jane"), "Mary-Jane");
        assert_eq!(normalize_name("jean-luc o'neil"), "Jean-Luc O'Neil");
    }

    #[test]
    fn parses_sections_and_comma_lists() {
        let input = "[dwarves]\nthorin, BALIN\ndwalin\n\n[elves]\n  legolas\n";
        let list = parse(input).expect("should parse");

        assert_eq!(
            list.categories.get("dwarves").unwrap(),
            &vec!["Thorin".to_string(), "Balin".to_string(), "Dwalin".to_string()]
        );
        assert_eq!(list.categories.get("elves").unwrap(), &vec!["Legolas".to_string()]);
    }

    #[test]
    fn reports_name_outside_any_section() {
        let input = "stray\n[dwarves]\nthorin\n";
        let errors = parse(input).unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line, 1);
        assert_eq!(errors[0].column, 1);
    }

    #[test]
    fn reports_unclosed_header_with_column_at_end() {
        let input = "[dwarves\nthorin\n";
        let errors = parse(input).unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line, 1);
        // "[dwarves" is 8 characters, so the caret lands one past the end.
        assert_eq!(errors[0].column, 9);
    }

    #[test]
    fn reports_empty_header_name() {
        let input = "[]\nthorin\n";
        let errors = parse(input).unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line, 1);
        assert_eq!(errors[0].column, 2);
    }

    #[test]
    fn reports_stray_comma_with_column_of_gap() {
        let input = "[dwarves]\nthorin,,dwalin\n";
        let errors = parse(input).unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line, 2);
        assert_eq!(errors[0].column, 8);
    }

    #[test]
    fn render_is_sorted_and_deduplicated() {
        let input = "[dwarves]\nThorin\nBalin\nthorin\n";
        let list = parse(input).unwrap();
        assert_eq!(render(&list), "[dwarves]\nBalin\nThorin\n\n");
    }
}
