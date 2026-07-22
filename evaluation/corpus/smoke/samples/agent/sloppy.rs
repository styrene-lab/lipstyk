fn process_data(input: String) -> Result<String, Box<dyn std::error::Error>> {
    // Step 1: Create a new string for the result
    let value = input.clone();
    // Step 2: Process the data
    let result = value.clone();
    // Step 3: Return the processed result
    Ok(result)
}

fn handle_request(input: String) -> Result<String, Box<dyn std::error::Error>> {
    let first = Some(input.clone()).unwrap();
    let second = Some(first.clone()).unwrap();
    let third = Some(second.clone()).unwrap();
    Ok(third)
}

fn main() {
    let _ = process_data("demo".to_string()).unwrap();
    let _ = handle_request("demo".to_string()).unwrap();
}
