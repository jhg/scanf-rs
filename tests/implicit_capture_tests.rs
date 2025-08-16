use scanf::sscanf;

#[test]
fn test_implicit_variable_capture() {
    let input = "Charlie: 35.5 kg";
    let mut name3: String = String::new();
    let mut weight: f32 = 0.0;
    let mut unit3: String = String::new();

    // Test the new syntax - no arguments needed!
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

    // Test mixed syntax - name and unit captured, age explicit
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

    // Old syntax should still work
    sscanf!(input, "{}: {}", &mut name, &mut age).unwrap();
    
    assert_eq!(name, "Bob");
    assert_eq!(age, 30);
}

#[test]
fn test_user_examples() {
    // Test exactly the examples the user provided
    
    // Example 1: Mixed named and anonymous (weight passed explicitly)
    let input3 = "Charlie: 35.5 kg";
    let mut name3: String = String::new();
    let mut weight: f32 = 0.0;
    let mut unit3: String = String::new();
    
    sscanf!(input3, "{name3}: {} {unit3}", &mut weight).unwrap();
    assert_eq!(name3, "Charlie");
    assert_eq!(weight, 35.5);
    assert_eq!(unit3, "kg");
    
    // Example 2: Full implicit capture (no arguments needed)
    let input3_b = "Diana: 42.0 pounds";
    let mut name3_b: String = String::new();
    let mut weight_b: f32 = 0.0;
    let mut unit3_b: String = String::new();
    
    sscanf!(input3_b, "{name3_b}: {weight_b} {unit3_b}").unwrap();
    assert_eq!(name3_b, "Diana");
    assert_eq!(weight_b, 42.0);
    assert_eq!(unit3_b, "pounds");
}