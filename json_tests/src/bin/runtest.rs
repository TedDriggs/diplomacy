//! Runs tests from a JSON file of Diplomacy test cases.

use json_tests::case::{Cases, DidPass, RawTestCase};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("Usage: testgen <path to json file>")
        .map_err(anyhow::Error::msg)?;
    execute_test_cases(&std::fs::read_to_string(&path)?)
}

fn execute_test_cases(json: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cases = serde_json::from_str::<Cases<RawTestCase>>(json)
        .map_err(|e| anyhow::Error::new(e).context("Parsing JSON"))?;

    let mut failures = vec![];

    for (test, result) in cases.run() {
        let Some(result) = result else {
            match test {
                RawTestCase::Todo(case) => {
                    eprintln!("Skipping test case '{}'. {}", case.info, case.todo)
                }
                RawTestCase::Case(case) => {
                    eprintln!("Test case '{}' did not produce a result.", case.full_name())
                }
            }
            continue;
        };

        if !result.did_pass() {
            failures.push(test.full_name());

            let RawTestCase::Case(test) = test else {
                eprintln!(
                    "Test case '{}' is a todo, but it failed with mismatches: {}",
                    test.full_name(),
                    serde_json::to_string_pretty(&result).unwrap()
                );
                continue;
            };

            eprintln!(
                "Test case '{}' failed with mismatches: {}\n{}",
                test.info,
                serde_json::to_string_pretty(&test.body).unwrap(),
                serde_json::to_string_pretty(&result).unwrap()
            );
        }
    }

    if !failures.is_empty() {
        eprintln!("Failures:\n{}", failures.join("\n"));
        let fail_count = failures.len();
        Err(anyhow::Error::msg(format!("{fail_count} test cases failed")).into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::execute_test_cases;

    /// Make sure the DATC tests run as part of `cargo test`, even if the generated Rust version is out of date.
    #[test]
    fn datc() {
        execute_test_cases(include_str!("../../datc.json")).expect("DATC test cases should pass");
    }
}
