use qas::prelude::*;

qas!("tests/c/notation.c");

#[cfg(test)]
#[test]
fn main() {
    assert_eq!(binary_3(), 3);
    assert_eq!(binary_127(), 127);
    assert_eq!(octal_7(), 7);
    assert_eq!(octal_100(), 100);
    assert_eq!(hexadecimal_98(), 98);
    assert_eq!(hexadecimal_answer(), 42);
    assert_eq!(hexadecimal_answer_uppercase(), 42);
}
