use super::InfectiousnessRateFn;
use crate::{ModelContext, parameters::ParametersExt, rate_fn::ConstantRate};
use ixa::{HashMap, prelude::*};
use std::hash::Hash;

pub trait RateFnGenerator {
    type RateFnInstance: IntoDynRateFn;
    fn assign(&self, context: &impl ModelContext, person_id: PersonId) -> Self::RateFnInstance;
}

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

macro_rules! define_rate_generators {
    ($name:ident {
        $(
            $rate_fn_id:ident
        ),* $(,)?
    }
    ) => {

        #[derive(Hash, Eq, PartialEq, Clone, Copy)]
        pub enum $name {
            $(
                $rate_fn_id
            )*,
        }

        impl RateFnGenerator for $name {
            type RateFnInstance = RateFn;
            fn assign(&self, context: &impl ModelContext, person_id: PersonId) -> Self::RateFnInstance {
                match self {
                    $(
                        Self::$rate_fn_id => {
                            $rate_fn_id.assign(context, person_id)
                        }
                    ),*
                }
            }
        }

    }
}

struct InfectionRate;
impl RateFnGenerator for InfectionRate {
    type RateFnInstance = RateFn;
    fn assign(&self, context: &impl ModelContext, _: PersonId) -> Self::RateFnInstance {
        let params = context.param_infection_rate().clone();
        RateFn::ConstantRate(params.try_into().unwrap())
    }
}

define_rate_generators!(RateFnId { InfectionRate });

struct RateFnContainer<T: Hash + Eq + RateFnGenerator> {
    rate_fn_index: HashMap<(PersonId, T), usize>,
    rate_instances: Vec<T::RateFnInstance>,
}

impl<T: Hash + Eq + RateFnGenerator> RateFnContainer<T> {
    fn add_instance(
        &mut self,
        person_id: PersonId,
        rate_fn_id: T,
        instance: T::RateFnInstance,
    ) -> usize {
        self.rate_instances.push(instance);
        let index = self.rate_instances.len() - 1;
        self.rate_fn_index.insert((person_id, rate_fn_id), index);
        index
    }
    fn get_instance(&self, person_id: PersonId, rate_fn: T) -> Option<&T::RateFnInstance> {
        let index = *self.rate_fn_index.get(&(person_id, rate_fn))?;
        self.rate_instances.get(index)
    }
}

type RateFnData = RateFnContainer<RateFnId>;

define_data_plugin!(
    RateFnPlugin,
    RateFnData,
    RateFnData {
        rate_fn_index: HashMap::default(),
        rate_instances: Vec::new(),
    }
);

pub trait InfectiousnessRateExt: ModelContext {
    fn assign_infection_rate(&mut self, person_id: PersonId, rate_fn: RateFnId) {
        let instance = rate_fn.assign(self, person_id);
        self.get_data_mut(RateFnPlugin)
            .add_instance(person_id, rate_fn, instance);
    }
    fn get_person_rate_fn(
        &self,
        person_id: PersonId,
        rate_fn: RateFnId,
    ) -> Box<&dyn InfectiousnessRateFn> {
        let data = self.get_data(RateFnPlugin);
        let rate_fn = data
            .get_instance(person_id, rate_fn)
            .expect("Rate function not found");
        rate_fn.into()
    }
}
impl<T> InfectiousnessRateExt for T where T: ModelContext {}
