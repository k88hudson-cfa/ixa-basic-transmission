use super::ConstantRate;
use super::InfectiousnessRateFn;
use crate::ixa_plus::query::QueryMap;
use ixa::{HashMap, prelude::*};
use std::hash::Hash;

pub trait RateFnGenerator<C: PluginContext>: Clone {
    #[allow(private_bounds)]
    type RateFnInstance: IntoDynRateFn;
    fn name(&self) -> &'static str;
    fn id(&self) -> RateFnId {
        RateFnId(self.name())
    }
    fn assign(&self, context: &C, person_id: PersonId) -> Self::RateFnInstance;
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
struct RateFnId(&'static str);

pub enum RateFn {
    ConstantRate(ConstantRate),
}

impl<'a> Into<Box<&'a dyn InfectiousnessRateFn>> for &'a RateFn {
    fn into(self) -> Box<&'a dyn InfectiousnessRateFn> {
        match self {
            RateFn::ConstantRate(rate) => Box::new(rate),
        }
    }
}

trait IntoDynRateFn {}
impl<T> IntoDynRateFn for T where for<'a> &'a T: Into<Box<&'a dyn InfectiousnessRateFn>> {}

pub struct RateFnDataContainer {
    per_person_rates: HashMap<(PersonId, RateFnId), usize>,
    query_rates: QueryMap<usize>,
    rate_instances: Vec<RateFn>,
}

impl Default for RateFnDataContainer {
    fn default() -> Self {
        Self {
            per_person_rates: HashMap::default(),
            rate_instances: Vec::new(),
        }
    }
}

impl RateFnDataContainer {
    pub fn add_instance<C: PluginContext, G: RateFnGenerator<C>>(
        &mut self,
        person_id: PersonId,
        generator: G,
        instance: RateFn,
    ) -> usize {
        self.rate_instances.push(instance);
        let index = self.rate_instances.len() - 1;
        self.per_person_rates
            .insert((person_id, generator.id()), index);
        index
    }
    pub fn get_instance<C: PluginContext, G: RateFnGenerator<C>>(
        &self,
        person_id: PersonId,
        generator: G,
    ) -> Option<&RateFn> {
        let index = *self.per_person_rates.get(&(person_id, generator.id()))?;
        self.rate_instances.get(index)
    }
}

define_data_plugin!(
    RateFnPlugin,
    RateFnDataContainer,
    RateFnDataContainer::default()
);

pub trait RateFnExt: PluginContext {
    fn assign_rate<G: RateFnGenerator<Self, RateFnInstance = RateFn>>(
        &mut self,
        person_id: PersonId,
        rate: G,
    ) {
        let instance = rate.assign(self, person_id);
        let data = self.get_data_mut(RateFnPlugin);
        data.add_instance(person_id, rate, instance);
    }
    fn get_person_rate_fn<G: RateFnGenerator<Self, RateFnInstance = RateFn>>(
        &self,
        person_id: PersonId,
        generator: G,
    ) -> Box<&dyn InfectiousnessRateFn> {
        let data = self.get_data(RateFnPlugin);
        let rate_fn = data
            .get_instance(person_id, generator)
            .expect("Rate function not found");
        rate_fn.into()
    }
}
impl<T> RateFnExt for T where T: PluginContext {}
