use scanf::sscanf_legacy;

#[test]
fn demonstrate_basic_functionality() {
    // Example 1: Generic placeholders (traditional approach)
    println!("=== Generic placeholders ===");
    let input1 = "Alice: 30 years old";
    let mut name1: String = String::new();
    let mut age1: u32 = 0;
    let mut unit1: String = String::new();

    sscanf_legacy!(input1, "{}: {} {} old", &mut name1, &mut age1, &mut unit1).unwrap();
    println!("Parsed: name={}, age={}, unit={}", name1, age1, unit1);
    assert_eq!(name1, "Alice");
    assert_eq!(age1, 30);
    assert_eq!(unit1, "years");

    // Example 2: Variable names for clarity (syntax accepted but works positionally)
    println!("\n=== Variable names for clarity ===");
    let input2 = "Bob: 25 years old";
    let mut name2: String = String::new();
    let mut age2: u32 = 0;
    let mut unit2: String = String::new();

    sscanf_legacy!(input2, "{name2}: {age2} {unit2} old", &mut name2, &mut age2, &mut unit2).unwrap();
    println!("Parsed: name={}, age={}, unit={}", name2, age2, unit2);
    assert_eq!(name2, "Bob");
    assert_eq!(age2, 25);
    assert_eq!(unit2, "years");

    // Example 3: Mixed syntax with variable names and generic placeholders
    println!("\n=== Mixed syntax ===");
    let input3 = "Charlie: 35.5 kg";
    let mut name3: String = String::new();
    let mut weight: f32 = 0.0;
    let mut unit3: String = String::new();

    sscanf_legacy!(input3, "{name3}: {} {unit3}", &mut name3, &mut weight, &mut unit3).unwrap();
    println!("Parsed: name={}, weight={}, unit={}", name3, weight, unit3);
    assert_eq!(name3, "Charlie");
    assert_eq!(weight, 35.5);
    assert_eq!(unit3, "kg");

    // Example 4: Pure generic placeholders
    println!("\n=== Pure generic placeholders ===");
    let input4 = "Diana: 28";
    let mut name4: String = String::new();
    let mut age4: u32 = 0;

    sscanf_legacy!(input4, "{}: {}", &mut name4, &mut age4).unwrap();
    println!("Parsed: name={}, age={}", name4, age4);
    assert_eq!(name4, "Diana");
    assert_eq!(age4, 28);
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