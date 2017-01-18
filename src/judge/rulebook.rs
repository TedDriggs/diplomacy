use super::calc::{atk_result, max_prevent_result, resistance_result, dislodger_of};
use super::resolver::{Adjudicate, ResolverState, ResolverContext};
use super::strength::MoveOutcome;
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
        use order::MainCommand::*;
        (match order.command {
                Move(..) => Rulebook::adjudicate_move(context, resolver, order).is_successful(),

                // A support order "succeeds" if the support is not cut. This doesn't
                // necessarily mean support got applied.
                Support(..) => !support::is_order_cut(context, resolver, order),

                // Hold and convoy orders succeed when the unit is not dislodged.
                Hold | Convoy(..) => dislodger_of(context, resolver, order).is_none(),
            })
            .into()
    }

    /// Determine the outcome for a move order. Will panic if order is not a move.
    fn adjudicate_move<'a>(ctx: &'a ResolverContext<'a>,
                           rslv: &mut ResolverState<'a, Self>,
                           ord: &'a MappedMainOrder)
                           -> MoveOutcome<'a> {
        MoveOutcome::new(atk_result(ctx, rslv, ord).expect("Supposed to be move order"),
                         max_prevent_result(ctx, rslv, ord),
                         resistance_result(ctx, rslv, ord))
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