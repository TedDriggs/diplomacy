//! Crate for testing JSON serialization and deserialization
//! of [`diplomacy`] data.
//!
//! The main crate does not have any direct need for [`serde_json`], so these
//! tests are kept separately.

pub mod case;
mod map_key;
mod outcome_with_state;

pub use map_key::{with_map_key, MapKey};
pub use outcome_with_state::OrderOutcomeWithState;

#[cfg(test)]
mod tests {
    use std::{fmt::Display, str::FromStr};

    use anyhow::Context;
    use diplomacy::{
        geo::standard_map,
        judge::{MappedBuildOrder, MappedMainOrder, Rulebook, Submission},
    };
    use serde::{de::DeserializeOwned, ser::Serializer, Serialize};

    /// Parse an order, serialize it to JSON, deserialize it, and check that the two structs are equal.
    fn roundtrip_order<T>(ord: &str) -> anyhow::Result<()>
    where
        T: FromStr + Serialize + DeserializeOwned + Display + Eq,
        <T as FromStr>::Err: Send + Sync + std::error::Error + 'static,
    {
        let parsed: T = ord.parse().with_context(|| "Parsing order")?;
        let serialized =
            serde_json::to_string_pretty(&parsed).with_context(|| "Serializing order")?;
        let roundtripped: T =
            serde_json::from_str(&serialized).with_context(|| "Deserializing order")?;
        if parsed == roundtripped {
            Ok(())
        } else {
            Err(anyhow::Error::msg(format!("Input: {ord} | Parsed: {parsed} | Roundtripped: {roundtripped} | Serialized: {serialized}")))
        }
    }

    #[track_caller]
    fn roundtrip_orders<T>(orders: impl IntoIterator<Item = &'static str>)
    where
        T: FromStr + Serialize + DeserializeOwned + Display + Eq,
        <T as FromStr>::Err: Send + Sync + std::error::Error + 'static,
    {
        let results = orders
            .into_iter()
            .map(|ord| roundtrip_order::<T>(ord).with_context(|| ord))
            .filter_map(Result::err)
            .collect::<Vec<_>>();

        for error in &results {
            eprintln!("{:#}", error);
        }

        if !results.is_empty() {
            panic!("{} roundtrips failed", results.len());
        }
    }

    #[test]
    fn serialize() {
        let orders: Vec<MappedMainOrder> = vec![
            "TUR: F ank -> con",
            "TUR: A bul -> con",
            "TUR: A smy -> ank",
            "TUR: A con -> smy",
        ]
        .into_iter()
        .map(|ord| ord.parse().unwrap())
        .collect();

        let submission = Submission::with_inferred_state(standard_map(), orders);
        let outcome = submission.adjudicate(Rulebook);
        let mut ser = serde_json::Serializer::pretty(std::io::stdout());
        ser.collect_seq(outcome.all_orders_with_outcomes()).unwrap();
    }

    #[test]
    fn roundtrip_main_orders() {
        roundtrip_orders::<MappedMainOrder>(vec![
            "TUR: F ank hold",
            "TUR: A bul -> con",
            "TUR: A bul -> con via convoy",
            "TUR: F aeg convoys bul -> con",
            "TUR: A rum supports A bul -> con",
            "AUS: A tri -> ser",
            "AUS: A ser -> bul",
            "TUR: A bul -> tri",
            "TUR: F aeg convoys bul -> tri",
            "TUR: F ion convoys bul -> tri",
            "TUR: F adr convoys bul -> tri",
            "ITA: F nap -> ion",
            "ENG: F iri supports F nao -> mao",
            "ENG: F nao -> mao",
            "FRA: F spa(nc) supports F mao",
            "FRA: F mao holds",
            "ITA: F lyo -> spa(sc)",
        ]);
    }

    #[test]
    fn roundtrip_build_orders() {
        roundtrip_orders::<MappedBuildOrder>(vec![
            "GER: A war build",
            "GER: A ber build",
            "GER: A mun build",
            "RUS: F stp(nc) build",
            "FRA: A par disband",
        ]);
    }
}
