/// Convertit une entrée arbitraire ("Auth", "auth", "auth-token") en snake_case.
pub fn to_snake_case(input: &str) -> String {
    let mut out = String::new();
    let mut prev_lower_or_digit = false;

    for c in input.chars() {
        if c.is_uppercase() {
            if prev_lower_or_digit {
                out.push('_');
            }
            out.extend(c.to_lowercase());
            prev_lower_or_digit = false;
        } else if c.is_alphanumeric() {
            out.push(c);
            prev_lower_or_digit = true;
        } else {
            // espace, tiret, underscore, etc.
            if !out.ends_with('_') && !out.is_empty() {
                out.push('_');
            }
            prev_lower_or_digit = false;
        }
    }

    out.trim_matches('_').to_string()
}

/// Convertit une entrée arbitraire en PascalCase, ex: "auth-token" -> "AuthToken".
pub fn to_pascal_case(input: &str) -> String {
    to_snake_case(input)
        .split('_')
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_case_basic() {
        assert_eq!(to_snake_case("auth"), "auth");
        assert_eq!(to_snake_case("Auth"), "auth");
        assert_eq!(to_snake_case("client-context"), "client_context");
        assert_eq!(to_snake_case("ClientContext"), "client_context");
    }

    #[test]
    fn pascal_case_basic() {
        assert_eq!(to_pascal_case("auth"), "Auth");
        assert_eq!(to_pascal_case("client-context"), "ClientContext");
    }
}