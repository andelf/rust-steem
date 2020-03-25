
extern crate cc;

fn main() {
    cc::Build::new()
        .file("city/city.cc")
        .flag("-msse4.2")
        .cpp(true)
        .include("city")
        .compile("city");
}