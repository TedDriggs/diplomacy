use order::{MainCommand, Order};
use super::calc::{atk_result, max_prevent_result, resistance_result, dislodger_of};
use super::resolver::{Adjudicate, ResolverState, ResolverContext};
use super::strength::{MoveOutcome};
use super::{MappedMainOrder, OrderState};
use super::support;

#[derive(Debug, Clone, Default)]
pub struct Rulebook;

impl Rulebook {
    fn adjudicate<'a>(&self,
                      context: &'a ResolverContext<'a>,
                      resolver: &mut ResolverState<'a, Self>,
                      order: &'a MappedMainOrder)
                      -> OrderState {
        (match order.command {
            MainCommand::Move(ref dest) => {
                MoveOutcome::new(atk_result(context, resolver, order).expect("Guaranteed to be move order"),
                                 max_prevent_result(context, resolver, dest.province()),
                                 resistance_result(context, resolver, order)).is_successful()
            }
            MainCommand::Support(..) => (!support::is_order_cut(context, resolver, order)),
            MainCommand::Hold | MainCommand::Convoy(..) => dislodger_of(context, resolver, order).is_none(),
        }).into()
    }
}

impl Adjudicate for Rulebook {
    fn adjudicate<'a>(&self,
                      context: &'a ResolverContext<'a>,
                      resolver: &mut ResolverState<'a, Self>,
                      order: &'a MappedMainOrder)
                      -> OrderState {
        Rulebook::adjudicate(&self, context, resolver, order)
    }
}