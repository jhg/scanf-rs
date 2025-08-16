use scanf::sscanf;

#[test]
fn demonstrate_new_syntax() {
    // Example 1: Generic placeholders (no specific types)
    println!("=== Generic placeholders ===");
    let input1 = "Alice: 30 years old";
    let mut name1: String = String::new();
    let mut age1: u32 = 0;
    let mut unit1: String = String::new();

    sscanf!(input1, "{}: {} {} old", name1, age1, unit1).unwrap();
    println!("Parsed: name={}, age={}, unit={}", name1, age1, unit1);
    assert_eq!(name1, "Alice");
    assert_eq!(age1, 30);
    assert_eq!(unit1, "years");

    // Example 2: Variable names for clarity
    println!("\n=== Variable names for clarity ===");
    let input2 = "Bob: 25 years old";
    let mut name2: String = String::new();
    let mut age2: u32 = 0;
    let mut unit2: String = String::new();

    sscanf!(input2, "{name2}: {age2} {unit2} old", name2, age2, unit2).unwrap();
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

    sscanf!(input3, "{name3}: {} {unit3}", name3, weight, unit3).unwrap();
    println!("Parsed: name={}, weight={}, unit={}", name3, weight, unit3);
    assert_eq!(name3, "Charlie");
    assert_eq!(weight, 35.5);
    assert_eq!(unit3, "kg");

    // Example 4: Pure generic placeholders
    println!("\n=== Pure generic placeholders ===");
    let input4 = "Diana: 28";
    let mut name4: String = String::new();
    let mut age4: u32 = 0;

    sscanf!(input4, "{}: {}", name4, age4).unwrap();
    println!("Parsed: name={}, age={}", name4, age4);
    assert_eq!(name4, "Diana");
    assert_eq!(age4, 28);
}

#[test]
fn test_variable_order_different_from_params() {
    // Test where variables in format string are in different order than parameters
    // Current implementation matches by position, not by variable name
    let input = "Score: 95, Player: Alice";
    let mut _player: String = String::new();
    let mut _score: u32 = 0;

    // Format string has {score} first, then {player}, but parameters are _player, _score
    // This means: _player gets "95", _score gets "Alice" - which will fail
    let result = sscanf!(input, "Score: {score}, Player: {player}", _player, _score);
    assert!(result.is_err()); // Should fail because "Alice" can't be parsed as u32
    
    // Correct way: match parameter order to format order
    let mut score2: u32 = 0;
    let mut player2: String = String::new();
    sscanf!(input, "Score: {score}, Player: {player}", score2, player2).unwrap();
    assert_eq!(score2, 95);
    assert_eq!(player2, "Alice");
}