extern crate cc;

fn main() {
    cc::Build::new()
        .file("gpio.c")
        .compile("gpio.a");
}