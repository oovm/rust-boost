#![allow(unused, dead_code)]

mod support;

#[cfg(feature = "terminal")]
mod simple;

#[test]
fn ready() {
    println!("it works!")
}
