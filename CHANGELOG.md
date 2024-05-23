# CHANGELOG

## Unreleased

-   **Breaking:** Remove `PartialEq` impls between outcomes and `judge::OrderState`. These did not uphold the Rust rules of `PartialEq`.
-   Make all `OrderOutcome` enums derive `Copy`
-   Only expose the preventing order in `AttackOutcome::Prevented` - exposing the supports was unnecessary complexity, and allowed for some confusing cases where the provided Prevent value couldn't have caused a failed move
-   Add `map_order` function to outcomes, which runs a mapping function over any orders referenced in the outcome. This makes it possible to convert order outcomes to be owned, for example.

## v0.1.3 (2024-05-22)

-   Fix adjudicator handling of PREVENT, DEFEND, and ATTACK strengths to avoid self-dislodgement.
-   Significantly improve DATC test coverage. Previously, not all tests were asserting the success or failure of move orders, allowing some bugs to remain undetected.

## v0.1.2 (2024-05-17)

-   Implement build-phase judging
-   Update Rust edition to 2021

## v0.1.1 (2022-12-28)

-   Fix warnings that panic message is not a literal [#8](https://github.com/TedDriggs/diplomacy/pull/8)
