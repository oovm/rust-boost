#![allow(unused, dead_code)]

mod simple;

#[cfg(feature = "serde")]
mod wire;

#[test]
fn ready() {
    println!("it works!")
}
