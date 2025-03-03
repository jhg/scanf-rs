use std::io;

pub fn placeholder_type_does_not_match(_: ()) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "Placeholder type does not match the variable type",
    ))
}

pub fn not_enough_placeholders(_: ()) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "There is not enough input placeholders for all variables.",
    ))
}
