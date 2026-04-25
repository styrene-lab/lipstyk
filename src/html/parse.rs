/// Lightweight HTML tag extractor — shared pre-parse for all HTML rules.
///
/// Not a full DOM parser. Extracts a flat list of tags with their
/// attributes, line numbers, and nesting depth. Skips content inside
/// `<script>` and `<style>` blocks to avoid false positives from
/// JS template literals and CSS selectors.

#[derive(Debug)]
pub struct Tag {
    pub name: String,
    pub line: usize,
    pub is_closing: bool,
    pub is_self_closing: bool,
    pub attrs: String,
}

#[derive(Debug)]
pub struct ParsedHtml {
    pub tags: Vec<Tag>,
    pub style_blocks: Vec<String>,
}

pub fn extract_tags(source: &str) -> ParsedHtml {
    let mut tags = Vec::new();
    let mut style_blocks = Vec::new();
    let mut in_script = false;
    let mut in_style = false;
    let mut style_buf = String::new();

    for (i, line) in source.lines().enumerate() {
        let lower = line.to_lowercase();
        let line_num = i + 1;

        if in_script {
            if lower.contains("</script>") {
                in_script = false;
            }
            continue;
        }

        if in_style {
            if lower.contains("</style>") {
                in_style = false;
                style_blocks.push(std::mem::take(&mut style_buf));
            } else {
                style_buf.push_str(line);
                style_buf.push('\n');
            }
            continue;
        }

        // Scan for tags on this line.
        let bytes = line.as_bytes();
        let mut pos = 0;

        while pos < bytes.len() {
            if bytes[pos] != b'<' {
                pos += 1;
                continue;
            }

            let _tag_start = pos;
            pos += 1;
            if pos >= bytes.len() {
                break;
            }

            // Skip comments and doctypes.
            if bytes[pos] == b'!' || bytes[pos] == b'?' {
                pos += 1;
                continue;
            }

            let is_closing = bytes[pos] == b'/';
            if is_closing {
                pos += 1;
            }

            // Extract tag name.
            let name_start = pos;
            while pos < bytes.len() && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'-') {
                pos += 1;
            }

            let name = line[name_start..pos].to_lowercase();
            if name.is_empty() {
                continue;
            }

            // Extract everything up to `>` as attrs.
            let attr_start = pos;
            let mut is_self_closing = false;
            while pos < bytes.len() && bytes[pos] != b'>' {
                pos += 1;
            }
            if pos > 0 && bytes.get(pos.wrapping_sub(1)).copied() == Some(b'/') {
                is_self_closing = true;
            }
            let attrs = line[attr_start..pos.min(bytes.len())].to_string();

            tags.push(Tag {
                name: name.clone(),
                line: line_num,
                is_closing,
                is_self_closing,
                attrs,
            });

            // Track script/style entry.
            if !is_closing && name == "script" {
                in_script = true;
            }
            if !is_closing && name == "style" {
                in_style = true;
            }

            if pos < bytes.len() {
                pos += 1; // skip '>'
            }
        }
    }

    ParsedHtml { tags, style_blocks }
}
