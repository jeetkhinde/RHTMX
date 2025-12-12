// File: src/directive_parser.rs
// Purpose: Parse RHTMX directives from HTML templates

use crate::value::Value;
use regex::Regex;
use std::collections::HashMap;

/// Parser for RHTMX template directives
pub struct DirectiveParser;

impl DirectiveParser {
    // r-if directive
    pub fn has_if_directive(tag: &str) -> bool {
        tag.contains("r-if=")
    }

    pub fn extract_if_condition(tag: &str) -> Option<String> {
        Self::extract_attr_value(tag, "r-if")
    }

    // r-else-if directive
    pub fn has_else_if_directive(tag: &str) -> bool {
        tag.contains("r-else-if=")
    }

    pub fn extract_else_if_condition(tag: &str) -> Option<String> {
        Self::extract_attr_value(tag, "r-else-if")
    }

    // r-else directive
    pub fn has_else_directive(tag: &str) -> bool {
        tag.contains("r-else")
    }

    // r-for directive
    pub fn has_for_directive(tag: &str) -> bool {
        tag.contains("r-for=")
    }

    pub fn extract_for_loop(tag: &str) -> Option<(String, Option<String>, String)> {
        let value = Self::extract_attr_value(tag, "r-for")?;
        // Pattern: "item in collection" or "(item, index) in collection"
        let re = Regex::new(r"^\(?\s*(\w+)(?:\s*,\s*(\w+))?\s*\)?\s+in\s+(\w+)$").ok()?;
        let caps = re.captures(&value)?;
        let item = caps.get(1)?.as_str().to_string();
        let index = caps.get(2).map(|m| m.as_str().to_string());
        let collection = caps.get(3)?.as_str().to_string();
        Some((item, index, collection))
    }

    // r-match directive
    pub fn has_match_directive(tag: &str) -> bool {
        tag.contains("r-match=")
    }

    pub fn extract_match_variable(tag: &str) -> Option<String> {
        Self::extract_attr_value(tag, "r-match")
    }

    // r-when directive
    pub fn has_when_directive(tag: &str) -> bool {
        tag.contains("r-when=")
    }

    pub fn extract_when_pattern(tag: &str) -> Option<String> {
        Self::extract_attr_value(tag, "r-when")
    }

    // r-default directive
    pub fn has_default_directive(tag: &str) -> bool {
        tag.contains("r-default")
    }

