#![cfg(test)]

mod test_cpu;


// Example function to test
fn add(a: u8, b: u8) -> u8 {
    a + b
}

#[test]
fn test_add() {
    assert_eq!(add(2, 3), 5);
}

