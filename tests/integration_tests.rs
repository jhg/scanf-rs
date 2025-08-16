use scanf_proc_macro::sscanf;
use scanf::sscanf_legacy;

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
    // Current implementation: all variables are assigned positionally regardless of names
    let input = "Score: 95, Player: Alice";
    let mut first_var: u32 = 0;  // Gets first placeholder value (95)
    let mut second_var: String = String::new(); // Gets second placeholder value (Alice)
    
    // The variable names in placeholders are ignored - assignment is purely positional
    sscanf_legacy!(input, "Score: {any_name}, Player: {other_name}", &mut first_var, &mut second_var).unwrap();
    assert_eq!(first_var, 95);
    assert_eq!(second_var, "Alice");
}

#[test]
fn test_type_mismatch_error() {
    let input = "Score: 95, Player: Alice";
    let mut wrong_type1: String = String::new(); // Trying to parse "95" as String (this works)
    let mut wrong_type2: u32 = 0; // Trying to parse "Alice" as u32 (this fails)
    
    let result = sscanf_legacy!(input, "Score: {score}, Player: {player}", &mut wrong_type1, &mut wrong_type2);
    assert!(result.is_err()); // Should fail because "Alice" can't be parsed as u32
}