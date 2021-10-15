use qas::prelude::*;

qas!("main.c");


fn main() {
    println!("{}", fib(u32::MAX - 1))
    // unsafe {
    //     println!("Date: {}\nTime: {}\nFunc: {}\nFile: {}\nLine: {}", date().to_rust(), time().to_rust(), func().to_rust(), file().to_rust(), line())
    // }
}