    /// Remove all r-* directives from a tag
    pub fn remove_directives(tag: &str) -> String {
        let re = Regex::new(r#"\s*r-(?:if|else-if|else|for|match|when|default)(?:="[^"]*"|='[^']*')?\s*"#).unwrap();
        re.replace_all(tag, " ").to_string()
    }

    /// Extract attribute value from tag
    fn extract_attr_value(tag: &str, attr: &str) -> Option<String> {
        let pattern = format!(r#"{}="([^"]*)""#, attr);
        let re = Regex::new(&pattern).ok()?;
        re.captures(tag)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }
}

/// Evaluator for template expressions
pub struct ExpressionEvaluator {
    variables: HashMap<String, Value>,
}

impl ExpressionEvaluator {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn from_variables(variables: HashMap<String, Value>) -> Self {
        Self { variables }
    }

    /// Evaluate expression to string
    pub fn eval_string(&self, expr: &str) -> String {
        let expr = expr.trim();

        // Handle property access (e.g., "user.name")
        if expr.contains('.') {
            return self.eval_property_path(expr);
        }

        // Direct variable lookup
        if let Some(value) = self.variables.get(expr) {
            return value.to_string();
        }

        // String literal
        if expr.starts_with('"') && expr.ends_with('"') {
            return expr[1..expr.len()-1].to_string();
        }
        if expr.starts_with('\'') && expr.ends_with('\'') {
            return expr[1..expr.len()-1].to_string();
        }

        // Return as-is if not found
        expr.to_string()
    }

    /// Evaluate expression to boolean
    pub fn eval_bool(&self, expr: &str) -> bool {
        let expr = expr.trim();

        // Handle comparisons
        if let Some((left, op, right)) = self.parse_comparison(expr) {
            let left_val = self.eval_string(&left);
            let right_val = self.eval_string(&right);
            return match op.as_str() {
                "==" => left_val == right_val,
                "!=" => left_val != right_val,
                ">" => left_val.parse::<f64>().unwrap_or(0.0) > right_val.parse::<f64>().unwrap_or(0.0),
                "<" => left_val.parse::<f64>().unwrap_or(0.0) < right_val.parse::<f64>().unwrap_or(0.0),
                ">=" => left_val.parse::<f64>().unwrap_or(0.0) >= right_val.parse::<f64>().unwrap_or(0.0),
                "<=" => left_val.parse::<f64>().unwrap_or(0.0) <= right_val.parse::<f64>().unwrap_or(0.0),
                _ => false,
            };
        }

        // Handle negation
        if expr.starts_with('!') {
            return !self.eval_bool(&expr[1..]);
        }

        // Check for "true"/"false" literals
        if expr == "true" {
            return true;
        }
        if expr == "false" {
            return false;
        }

        // Variable truthiness
        if let Some(value) = self.variables.get(expr) {
            return value.to_bool();
        }

        // Non-empty string is truthy
        !expr.is_empty()
    }

    /// Get array from variable
    pub fn get_array(&self, name: &str) -> Option<Vec<Value>> {
        match self.variables.get(name)? {
            Value::Array(arr) => Some(arr.clone()),
            _ => None,
        }
    }

    /// Evaluate property path (e.g., "user.address.city")
    fn eval_property_path(&self, path: &str) -> String {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return path.to_string();
        }

        let mut current = match self.variables.get(parts[0]) {
            Some(v) => v.clone(),
            None => return path.to_string(),
        };

        for part in &parts[1..] {
            current = match &current {
                Value::Object(obj) => match obj.get(*part) {
                    Some(v) => v.clone(),
                    None => return "".to_string(),
                },
                _ => return "".to_string(),
            };
        }

        current.to_string()
    }

    /// Parse comparison expression
    fn parse_comparison(&self, expr: &str) -> Option<(String, String, String)> {
        for op in ["==", "!=", ">=", "<=", ">", "<"] {
            if let Some(pos) = expr.find(op) {
                let left = expr[..pos].trim().to_string();
                let right = expr[pos + op.len()..].trim().to_string();
                return Some((left, op.to_string(), right));
            }
        }
        None
    }
}

impl Default for ExpressionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directive_detection() {
        assert!(DirectiveParser::has_if_directive(r#"<div r-if="show">"#));
        assert!(DirectiveParser::has_for_directive(r#"<li r-for="item in items">"#));
        assert!(DirectiveParser::has_match_directive(r#"<div r-match="status">"#));
    }

    #[test]
    fn test_for_loop_extraction() {
        let result = DirectiveParser::extract_for_loop(r#"<li r-for="item in items">"#);
        assert!(result.is_some());
        let (item, index, collection) = result.unwrap();
        assert_eq!(item, "item");
        assert!(index.is_none());
        assert_eq!(collection, "items");
    }

    #[test]
    fn test_eval_string() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), Value::String("Alice".to_string()));

        let eval = ExpressionEvaluator::from_variables(vars);
        assert_eq!(eval.eval_string("name"), "Alice");
    }

    #[test]
    fn test_eval_bool() {
        let mut vars = HashMap::new();
        vars.insert("show".to_string(), Value::Bool(true));
        vars.insert("count".to_string(), Value::Number(5.0));

        let eval = ExpressionEvaluator::from_variables(vars);
        assert!(eval.eval_bool("show"));
        assert!(eval.eval_bool("count > 0"));
        assert!(!eval.eval_bool("count < 0"));
    }
}
