use crate::ModelContext;
use crate::forecast;
use crate::rate_fn::RateFn;
use anyhow::Result;
use ixa::{PersonPropertyChangeEvent, prelude::*};
use rand_distr::Binomial;
use serde::{Deserialize, Serialize};

define_rng!(InfectionRng);

define_person_property_with_default!(InfectionStatus, I, I::Susceptible);

// define_rate!(InfectiousRate, |context, person_id| {
//     let params = context.param_infection_rate().clone();
//     let rate: RateFn = params.try_into().unwrap();
//     rate
// });

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub enum I {
    Susceptible,
    Infectious {
        infection_time: Option<f64>,
        infected_by: Option<PersonId>,
    },
    Recovered {
        infection_time: Option<f64>,
        recovery_time: Option<f64>,
    },
}
impl I {
    fn initial_infected() -> Self {
        Self::Infectious {
            infection_time: Some(0.0),
            infected_by: None,
        }
    }
    fn initial_recovered() -> Self {
        Self::Recovered {
            infection_time: None,
            recovery_time: None,
        }
    }
    fn is_infectious(&self) -> bool {
        matches!(self, I::Infectious { .. })
    }
    fn is_recovered(&self) -> bool {
        matches!(self, I::Recovered { .. })
    }
}

pub trait InfectionManagerExt: ModelContext {
    fn is_susceptible(&self, person_id: PersonId) -> bool {
        self.get_person_property(person_id, InfectionStatus) == I::Susceptible
    }
    fn is_infectious(&self, person_id: PersonId) -> bool {
        self.get_person_property(person_id, InfectionStatus)
            .is_infectious()
    }
    fn is_recovered(&self, person_id: PersonId) -> bool {
        self.get_person_property(person_id, InfectionStatus)
            .is_recovered()
    }
    fn sample_proportional(&self, p: f64) -> usize {
        let pop = self.get_current_population() as u64;
        let dist = Binomial::new(pop, p).unwrap();
        self.sample_distr(InfectionRng, dist) as usize
    }
    fn seed_initial_infection_status(&mut self, initial_infected: f64, initial_recovered: f64) {
        let infected = self.sample_proportional(initial_infected);
        let recovered = self.sample_proportional(initial_recovered);
        let total = infected + recovered;
        let person_ids = self.sample_people(InfectionRng, (), total);
        for (i, person_id) in person_ids.iter().enumerate() {
            if i < infected {
                self.set_person_property(*person_id, InfectionStatus, I::initial_infected());
            } else {
                self.set_person_property(*person_id, InfectionStatus, I::initial_recovered());
            }
        }
    }
    fn start_infection_propagation_loop(&mut self) {
        // Subscribe to the person becoming infectious to trigger the infection propagation loop
        self.subscribe_to_event(
            |context, event: PersonPropertyChangeEvent<InfectionStatus>| {
                if event.current.is_infectious() {
                    return;
                }
                forecast::schedule_next_forecasted_infection(context, event.person_id);
                forecast::schedule_recovery(context, event.person_id);
            },
        );
    }
    // This function should be called from the main loop whenever
    // someone is first infected. It assigns all their properties needed to
    // calculate intrinsic infectiousness
    fn infect_person(&mut self, target_id: PersonId, source_id: PersonId) {
        let infection_time = self.get_current_time();
        self.set_person_property(
            target_id,
            InfectionStatus,
            I::Infectious {
                infection_time: Some(infection_time),
                infected_by: Some(source_id),
            },
        );
    }
    fn recover_person(&mut self, person_id: PersonId) {
        let recovery_time = self.get_current_time();
        let I::Infectious { infection_time, .. } =
            self.get_person_property(person_id, InfectionStatus)
        else {
            panic!("Person {person_id} is not infectious")
        };
        self.set_person_property(
            person_id,
            InfectionStatus,
            I::Recovered {
                recovery_time: Some(recovery_time),
                infection_time,
            },
        );
    }
    fn get_elapsed_infection_time(&self, person_id: PersonId) -> Result<f64> {
        let I::Infectious { infection_time, .. } =
            self.get_person_property(person_id, InfectionStatus)
        else {
            anyhow::bail!("Person {person_id} is not infectious");
        };
        Ok(self.get_current_time() - infection_time.unwrap_or(0.0))
    }
}
impl<T> InfectionManagerExt for T where T: ModelContext {}
