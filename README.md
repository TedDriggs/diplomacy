# diplomacy

[![Build Status](https://github.com/TedDriggs/diplomacy/workflows/CI/badge.svg)](https://github.com/TedDriggs/diplomacy/actions)
[![Latest Version](https://img.shields.io/crates/v/diplomacy.svg)](https://crates.io/crates/diplomacy)

The `diplomacy` crate provides a [DATC-compliant](https://webdiplomacy.net/doc/DATC_v3_0.html) adjudicator for the game Diplomacy.
In Diplomacy, players secretly submit orders to a central judge, and all orders are resolved simultaneously.
Order outcomes depend on one another, making correct adjudication difficult to implement.

# Goals

This project aims to make innovation in the Diplomacy user experience realm easier.
It seeks to achieve that by:

1. Providing a library that works in multiple environments, including server-side, in-browser, or in a native mobile app.
2. Providing good feedback on why an order succeeded or failed

# Non-Goals

This is not going to be a complete Diplomacy app.
Any sort of persistence or UI is out of scope.

# Optional Features

-   `serde`: Enable serialization and deserialization of many crate types.
-   `dependency-graph`: Add resolver tracing that generates GraphViz-compatible dependency visualizations for main phase resolution.
