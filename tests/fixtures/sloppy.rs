use std::collections::HashMap;
use std::error::Error;

fn main() {
    // create a new hashmap
    let mut map = HashMap::new();

    // insert values into the map
    map.insert("key1".to_string(), "value1".to_string());
    map.insert("key2".to_string(), "value2".to_string());

    // get the value from the map
    let value = map.get("key1").unwrap();

    // clone the value
    let owned = value.clone();

    // print the result
    println!("{}", owned);

    let data = Some("hello");
    // unwrap the data
    let inner = data.unwrap();

    let items: Vec<String> = vec!["a".to_string(), "b".to_string()];
    let copied = items.clone();
    let also_copied = items.clone();
    let yet_again = copied.clone();

    let result: Result<i32, String> = Ok(42);
    let num = result.unwrap();
    let parsed = "123".parse::<i32>().unwrap();
    let env_var = std::env::var("HOME").unwrap().clone();

    // needless type annotations
    let count: i32 = 0;
    let flag: bool = true;
    let name: String = String::new();
    let buffer: Vec<u8> = Vec::new();

    // verbose match on option
    let maybe = Some(42);
    let val = match maybe {
        Some(v) => v,
        None => 0,
    };

    // verbose match on bool
    let toggle = true;
    let label = match toggle {
        true => "on",
        false => "off",
    };

    // Step 1: Read the config file
    let config = std::fs::read_to_string("config.toml").unwrap();
    // Step 2: Parse the config
    let parsed_config = config.clone();
    // Step 3: Validate the config
    let valid = !parsed_config.is_empty();
    // Step 4: Apply the config
    println!("{}", valid);

    // C-style index loop
    let numbers = vec![1, 2, 3, 4, 5];
    for i in 0..numbers.len() {
        println!("{}", numbers[i]);
    }

    // Error swallowing
    let res: Result<i32, &str> = Err("boom");
    match res {
        Ok(v) => println!("{}", v),
        Err(_) => {}
    }

    // TODO: Add error handling
    // TODO: Implement this
    // TODO: Add logging
}

// generic names with String params
fn process_data(input: String) -> String {
    input.to_uppercase()
}

fn handle_request(req: String) -> String {
    req.to_uppercase()
}

fn do_work() {
    println!("working");
}

fn get_result() -> i32 {
    42
}

fn perform_action(x: i32) -> i32 {
    x + 1
}

// needless lifetime
fn get_first<'a>(s: &'a str) -> &'a str {
    &s[..1]
}

// Box<dyn Error> catch-all
fn load_config(path: &str) -> Result<String, Box<dyn Error>> {
    Ok(std::fs::read_to_string(path)?)
}

fn parse_input(s: &str) -> Result<i32, Box<dyn Error>> {
    Ok(s.parse()?)
}

fn validate(data: &str) -> Result<(), Box<dyn Error>> {
    if data.is_empty() {
        Err("empty".into())
    } else {
        Ok(())
    }
}

// derive stacking
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Config {
    name: String,
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
struct Settings {
    items: Vec<String>,
}

// dead code suppressions
#[allow(dead_code)]
fn unused_helper() -> i32 { 42 }

#[allow(dead_code)]
fn another_unused() -> String { String::new() }

#[allow(unused)]
fn yet_another() {}

// structurally repetitive functions (all same shape: 1 param, 2 stmts, returns)
fn transform_a(input: &str) -> String {
    let upper = input.to_uppercase();
    upper
}

fn transform_b(input: &str) -> String {
    let trimmed = input.trim().to_string();
    trimmed
}

fn transform_c(input: &str) -> String {
    let lower = input.to_lowercase();
    lower
}

fn transform_d(input: &str) -> String {
    let replaced = input.replace(' ', "_");
    replaced
}
