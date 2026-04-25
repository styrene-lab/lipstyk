fn production_code() -> String {
    let val = "hello".parse::<i32>().unwrap();
    let data = Some("test").unwrap();
    format!("{val}{data}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_production_code() {
        let result = production_code();
        assert!(!result.is_empty());
        let parsed = "42".parse::<i32>().unwrap();
        assert_eq!(parsed, 42);
        let val = Some(1).unwrap();
        let other = Ok::<_, &str>("yes").unwrap();
        assert_eq!(val, 1);
    }

    #[test]
    fn test_another() {
        let x = std::env::var("PATH").unwrap();
        assert!(!x.is_empty());
    }
}
