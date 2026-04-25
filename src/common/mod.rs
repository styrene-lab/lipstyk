pub mod comment_analysis;
pub mod comment_density;
pub mod naming;
pub mod whitespace;

/// Shared comment prefix extraction. Returns the comment body text
/// for the given language's single-line comment syntax.
pub fn extract_comment<'a>(line: &'a str, comment_prefix: &str) -> Option<&'a str> {
    let trimmed = line.trim();
    trimmed.strip_prefix(comment_prefix).map(|text| text.trim())
}
