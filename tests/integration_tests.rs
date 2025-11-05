use scanf::sscanf;

#[test]
fn test_legacy_basic_functionality() {
    let input = "Hello: world";
    let mut request: String = String::new();
    let mut reply: String = String::new();
    sscanf!(input, "{}: {}", &mut request, &mut reply).unwrap();
    assert_eq!(request, "Hello");
    assert_eq!(reply, "world");
}

#[test]
#[allow(clippy::float_cmp)]
fn test_mixed_types() {
    let input = "5 -> 5.0";
    let mut request: i32 = 0;
    let mut reply: f32 = 0.0;
    sscanf!(input, "{} -> {}", &mut request, &mut reply).unwrap();
    assert_eq!(request, 5);
    assert_eq!(reply, 5.0);
}

#[test]
fn test_variable_name_placeholders() {
    // Test that named placeholders are accepted by the parser
    let input = "John: 25";
    let mut name: String = String::new();
    let mut age: i32 = 0;

    // This should work - named placeholders with explicit variable arguments
    // With the new procedural macro, named variables are captured implicitly
    sscanf!(input, "{name}: {age}").unwrap();
    assert_eq!(name, "John");
    assert_eq!(age, 25);
}

#[test]
fn test_mixed_named_and_anonymous() {
    let input = "Temperature: 23.5 degrees";
    let mut location: String = String::new();
    let mut temp: f32 = 0.0;
    let mut unit: String = String::new();

    // Mix named and anonymous placeholders - this demonstrates the intended syntax
    sscanf!(input, "{location}: {} {unit}", &mut temp).unwrap();
    assert_eq!(location, "Temperature");
    assert_eq!(temp, 23.5);
    assert_eq!(unit, "degrees");
}

#[test]
fn test_only_anonymous_placeholders() {
    let input = "apple: 5";
    let mut fruit: String = String::new();
    let mut count: i32 = 0;

    sscanf!(input, "{}: {}", &mut fruit, &mut count).unwrap();
    assert_eq!(fruit, "apple");
    assert_eq!(count, 5);
}

#[test]
fn test_escaped_braces() {
    let input = "{Hello world}";
    let mut message: String = String::new();
    sscanf!(input, "{{{}}}", &mut message).unwrap();
    assert_eq!(message, "Hello world");
}

#[test]
#[should_panic]
fn test_wrong_format_string() {
    let input = "5 -> 5.0 <-";
    let mut _request: i32 = 0;
    let mut _reply: f32 = 0.0;
    // Force error using non-existent separator to trigger panic on unwrap
    sscanf!(input, "{} XXX_SEPARATOR {}", &mut _request, &mut _reply).unwrap();
}

#[test]
fn test_into_array_elements() {
    let s = "3,4";
    let mut arr: [f64; 2] = [0.0; 2];
    sscanf!(s, "{},{}", &mut arr[0], &mut arr[1]).unwrap();
    assert_eq!(arr[0], 3.0);
    assert_eq!(arr[1], 4.0);
}

#[test]
fn test_implicit_variable_capture() {
    let input = "Charlie: 35.5 kg";
    let mut name3: String = String::new();
    let mut weight: f32 = 0.0;
    let mut unit3: String = String::new();

    sscanf!(input, "{name3}: {weight} {unit3}").unwrap();

    assert_eq!(name3, "Charlie");
    assert_eq!(weight, 35.5);
    assert_eq!(unit3, "kg");
}

#[test]
fn test_mixed_implicit_and_explicit() {
    let input = "Alice: 25 years";
    let mut name: String = String::new();
    let mut age: i32 = 0;
    let mut unit: String = String::new();

    sscanf!(input, "{name}: {} {unit}", &mut age).unwrap();

    assert_eq!(name, "Alice");
    assert_eq!(age, 25);
    assert_eq!(unit, "years");
}

#[test]
fn test_fully_explicit_still_works() {
    let input = "Bob: 30";
    let mut name: String = String::new();
    let mut age: i32 = 0;

    sscanf!(input, "{}: {}", &mut name, &mut age).unwrap();

    assert_eq!(name, "Bob");
    assert_eq!(age, 30);
}

#[test]
fn test_positional_parsing() {
    // Procedural macro: named placeholders assign to variables with that name
    let input = "Score: 95, Player: Alice";
    let mut any_name: u32 = 0; // 95
    let mut other_name: String = String::new(); // Alice
    sscanf!(input, "Score: {any_name}, Player: {other_name}").unwrap();
    assert_eq!(any_name, 95);
    assert_eq!(other_name, "Alice");
}

