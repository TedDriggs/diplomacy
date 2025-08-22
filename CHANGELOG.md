# CHANGELOG

## v0.2.0

-   **Breaking:** Remove `PartialEq` impls between outcomes and `judge::OrderState`. These did not uphold the Rust rules of `PartialEq`.
-   **Breaking:** Rename `OrderOutcome::Invalid` to `OrderOutcome::Ilegal` to match the distinction drawn by the DATC.
-   **Breaking:** Change `judge::build::Context` to `judge::build::Submission` and start requiring a ruleset be passed for adjudication. [#16](https://github.com/TedDriggs/diplomacy/issues/16)
-   Make all `OrderOutcome` enums derive `Copy`
-   Only expose the preventing order in `AttackOutcome::Prevented` - exposing the supports was unnecessary complexity, and allowed for some confusing cases where the provided Prevent value couldn't have caused a failed move
-   Add `map_order` function to outcomes, which runs a mapping function over any orders referenced in the outcome. This makes it possible to convert order outcomes to be owned, for example.
-   Add `retreat::Start::from_raw_parts`; this allows construction of a start from a deserialized snapshot. `Start` deliberately does not directly implement `Serialize` or `Deserialize`, as its internal representation is not suitable for serialization.
-   Add `PartialEq` and `Eq` to `retreat::Start` to allow for comparison of deserialized starts with computed ones. With this, callers can store the retreat start to ensure players see exactly the same game state, while also detecting if there is any drift in adjudication.

## v0.1.3 (2024-05-22)

-   Fix adjudicator handling of PREVENT, DEFEND, and ATTACK strengths to avoid self-dislodgement.
-   Significantly improve DATC test coverage. Previously, not all tests were asserting the success or failure of move orders, allowing some bugs to remain undetected.

## v0.1.2 (2024-05-17)

-   Implement build-phase judging
-   Update Rust edition to 2021

## v0.1.1 (2022-12-28)

-   Fix warnings that panic message is not a literal [#8](https://github.com/TedDriggs/diplomacy/pull/8)
