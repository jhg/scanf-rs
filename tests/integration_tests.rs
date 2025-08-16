use scanf_proc_macro::sscanf;

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
    // Procedural macro: placeholders nombrados asignan a variables con ese nombre
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
    let mut _player: u32 = 0; // Falla al parsear "Alice" como u32
    let result = sscanf!(input, "Score: {score}, Player: {_player}");
    assert!(result.is_err());
    assert_eq!(score, "95"); // usar variable
}