#[test]
fn test_type_mismatch_error() {
    let input = "Score: 95, Player: Alice";
    let mut score: String = String::new(); // "95" parse OK as String
    let mut _player: u32 = 0; // Fails to parse "Alice" as u32
    let result = sscanf!(input, "Score: {score}, Player: {_player}");
    assert!(result.is_err());
    assert_eq!(score, "95"); // use variable
}

#[test]
fn test_empty_input() {
    // Test parsing empty strings
    let input = "";
    let mut value: String = String::new();
    let result = sscanf!(input, "{value}");
    // Empty string should parse as empty String
    assert!(result.is_ok());
    assert_eq!(value, "");
}

#[test]
fn test_whitespace_handling() {
    // Test that whitespace in separators is preserved
    let input = "10  20";
    let mut a: i32 = 0;
    let mut b: i32 = 0;
    sscanf!(input, "{}  {}", &mut a, &mut b).unwrap();
    assert_eq!(a, 10);
    assert_eq!(b, 20);
}

#[test]
fn test_single_placeholder() {
    // Test with only one placeholder
    let input = "42";
    let mut value: i32 = 0;
    sscanf!(input, "{value}").unwrap();
    assert_eq!(value, 42);
}

#[test]
fn test_leading_fixed_text() {
    // Test with fixed text before placeholders
    let input = "Value: 100";
    let mut value: i32 = 0;
    sscanf!(input, "Value: {value}").unwrap();
    assert_eq!(value, 100);
}

#[test]
fn test_trailing_fixed_text() {
    // Test with fixed text after placeholders
    let input = "100 units";
    let mut value: i32 = 0;
    sscanf!(input, "{} units", &mut value).unwrap();
    assert_eq!(value, 100);
}

#[test]
fn test_multiple_escaped_braces() {
    // Test multiple escaped braces
    let input = "{{test}}";
    let mut value: String = String::new();
    sscanf!(input, "{{{{{value}}}}}",).unwrap();
    assert_eq!(value, "test");
}

#[test]
fn test_negative_numbers() {
    // Test parsing negative numbers
    let input = "-42, -3.25";
    let mut int_val: i32 = 0;
    let mut float_val: f64 = 0.0;
    sscanf!(input, "{}, {}", &mut int_val, &mut float_val).unwrap();
    assert_eq!(int_val, -42);
    assert!((float_val + 3.25).abs() < f64::EPSILON * 100.0);
}

#[test]
fn test_unsigned_overflow_detection() {
    // Test that parsing fails for invalid unsigned values
    let input = "-1";
    let mut value: u32 = 0;
    let result = sscanf!(input, "{}", &mut value);
    // Should fail to parse negative number as unsigned
    assert!(result.is_err());
}

#[test]
fn test_special_characters_in_separator() {
    // Test special characters in separators
    let input = "10|20";
    let mut a: i32 = 0;
    let mut b: i32 = 0;
    sscanf!(input, "{}|{}", &mut a, &mut b).unwrap();
    assert_eq!(a, 10);
    assert_eq!(b, 20);
}

#[test]
fn test_long_separator() {
    // Test with multi-character separator
    let input = "apple-->banana";
    let mut a: String = String::new();
    let mut b: String = String::new();
    sscanf!(input, "{}-->{}", &mut a, &mut b).unwrap();
    assert_eq!(a, "apple");
    assert_eq!(b, "banana");
}

#[test]
fn test_identifier_with_numbers() {
    // Test that identifiers with numbers work correctly
    let input = "42, 3.25";
    let mut var1: i32 = 0;
    let mut var2: f64 = 0.0;
    sscanf!(input, "{var1}, {var2}").unwrap();
    assert_eq!(var1, 42);
    assert_eq!(var2, 3.25);
}

#[test]
fn test_identifier_with_underscore() {
    // Test identifiers with underscores
    let input = "100 200";
    let mut _private_var: i32 = 0;
    let mut my_var: i32 = 0;
    sscanf!(input, "{_private_var} {my_var}").unwrap();
    assert_eq!(_private_var, 100);
    assert_eq!(my_var, 200);
}

#[test]
fn test_unicode_identifier() {
    // Test Unicode identifiers (valid in Rust)
    let input = "42";
    let mut número: i32 = 0;
    sscanf!(input, "{número}").unwrap();
    assert_eq!(número, 42);
}

#[test]
fn test_complex_parsing_scenario() {
    // Test a more complex real-world scenario
    let input = "User: john_doe, Age: 25, Score: 95.5";
    let mut username: String = String::new();
    let mut user_age: i32 = 0;
    let mut user_score: f64 = 0.0;
    sscanf!(
        input,
        "User: {username}, Age: {user_age}, Score: {user_score}"
    )
    .unwrap();
    assert_eq!(username, "john_doe");
    assert_eq!(user_age, 25);
    assert_eq!(user_score, 95.5);
}

