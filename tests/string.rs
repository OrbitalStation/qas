use qas::prelude::*;

qas!("tests/c/string.c");

#[cfg(test)]
#[test]
fn main() {
    assert_eq!(unsafe { hi().to_rust() }, "Hi there");
    assert_eq!(unsafe { return_same("Return me\0".to_c()).to_rust() }, "Return me");
    assert_eq!(unsafe { hi().to_rust().to_c().to_rust() }, "Hi there")
}
