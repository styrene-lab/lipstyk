use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags overuse of bang (`!`) functions in Elixir. The AI reaches for
/// `File.read!` etc. to skip the `{:ok, _} | {:error, _}` handling
/// idiomatic Elixir leans on. Matches the qualified `Module.function!(`
/// pattern, so `!=` and `!expr` negation never trip it.
pub struct BangOveruse;

/// Bangs where raising is the intended contract (config/env access at
/// boot), and not a shortcut, will be excluded from the count.
const ALLOWED_BANGS: &[&str] = &[
    "Application.fetch_env!",
    "Application.compile_env!",
    "System.fetch_env!",
];

const fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// True if `line` has a non-allowlisted qualified bang call.
///
/// Byte-wise scan is panic-safe: Elixir identifiers are ASCII, so every
/// non-ident byte (including bytes of an adjacent multi-byte UTF-8 char)
/// is a char boundary, and the slices below can't split mid-char.
fn has_flaggable_bang(line: &str) -> bool {
    line.match_indices("!(").any(|(at, _)| {
        let head = &line[..at];
        // Function name: the trailing run of identifier bytes.
        let fn_at = head
            .as_bytes()
            .iter()
            .rposition(|&b| !is_ident_byte(b))
            .map_or(0, |p| p + 1);
        if !matches!(head.as_bytes().get(fn_at), Some(b) if b.is_ascii_lowercase() || *b == b'_') {
            return false; // function names are lowercase/underscore-led
        }
        // Module path: identifier/`.` run before the separating `.`.
        let Some(path) = head[..fn_at].strip_suffix('.') else {
            return false;
        };
        let mod_at = path
            .as_bytes()
            .iter()
            .rposition(|&b| !(is_ident_byte(b) || b == b'.'))
            .map_or(0, |p| p + 1);
        if !path
            .as_bytes()
            .get(mod_at)
            .is_some_and(u8::is_ascii_uppercase)
        {
            return false; // module paths are uppercase-led
        }
        let call = &head[mod_at..]; // `Module.function`, no trailing `!`
        !ALLOWED_BANGS
            .iter()
            .any(|a| a.strip_suffix('!') == Some(call))
    })
}

impl SourceRule for BangOveruse {
    fn name(&self) -> &'static str {
        "elixir-bang-overuse"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Elixir]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut hits = Vec::new();

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                continue;
            }
            if has_flaggable_bang(trimmed) {
                hits.push(i + 1);
            }
        }

        if hits.len() < 3 {
            return Vec::new();
        }

        vec![Diagnostic {
            rule: "elixir-bang-overuse",
            message: format!(
                "{} bang (`!`) calls — prefer `{{:ok, _}} | {{:error, _}}` and `with` for control flow",
                hits.len()
            ),
            line: hits[0],
            severity: if hits.len() > 8 {
                Severity::Slop
            } else {
                Severity::Warning
            },
            weight: if hits.len() > 8 { 3.0 } else { 1.5 },
        }]
    }
}
