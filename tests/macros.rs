use qas::prelude::*;

qas!("tests/c/macros.c");

#[cfg(test)]
#[test]
fn main() {
    assume_1();
    assume_2()
}
