use crate::ext::*;
use crate::ixa_plus::rate_fn::*;
use ixa::{PersonPropertyChangeEvent, prelude::*};
use serde::{Deserialize, Serialize};

define_rng!(InfectionRng);
define_person_property_with_default!(InfectionStatus, I, I::Susceptible);

pub struct InfectionRate;
impl PerPersonRate for InfectionRate {
    fn assign(context: &impl ModelContext) {
        let r_distr = context.param_infection_rate();
        let duration_distr = context.param_infection_duration();
        let params = ConstantRateParams {
            r: context.sample_distr(InfectionRng, r_distr),
            infection_duration: context.sample_distr(InfectionRng, duration_distr),
        };
        RateFn::ConstantRate(params.try_into().unwrap())
    }
}

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
    #[allow(dead_code)]
    fn is_recovered(&self) -> bool {
        matches!(self, I::Recovered { .. })
    }
}

pub trait InfectionManagerExt: ModelContext {
    fn is_susceptible(&self, person_id: PersonId) -> bool {
        self.get_person_property(person_id, InfectionStatus) == I::Susceptible
    }
    #[allow(dead_code)]
    fn is_infectious(&self, person_id: PersonId) -> bool {
        self.get_person_property(person_id, InfectionStatus)
            .is_infectious()
    }
    #[allow(dead_code)]
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

    fn forecast_next_infection_time(
        context: &impl PluginContext,
        person_id: PersonId,
    ) -> Option<Forecast> {
        // Get the person's individual infectiousness
        let rate_fn = context.get_person_rate_fn(person_id, InfectionRate);
        // This scales infectiousness by the maximum possible infectiousness across all settings
        let scale = max_total_infectiousness_multiplier(context, person_id);
        let elapsed = context.get_elapsed_infection_time(person_id).ok()?;
        let total_rate_fn = ScaledRateFn::new(*rate_fn, scale, elapsed);

        // Draw an exponential and use that to determine the next time
        let exp = Exp::new(1.0).unwrap();
        let e = context.sample_distr(ForecastRng, exp);
        // Note: this returns None if forecasted > infectious period
        let t = total_rate_fn.inverse_cum_rate(e)?;

        let next_time = context.get_current_time() + t;
        let forecasted_total_infectiousness = total_rate_fn.rate(t);

        if next_time && forecasted_total_infectiousness > 0.0 {
            context.add_plan(next_time, move |context| {
                if evaluate_forecast(context, person_id, forecasted_total_infectiousness) {
                    self.attempt_transmission(context, person);
                }
                // Continue scheduling forecasts until the person recovers.
                schedule_next_forecasted_infection(context, person);
            });
        }
    }

    // This function should be called from the main loop whenever
    // someone is first infected. It assigns all their properties needed to
    // calculate intrinsic infectiousness
    fn set_infection_status(&mut self, target_id: PersonId, source_id: PersonId) {
        let infection_time = self.get_current_time();
        self.assign_rate(target_id, InfectionRate);
        self.set_person_property(
            target_id,
            InfectionStatus,
            I::Infectious {
                infection_time: Some(infection_time),
                infected_by: Some(source_id),
            },
        );
    }
    fn set_recovery_status(&mut self, person_id: PersonId) {
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