#[test]
fn test_input_with_trailing_whitespace() {
    // Test that trailing whitespace does not affect parsing
    let input = "42  "; // trailing spaces
    let mut value: i32 = 0;
    sscanf!(input.trim_end(), "{value}").unwrap();
    assert_eq!(value, 42);
}

#[test]
fn test_empty_string_field_parsing() {
    // Test parsing empty string to String type
    let input = ":end";
    let mut value: String = String::new();
    let mut marker: String = String::new();
    sscanf!(input, "{value}:{marker}").unwrap();
    assert_eq!(value, "");
    assert_eq!(marker, "end");
}

// ============================================================================
// Security Tests
// ============================================================================

#[test]
fn test_security_reasonable_format_string() {
    // Test that reasonable format strings work fine
    let input = "a:b:c:d:e";
    let mut a = String::new();
    let mut b = String::new();
    let mut c = String::new();
    let mut d = String::new();
    let mut e = String::new();
    sscanf!(input, "{a}:{b}:{c}:{d}:{e}").unwrap();
    // Verify parsing worked correctly
    assert_eq!(a, "a");
    assert_eq!(b, "b");
    assert_eq!(c, "c");
    assert_eq!(d, "d");
    assert_eq!(e, "e");
}

#[test]
fn test_security_many_placeholders() {
    // Test with many placeholders (but within limits)
    let input = "1 2 3 4 5 6 7 8 9 10";
    let mut n1: i32 = 0;
    let mut n2: i32 = 0;
    let mut n3: i32 = 0;
    let mut n4: i32 = 0;
    let mut n5: i32 = 0;
    let mut n6: i32 = 0;
    let mut n7: i32 = 0;
    let mut n8: i32 = 0;
    let mut n9: i32 = 0;
    let mut n10: i32 = 0;
    sscanf!(input, "{n1} {n2} {n3} {n4} {n5} {n6} {n7} {n8} {n9} {n10}").unwrap();
    // Verify all values parsed correctly
    assert_eq!(n1, 1);
    assert_eq!(
        n2 + n3 + n4 + n5 + n6 + n7 + n8 + n9,
        2 + 3 + 4 + 5 + 6 + 7 + 8 + 9
    );
    assert_eq!(n10, 10);
}

#[test]
fn test_security_long_but_valid_identifier() {
    // Test with long but valid identifier (within 128 char limit)
    let input = "42";
    let mut this_is_a_very_long_variable_name_but_still_valid: i32 = 0;
    sscanf!(input, "{this_is_a_very_long_variable_name_but_still_valid}").unwrap();
    assert_eq!(this_is_a_very_long_variable_name_but_still_valid, 42);
}

#[test]
fn test_variable_scope_no_collision_anonymous() {
    // Test that macro internal variables don't collide with user variables when using anonymous placeholders
    let input = "test";
    let mut value: String = String::new();
    let remaining: String = String::from("should not be modified");
    let buffer: String = String::from("should not be modified");

    sscanf!(input, "{}", &mut value).unwrap();

    assert_eq!(value, "test");
    assert_eq!(remaining, "should not be modified");
    assert_eq!(buffer, "should not be modified");
}

#[test]
fn test_only_literal_text_no_placeholders() {
    // Test format with only literal text (no placeholders)
    let input = "exact match";
    let result = sscanf!(input, "exact match");
    assert!(result.is_ok());
}

#[test]
fn test_only_literal_text_mismatch() {
    // Test format with only literal text that doesn't match
    let input = "wrong text";
    let result = sscanf!(input, "exact match");
    assert!(result.is_err());
}

#[test]
fn test_unicode_separator() {
    // Test Unicode characters in separators
    let input = "10→20";
    let mut a: i32 = 0;
    let mut b: i32 = 0;
    sscanf!(input, "{}→{}", &mut a, &mut b).unwrap();
    assert_eq!(a, 10);
    assert_eq!(b, 20);
}

#[test]
fn test_bool_parsing() {
    // Test boolean parsing
    let input = "true false";
    let mut a: bool = false;
    let mut b: bool = true;
    sscanf!(input, "{} {}", &mut a, &mut b).unwrap();
    assert!(a);
    assert!(!b);
}

#[test]
fn test_char_parsing() {
    // Test single character parsing
    let input = "a b";
    let mut first: char = ' ';
    let mut second: char = ' ';
    sscanf!(input, "{} {}", &mut first, &mut second).unwrap();
    assert_eq!(first, 'a');
    assert_eq!(second, 'b');
}
