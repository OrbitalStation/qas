fn main() {
    println!("{}", qas::c::start(std::fs::read_to_string("main.c").unwrap()));
}
