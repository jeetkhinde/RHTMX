// File: rhtmx-macro/src/validation.rs
// Purpose: Validation derive macro and attribute processing

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Expr, ExprLit, Fields, Lit};

/// Parse validation attributes from a field
pub fn extract_validation_attrs(attrs: &[syn::Attribute]) -> Vec<ValidationAttr> {
    let mut validations = Vec::new();

    for attr in attrs {
        let path = &attr.path();
        let attr_name = path.segments.last().map(|s| s.ident.to_string());

        match attr_name.as_deref() {
            Some("email") => {
                validations.push(ValidationAttr::Email);
            }
            Some("no_public_domains") => {
                validations.push(ValidationAttr::NoPublicDomains);
            }
            Some("blocked_domains") => {
                // Parse blocked_domains("domain1", "domain2")
                let mut domains = Vec::new();
                let _ = attr.parse_nested_meta(|meta| {
                    if let Ok(value) = meta.value() {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        })) = value.parse::<Expr>()
                        {
                            domains.push(s.value());
                        }
                    }
                    Ok(())
                });
                if !domains.is_empty() {
                    validations.push(ValidationAttr::BlockedDomains(domains));
                }
            }
            Some("password") => {
                // Parse password("strong") or password(r"regex")
                let _ = attr.parse_nested_meta(|meta| {
                    if let Ok(value) = meta.value() {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        })) = value.parse::<Expr>()
                        {
                            validations.push(ValidationAttr::Password(s.value()));
                        }
                    }
                    Ok(())
                });
            }
            Some("min") => {
                let _ = attr.parse_nested_meta(|meta| {
                    if let Ok(value) = meta.value() {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Int(i), ..
                        })) = value.parse::<Expr>()
                        {
                            if let Ok(val) = i.base10_parse::<i64>() {
                                validations.push(ValidationAttr::Min(val));
                            }
                        }
                    }
                    Ok(())
                });
            }
            Some("max") => {
                let _ = attr.parse_nested_meta(|meta| {
                    if let Ok(value) = meta.value() {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Int(i), ..
                        })) = value.parse::<Expr>()
                        {
                            if let Ok(val) = i.base10_parse::<i64>() {
                                validations.push(ValidationAttr::Max(val));
                            }
                        }
                    }
                    Ok(())
                });
            }
            Some("range") => {
                // Parse range(min, max)
                let mut nums = Vec::new();
                let _ = attr.parse_nested_meta(|meta| {
                    if let Ok(value) = meta.value() {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Int(i), ..
                        })) = value.parse::<Expr>()
                        {
                            if let Ok(val) = i.base10_parse::<i64>() {
                                nums.push(val);
                            }
                        }
                    }
                    Ok(())
                });
                if nums.len() >= 2 {
                    validations.push(ValidationAttr::Range(nums[0], nums[1]));
                }
            }
            Some("min_length") => {
                let _ = attr.parse_nested_meta(|meta| {
                    if let Ok(value) = meta.value() {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Int(i), ..
                        })) = value.parse::<Expr>()
                        {
                            if let Ok(val) = i.base10_parse::<usize>() {
                                validations.push(ValidationAttr::MinLength(val));
                            }
                        }
                    }
                    Ok(())
                });
            }
            Some("max_length") => {
                let _ = attr.parse_nested_meta(|meta| {
                    if let Ok(value) = meta.value() {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Int(i), ..
                        })) = value.parse::<Expr>()
                        {
                            if let Ok(val) = i.base10_parse::<usize>() {
                                validations.push(ValidationAttr::MaxLength(val));
                            }
                        }
                    }
                    Ok(())
                });
            }
            Some("length") => {
                let mut nums = Vec::new();
                let _ = attr.parse_nested_meta(|meta| {
                    if let Ok(value) = meta.value() {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Int(i), ..
                        })) = value.parse::<Expr>()
                        {
                            if let Ok(val) = i.base10_parse::<usize>() {
                                nums.push(val);
                            }
                        }
                    }
                    Ok(())
                });
                if nums.len() >= 2 {
                    validations.push(ValidationAttr::Length(nums[0], nums[1]));
                }
            }
            Some("regex") => {
                let _ = attr.parse_nested_meta(|meta| {
                    if let Ok(value) = meta.value() {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        })) = value.parse::<Expr>()
                        {
                            validations.push(ValidationAttr::Regex(s.value()));
                        }
                    }
                    Ok(())
                });
            }
            Some("url") => {
                validations.push(ValidationAttr::Url);
            }
            Some("allow_whitespace") => {
                validations.push(ValidationAttr::AllowWhitespace);
            }
            Some("required") => {
                validations.push(ValidationAttr::Required);
            }
            Some("contains") => {
                let _ = attr.parse_nested_meta(|meta| {
                    if let Ok(value) = meta.value() {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        })) = value.parse::<Expr>()
                        {
                            validations.push(ValidationAttr::Contains(s.value()));
                        }
                    }
                    Ok(())
                });
            }
            Some("not_contains") => {
                let _ = attr.parse_nested_meta(|meta| {
                    if let Ok(value) = meta.value() {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        })) = value.parse::<Expr>()
                        {
                            validations.push(ValidationAttr::NotContains(s.value()));
                        }
                    }
                    Ok(())
                });
            }
            Some("starts_with") => {
                let _ = attr.parse_nested_meta(|meta| {
                    if let Ok(value) = meta.value() {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        })) = value.parse::<Expr>()
                        {
                            validations.push(ValidationAttr::StartsWith(s.value()));
                        }
                    }
                    Ok(())
                });
            }
            Some("ends_with") => {
                let _ = attr.parse_nested_meta(|meta| {
                    if let Ok(value) = meta.value() {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        })) = value.parse::<Expr>()
                        {
                            validations.push(ValidationAttr::EndsWith(s.value()));
                        }
                    }
                    Ok(())
                });
            }
            Some("equals") => {
                let _ = attr.parse_nested_meta(|meta| {
                    if let Ok(value) = meta.value() {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        })) = value.parse::<Expr>()
                        {
                            validations.push(ValidationAttr::Equals(s.value()));
                        }
                    }
                    Ok(())
                });
            }
            Some("not_equals") => {
                let _ = attr.parse_nested_meta(|meta| {
                    if let Ok(value) = meta.value() {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        })) = value.parse::<Expr>()
                        {
                            validations.push(ValidationAttr::NotEquals(s.value()));
                        }
                    }
                    Ok(())
                });
            }
            Some("equals_field") => {
                let _ = attr.parse_nested_meta(|meta| {
                    if let Ok(value) = meta.value() {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        })) = value.parse::<Expr>()
                        {
                            validations.push(ValidationAttr::EqualsField(s.value()));
                        }
                    }
                    Ok(())
                });
            }
            Some("depends_on") => {
                // Parse depends_on("field", "value")
                let mut params = Vec::new();
                let _ = attr.parse_nested_meta(|meta| {
                    if let Ok(value) = meta.value() {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        })) = value.parse::<Expr>()
                        {
                            params.push(s.value());
                        }
                    }
                    Ok(())
                });
                if params.len() >= 2 {
                    validations.push(ValidationAttr::DependsOn(params[0].clone(), params[1].clone()));
                }
            }
            Some("min_items") => {
                let _ = attr.parse_nested_meta(|meta| {
                    if let Ok(value) = meta.value() {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Int(i), ..
                        })) = value.parse::<Expr>()
                        {
                            if let Ok(val) = i.base10_parse::<usize>() {
                                validations.push(ValidationAttr::MinItems(val));
                            }
                        }
                    }
                    Ok(())
                });
            }
            Some("max_items") => {
                let _ = attr.parse_nested_meta(|meta| {
                    if let Ok(value) = meta.value() {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Int(i), ..
                        })) = value.parse::<Expr>()
                        {
                            if let Ok(val) = i.base10_parse::<usize>() {
                                validations.push(ValidationAttr::MaxItems(val));
                            }
                        }
                    }
                    Ok(())
                });
            }
            Some("unique") => {
                validations.push(ValidationAttr::Unique);
            }
            Some("enum_variant") => {
                // Parse enum_variant("value1", "value2", ...)
                let mut variants = Vec::new();
                let _ = attr.parse_nested_meta(|meta| {
                    if let Ok(value) = meta.value() {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        })) = value.parse::<Expr>()
                        {
                            variants.push(s.value());
                        }
                    }
                    Ok(())
                });
                if !variants.is_empty() {
                    validations.push(ValidationAttr::EnumVariant(variants));
                }
            }
            Some("message") => {
                let _ = attr.parse_nested_meta(|meta| {
                    if let Ok(value) = meta.value() {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        })) = value.parse::<Expr>()
                        {
                            validations.push(ValidationAttr::Message(s.value()));
                        }
                    }
                    Ok(())
                });
            }
            Some("label") => {
                let _ = attr.parse_nested_meta(|meta| {
                    if let Ok(value) = meta.value() {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        })) = value.parse::<Expr>()
                        {
                            validations.push(ValidationAttr::Label(s.value()));
                        }
                    }
                    Ok(())
                });
            }
            Some("message_key") => {
                let _ = attr.parse_nested_meta(|meta| {
                    if let Ok(value) = meta.value() {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        })) = value.parse::<Expr>()
                        {
                            validations.push(ValidationAttr::MessageKey(s.value()));
                        }
                    }
                    Ok(())
                });
            }
            Some("custom") => {
                let _ = attr.parse_nested_meta(|meta| {
                    if let Ok(value) = meta.value() {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        })) = value.parse::<Expr>()
                        {
                            validations.push(ValidationAttr::Custom(s.value()));
                        }
                    }
                    Ok(())
                });
            }
            Some("query") => {
                validations.push(ValidationAttr::Query);
            }
            Some("form") => {
                validations.push(ValidationAttr::Form);
            }
            Some("path") => {
                validations.push(ValidationAttr::Path);
            }
            _ => {}
        }
    }

    validations
}

