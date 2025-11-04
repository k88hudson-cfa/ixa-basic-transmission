use crate::ixa_plus::rate_fn::RateFn;
use ixa::{people::Query, prelude::*};

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
struct RateFnId(&'static str);

pub trait RateFnGenerator<C: PluginContext>: Clone {
    type RateFnInstance;
    fn id(&self) -> RateFnId;
    fn assign_rate<Q: Query>(
        &self,
        context: &C,
        person_id: PersonId,
        query: Q,
    ) -> Self::RateFnInstance;
}

pub trait PerPersonRate<C: PluginContext> {
    fn name() -> &'static str;
    fn assign(context: &C, person_id: PersonId) {}
}

pub enum RateGeneratorPhase {
    Initialization
}

impl<T: Clone, C: PluginContext> RateFnGenerator<C> for T
where
    T: PerPersonRate<C>,
{
    type RateFnInstance = RateFn;
    fn id(&self) -> RateFnId {
        RateFnId(self.name())
    }
    fn assign_rate<Q: Query>(
        &self,
        context: &C,
        person_id: PersonId,
        query: Q,
        phase:
    ) -> Self::RateFnInstance {
        self.assign
    }
}
