use super::calc::{max_prevent_result, dislodger_of, indiscriminate_atk_strength, is_head_to_head};
use super::resolver::{Adjudicate, ResolverState, ResolverContext};
use super::{MappedMainOrder, OrderState};
use super::support::{self, SupportOutcome};
use order::Command;
use judge::strength::Strength;

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
                Move(..) => Rulebook::adjudicate_move2(context, resolver, order).into(),

                // A support order "succeeds" if the support is not cut. This doesn't
                // necessarily mean support got applied.
                Support(..) => {
                    Rulebook::adjudicate_support(context, resolver, order).is_successful()
                }

                // Hold and convoy orders succeed when the unit is not dislodged.
                Hold | Convoy(..) => dislodger_of(context, resolver, order).is_none(),
            })
            .into()
    }

    pub fn adjudicate_move2<'a>(ctx: &'a ResolverContext<'a>,
                                rslv: &mut ResolverState<'a, Self>,
                                ord: &'a MappedMainOrder)
                                -> AttackOutcome {
        if ord.command.move_dest() == Some(&ord.region) {
            AttackOutcome::MoveToSelf
        } else {
            let atk_with_ff = indiscriminate_atk_strength(ctx, rslv, ord);
            let prevent = max_prevent_result(ctx, rslv, ord);
            if atk_with_ff <= prevent.strength() {
                AttackOutcome::Prevented
            } else {
                if let Some(occupier) =
                       ctx.find_order_to_province(ord.command.move_dest().unwrap().into()) {
                    let mut resistance = 0;
                    
                    // head-to-heads and non-moves get their support
                    if !occupier.command.is_move() || is_head_to_head(occupier, ord) {
                        resistance = 1 + support::find_successful_for(ctx, rslv, occupier).len();
                    }
                    // failed exits resist with strength 1 (the unit trapped in the province)
                    else if rslv.resolve(ctx, occupier) == OrderState::Fails {
                        resistance = 1;
                    }

                    // Head-to-head, failed exit, and hold cases all collapse in friendly fire.
                    if resistance > 0 && ord.nation == occupier.nation {
                        AttackOutcome::FriendlyFire
                    } else if atk_with_ff <= resistance {
                        AttackOutcome::LostHeadToHead
                    } else {
                        AttackOutcome::Succeeds
                    }
                } else {
                    AttackOutcome::Succeeds
                }
            }
        }
    }

    pub fn adjudicate_support<'a>(ctx: &'a ResolverContext<'a>,
                                  rslv: &mut ResolverState<'a, Self>,
                                  ord: &'a MappedMainOrder)
                                  -> SupportOutcome<'a> {
        if support::is_supporting_self(ord) {
            SupportOutcome::SupportingSelf
        } else {
            match support::find_cutting_order(ctx, rslv, ord) {
                Some(cutter) => SupportOutcome::CutBy(cutter),
                None => SupportOutcome::NotDisrupted,
            }
        }
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

pub enum AttackOutcome {
    MoveToSelf,
    NoPath,
    FriendlyFire,
    Prevented,
    LostHeadToHead,
    Succeeds,
}

impl From<AttackOutcome> for OrderState {
    fn from(ao: AttackOutcome) -> Self {
        use self::AttackOutcome::*;
        match ao {
            Succeeds => OrderState::Succeeds,
            NoPath | MoveToSelf | FriendlyFire | Prevented | LostHeadToHead => OrderState::Fails,
        }
    }
}

impl From<AttackOutcome> for bool {
    fn from(ao: AttackOutcome) -> Self {
        OrderState::from(ao).into()
    }
}