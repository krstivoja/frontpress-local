//! Tiny helper used by the e2e test: print the same bcrypt hash the app
//! writes into config.php, so we can prove PHP's password_verify() accepts it.
//! Usage: cargo run --example genhash -- <password>

fn main() {
    let pw = std::env::args().nth(1).expect("usage: genhash <password>");
    println!("{}", bcrypt::hash(pw, 12).expect("bcrypt"));
}
