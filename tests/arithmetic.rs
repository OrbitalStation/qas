use qas::prelude::*;

qas!("tests/c/arithmetic.c");

#[cfg(test)]
#[test]
fn main() {
    assert_eq!(add(29, 41), 70);
    assert_eq!(mulf(2.5, 4.0), 10.0);
    assert_eq!(divd(6.9, 3.0), 2.3000000000000003);
}