#[derive(Debug, Clone)]
pub enum ValidationAttr {
    // Email validators
    Email,
    NoPublicDomains,
    BlockedDomains(Vec<String>),

    // Password validators
    Password(String), // Pattern name or regex

    // Numeric validators
    Min(i64),
    Max(i64),
    Range(i64, i64),

    // String validators
    MinLength(usize),
    MaxLength(usize),
    Length(usize, usize),
    Regex(String),
    Url,
    AllowWhitespace,

    // String matching validators
    Contains(String),
    NotContains(String),
    StartsWith(String),
    EndsWith(String),

    // Equality validators
    Equals(String),
    NotEquals(String),
    EqualsField(String),

    // Conditional validators
    DependsOn(String, String), // (field_name, required_value)

    // Collection validators
    MinItems(usize),
    MaxItems(usize),
    Unique,

    // Enum/value restriction
    EnumVariant(Vec<String>),

    // Custom messages & labels
    Message(String),
    Label(String),
    MessageKey(String),

    // Custom validation
    Custom(String), // Function name

    // General
    Required,
    Query,
    Form,
    Path,
}

/// Generate validation implementation for a struct
pub fn impl_validate(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Validate only supports structs with named fields"),
        },
        _ => panic!("Validate only supports structs"),
    };

    let mut validation_code = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let validations = extract_validation_attrs(&field.attrs);

        if validations.is_empty() {
            continue;
        }

        // Check if field is Option<T>
        let is_option = is_option_type(&field.ty);
        let has_allow_whitespace = validations
            .iter()
            .any(|v| matches!(v, ValidationAttr::AllowWhitespace));

        // Extract custom message, label, and message_key if present
        let custom_message = validations
            .iter()
            .find_map(|v| match v {
                ValidationAttr::Message(msg) => Some(msg.clone()),
                _ => None,
            });

        let field_label = validations
            .iter()
            .find_map(|v| match v {
                ValidationAttr::Label(label) => Some(label.clone()),
                _ => None,
            })
            .unwrap_or_else(|| field_name_str.clone());

        for validation in &validations {
            let validation_check = match validation {
                ValidationAttr::Email => {
                    quote! {
                        if !rhtmx::validation::validators::is_valid_email(&self.#field_name) {
                            errors.insert(#field_name_str.to_string(), "Invalid email address".to_string());
                        }
                    }
                }
                ValidationAttr::NoPublicDomains => {
                    quote! {
                        if rhtmx::validation::validators::is_public_domain(&self.#field_name) {
                            errors.insert(#field_name_str.to_string(), "Public email domains not allowed".to_string());
                        }
                    }
                }
                ValidationAttr::BlockedDomains(domains) => {
                    let domains_vec = domains
                        .iter()
                        .map(|d| quote! { #d.to_string() })
                        .collect::<Vec<_>>();
                    quote! {
                        if rhtmx::validation::validators::is_blocked_domain(&self.#field_name, &vec![#(#domains_vec),*]) {
                            errors.insert(#field_name_str.to_string(), "Email domain is blocked".to_string());
                        }
                    }
                }
                ValidationAttr::Password(pattern) => {
                    quote! {
                        if let Err(msg) = rhtmx::validation::validators::validate_password(&self.#field_name, #pattern) {
                            errors.insert(#field_name_str.to_string(), msg);
                        }
                    }
                }
                ValidationAttr::Min(min_val) => {
                    quote! {
                        if self.#field_name < #min_val {
                            errors.insert(#field_name_str.to_string(), format!("Must be at least {}", #min_val));
                        }
                    }
                }
                ValidationAttr::Max(max_val) => {
                    quote! {
                        if self.#field_name > #max_val {
                            errors.insert(#field_name_str.to_string(), format!("Must be at most {}", #max_val));
                        }
                    }
                }
                ValidationAttr::Range(min_val, max_val) => {
                    quote! {
                        if self.#field_name < #min_val || self.#field_name > #max_val {
                            errors.insert(#field_name_str.to_string(), format!("Must be between {} and {}", #min_val, #max_val));
                        }
                    }
                }
                ValidationAttr::MinLength(min_len) => {
                    quote! {
                        if self.#field_name.len() < #min_len {
                            errors.insert(#field_name_str.to_string(), format!("Must be at least {} characters", #min_len));
                        }
                    }
                }
                ValidationAttr::MaxLength(max_len) => {
                    quote! {
                        if self.#field_name.len() > #max_len {
                            errors.insert(#field_name_str.to_string(), format!("Must be at most {} characters", #max_len));
                        }
                    }
                }
                ValidationAttr::Length(min_len, max_len) => {
                    quote! {
                        let len = self.#field_name.len();
                        if len < #min_len || len > #max_len {
                            errors.insert(#field_name_str.to_string(), format!("Must be between {} and {} characters", #min_len, #max_len));
                        }
                    }
                }
                ValidationAttr::Regex(pattern) => {
                    quote! {
                        if !rhtmx::validation::validators::matches_regex(&self.#field_name, #pattern) {
                            errors.insert(#field_name_str.to_string(), "Invalid format".to_string());
                        }
                    }
                }
                ValidationAttr::Url => {
                    quote! {
                        if !rhtmx::validation::validators::is_valid_url(&self.#field_name) {
                            errors.insert(#field_name_str.to_string(), "Invalid URL".to_string());
                        }
                    }
                }
                ValidationAttr::Contains(substring) => {
                    quote! {
                        if !self.#field_name.contains(#substring) {
                            errors.insert(#field_name_str.to_string(), format!("Must contain '{}'", #substring));
                        }
                    }
                }
                ValidationAttr::NotContains(substring) => {
                    quote! {
                        if self.#field_name.contains(#substring) {
                            errors.insert(#field_name_str.to_string(), format!("Must not contain '{}'", #substring));
                        }
                    }
                }
                ValidationAttr::StartsWith(prefix) => {
                    quote! {
                        if !self.#field_name.starts_with(#prefix) {
                            errors.insert(#field_name_str.to_string(), format!("Must start with '{}'", #prefix));
                        }
                    }
                }
                ValidationAttr::EndsWith(suffix) => {
                    quote! {
                        if !self.#field_name.ends_with(#suffix) {
                            errors.insert(#field_name_str.to_string(), format!("Must end with '{}'", #suffix));
                        }
                    }
                }
                ValidationAttr::Equals(value) => {
                    quote! {
                        if self.#field_name != #value {
                            errors.insert(#field_name_str.to_string(), format!("Must equal '{}'", #value));
                        }
                    }
                }
                ValidationAttr::NotEquals(value) => {
                    quote! {
                        if self.#field_name == #value {
                            errors.insert(#field_name_str.to_string(), format!("Must not equal '{}'", #value));
                        }
                    }
                }
                ValidationAttr::EqualsField(other_field) => {
                    let other_field_ident = syn::Ident::new(&other_field, proc_macro2::Span::call_site());
                    quote! {
                        if self.#field_name != self.#other_field_ident {
                            errors.insert(#field_name_str.to_string(), format!("Must match {}", #other_field));
                        }
                    }
                }
                ValidationAttr::DependsOn(dep_field, dep_value) => {
                    let dep_field_ident = syn::Ident::new(&dep_field, proc_macro2::Span::call_site());
                    quote! {
                        if self.#dep_field_ident == #dep_value {
                            if let Some(ref val) = self.#field_name {
                                if val.is_empty() {
                                    errors.insert(#field_name_str.to_string(), format!("Required when {} is {}", #dep_field, #dep_value));
                                }
                            } else {
                                errors.insert(#field_name_str.to_string(), format!("Required when {} is {}", #dep_field, #dep_value));
                            }
                        }
                    }
                }
                ValidationAttr::MinItems(min_count) => {
                    quote! {
                        if self.#field_name.len() < #min_count {
                            errors.insert(#field_name_str.to_string(), format!("Must have at least {} items", #min_count));
                        }
                    }
                }
                ValidationAttr::MaxItems(max_count) => {
                    quote! {
                        if self.#field_name.len() > #max_count {
                            errors.insert(#field_name_str.to_string(), format!("Must have at most {} items", #max_count));
                        }
                    }
                }
                ValidationAttr::Unique => {
                    quote! {
                        {
                            let mut seen = std::collections::HashSet::new();
                            for item in &self.#field_name {
                                if !seen.insert(item) {
                                    errors.insert(#field_name_str.to_string(), "All items must be unique".to_string());
                                    break;
                                }
                            }
                        }
                    }
                }
                ValidationAttr::EnumVariant(allowed_values) => {
                    let values_vec = allowed_values
                        .iter()
                        .map(|v| quote! { #v })
                        .collect::<Vec<_>>();
                    quote! {
                        {
                            let allowed = vec![#(#values_vec),*];
                            if !allowed.contains(&self.#field_name.as_str()) {
                                errors.insert(#field_name_str.to_string(), format!("Must be one of: {}", allowed.join(", ")));
                            }
                        }
                    }
                }
                ValidationAttr::Custom(func_name) => {
                    let func_ident = syn::Ident::new(&func_name, proc_macro2::Span::call_site());
                    quote! {
                        if let Err(msg) = #func_ident(&self.#field_name) {
                            errors.insert(#field_name_str.to_string(), msg);
                        }
                    }
                }
                ValidationAttr::Required => {
                    if is_option {
                        let error_msg = custom_message
                            .clone()
                            .unwrap_or_else(|| format!("{} is required", field_label));
                        quote! {
                            if self.#field_name.is_none() {
                                errors.insert(#field_name_str.to_string(), #error_msg.to_string());
                            }
                        }
                    } else {
                        continue;
                    }
                }
                ValidationAttr::Message(_)
                | ValidationAttr::Label(_)
                | ValidationAttr::MessageKey(_)
                | ValidationAttr::AllowWhitespace
                | ValidationAttr::Query
                | ValidationAttr::Form
                | ValidationAttr::Path => continue,
            };

            validation_code.push(validation_check);
        }

        // Add default whitespace handling for String fields (not Option)
        if !is_option && !has_allow_whitespace {
            // Check if the field is a String type
            if is_string_type(&field.ty) {
                let error_msg = custom_message
                    .unwrap_or_else(|| format!("{} is required", field_label));
                validation_code.push(quote! {
                    if self.#field_name.trim().is_empty() {
                        errors.insert(#field_name_str.to_string(), #error_msg.to_string());
                    }
                });
            }
        }
    }

    quote! {
        impl rhtmx::validation::Validate for #name {
            fn validate(&self) -> Result<(), std::collections::HashMap<String, String>> {
                let mut errors = std::collections::HashMap::new();

                #(#validation_code)*

                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(errors)
                }
            }
        }
    }
}

/// Check if a type is Option<T>
fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

/// Check if a type is String
fn is_string_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "String";
        }
    }
    false
}
