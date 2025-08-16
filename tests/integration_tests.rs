use scanf_proc_macro::sscanf;

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
    // Con el nuevo macro procedural, variables nombradas se capturan implícitamente
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
    // Forzamos error usando separador inexistente para provocar panic al unwrap
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
