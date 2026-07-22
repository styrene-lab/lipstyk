fn parse_request(body: &str) -> Result<&str, &'static str> {
    let value = body.trim();
    if value.is_empty() {
        return Err("request body is empty");
    }
    Ok(value)
}

fn main() {
    if let Ok(value) = parse_request("ready") {
        println!("{value}");
    }
}
