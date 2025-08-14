//! Generates Rust tests from a JSON file of Diplomacy test cases.

use json_tests::case::{Cases, RawTestCase};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("Usage: testgen <path to json file>")
        .map_err(anyhow::Error::msg)?;
    let json = std::fs::read_to_string(&path)?;
    let cases = serde_json::from_str::<Cases<RawTestCase>>(&json)
        .map_err(|e| anyhow::Error::new(e).context(format!("Parsing {path}")))?;

    println!(
        "// @generated\n{}",
        prettyplease::unparse(&syn::parse_quote!(#cases))
    );

    Ok(())
}
