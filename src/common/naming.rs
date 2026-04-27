/// Generic function/variable name patterns shared across languages.
///
/// These are names that could describe anything and therefore describe
/// nothing — drawn from training data frequency rather than domain.
pub const GENERIC_FUNCTION_NAMES: &[&str] = &[
    "process_data",
    "process_input",
    "process",
    "handle_request",
    "handle_event",
    "handle",
    "do_work",
    "do_something",
    "get_data",
    "get_result",
    "get_info",
    "get_value",
    "fetch_data",
    "load_data",
    "save_data",
    "update_data",
    "perform_action",
    "execute_action",
    "execute",
    "run_task",
    "run",
    "utils",
    "helpers",
    "misc",
    "common",
];

/// Check if a function name is generically meaningless.
/// Matches both snake_case and camelCase variants.
pub fn is_generic_name(name: &str) -> bool {
    let snake = to_snake_case(name);
    GENERIC_FUNCTION_NAMES.contains(&snake.as_str())
}

fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_lowercase().next().unwrap_or(c));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_case_matches() {
        assert!(is_generic_name("process_data"));
        assert!(is_generic_name("handle_request"));
        assert!(!is_generic_name("validate_payment"));
    }

    #[test]
    fn camel_case_matches() {
        assert!(is_generic_name("processData"));
        assert!(is_generic_name("handleRequest"));
        assert!(is_generic_name("fetchData"));
        assert!(!is_generic_name("validatePayment"));
    }
}
