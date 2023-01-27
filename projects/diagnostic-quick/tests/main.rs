mod for_rust_decimal;

use std::str::FromStr;

use diagnostic_quick::QResult;

#[test]
fn ready() -> QResult {
    println!("num: {}", num::BigInt::from_str("1234567890")?);
    println!("mime: {}", mime::Mime::from_str("text/plain")?);
    println!("semver: {}", semver::Version::from_str("1.0.0")?);
    println!("url: {}", url::Url::from_str("https://www.google.com")?);
    println!("email_address: {}", email_address::EmailAddress::from_str("johnstonsk@gmail.com")?);
    println!("toml: {}", toml::Value::from_str("key = true")?);
    println!("serde_json: {}", serde_json::Value::from_str("true")?);
    Ok(())
}
