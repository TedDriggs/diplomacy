use diplomacy::judge::OrderState;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct OrderOutcomeWithState<O> {
    /// Whether the order succeeded.
    succeeds: bool,
    /// The specific outcome of the order.
    outcome: O,
}

impl<O> OrderOutcomeWithState<O>
where
    for<'a> &'a O: Into<OrderState>,
{
    pub fn new(outcome: O) -> Self {
        Self {
            succeeds: bool::from((&outcome).into()),
            outcome,
        }
    }
}
