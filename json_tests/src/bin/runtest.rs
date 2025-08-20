//! Runs tests from a JSON file of Diplomacy test cases.

use json_tests::case::{Cases, DidPass, RawTestCase, TestCase};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("Usage: testgen <path to json file>")
        .map_err(anyhow::Error::msg)?;
    let json = std::fs::read_to_string(&path)?;
    let cases: Vec<TestCase> = serde_json::from_str::<Cases<RawTestCase>>(&json)
        .map_err(|e| anyhow::Error::new(e).context(format!("Parsing {path}")))?
        .cases
        .into_iter()
        .filter_map(|v| match v {
            RawTestCase::Case(test_case) => Some(test_case),
            RawTestCase::Todo(e) => {
                eprintln!("Skipping test case '{}'. {}", e.info, e.todo);
                None
            }
        })
        .collect();

    let mut failures = vec![];

    for test in cases {
        let result = test.run();
        if !result.did_pass() {
            failures.push(test.full_name());
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
