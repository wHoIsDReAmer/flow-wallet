pub fn format_units(value: &str, decimals: u32) -> String {
    let decimals = decimals as usize;
    if value.is_empty() {
        return "0".to_string();
    }

    // Check if value contains non-digits
    if value.chars().any(|c| !c.is_ascii_digit()) {
        return value.to_string(); // Return as-is if invalid
    }

    if value.len() <= decimals {
        let mut padded = String::with_capacity(decimals + 2);
        padded.push_str("0.");
        for _ in 0..(decimals - value.len()) {
            padded.push('0');
        }
        padded.push_str(value);
        return padded;
    }

    let split_idx = value.len() - decimals;
    let (integer, fractional) = value.split_at(split_idx);
    format!("{}.{}", integer, fractional)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_units() {
        assert_eq!(format_units("50059810", 6), "50.059810");
        assert_eq!(format_units("1000000", 6), "1.000000");
        assert_eq!(format_units("1", 6), "0.000001");
        assert_eq!(format_units("123", 6), "0.000123");
        assert_eq!(format_units("0", 6), "0.000000");

        // LTC case (8 decimals)
        assert_eq!(format_units("100000000", 8), "1.00000000");
    }
}
