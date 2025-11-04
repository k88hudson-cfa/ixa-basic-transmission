use ixa::prelude::*;
use ixa::trace;
use rand_distr::Exp;

use crate::ext::*;
use crate::infection_manager::InfectionRate;
use crate::ixa_plus::rate_fn::*;

define_rng!(ForecastRng);

pub struct Forecast {
    pub next_time: f64,
    pub forecasted_total_infectiousness: f64,
}

/// Calculate the scaling factor that accounts for the total infectiousness
/// for a person, given factors related to their environment, such as the number of people
/// they come in contact with or how close they are.
/// This is used to scale the intrinsic infectiousness function of that person.
/// All modifiers of the infector's intrinsic infectiousness are aggregated and returned
/// as a single float to multiply by the base total infectiousness.
/// This assumes that transmission modifiers of total infectiousness are independent of
/// the setting type and are linear
pub fn calc_actual_total_infectiousness_multiplier(
    _context: &impl PluginContext,
    _person_id: PersonId,
) -> f64 {
    1.0
}

/// Calculate the maximum possible scaling factor for total infectiousness
/// for a person, given information we know at the time of a forecast.
/// The modifier used for intrinsic infectiousness is ignored because all modifiers must
/// be less than or equal to one.
pub fn max_total_infectiousness_multiplier(
    _context: &impl PluginContext,
    _person_id: PersonId,
) -> f64 {
    1.0
}

/// Forecast of the next expected infection time, and the expected rate of
/// infection at that time.
pub fn get_forecast(context: &impl ModelContext, person_id: PersonId) -> Option<Forecast> {
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

    Some(Forecast {
        next_time,
        forecasted_total_infectiousness,
    })
}

/// Evaluates a forecast against the actual current infectious,
/// Returns a contact to be infected or None if the forecast is rejected
pub fn evaluate_forecast(
    context: &mut Context,
    person_id: PersonId,
    forecasted_total_infectiousness: f64,
) -> bool {
    let rate_fn = context.get_person_rate_fn(person_id, InfectionRate);

    let total_multiplier = calc_actual_total_infectiousness_multiplier(context, person_id);
    let total_rate_fn = ScaledRateFn::new(*rate_fn, total_multiplier, 0.0);

    let elapsed_t = context.get_elapsed_infection_time(person_id).unwrap();
    let current_infectiousness = total_rate_fn.rate(elapsed_t);

    assert!(
        // 1e-10 is a small enough tolerance for floating point comparison.
        current_infectiousness <= forecasted_total_infectiousness + 1e-10,
        "Person {person_id}: Forecasted infectiousness must always be greater than or equal to current infectiousness. Current: {current_infectiousness}, Forecasted: {forecasted_total_infectiousness}"
    );

    // If they are less infectious as we expected...
    if current_infectiousness < forecasted_total_infectiousness {
        // Reject with the ratio of current vs the forecasted
        if !context.sample_bool(
            ForecastRng,
            current_infectiousness / forecasted_total_infectiousness,
        ) {
            trace!("Person {person_id}: Forecast rejected");

            return false;
        }
    }

    true
}

pub fn schedule_recovery(context: &mut impl PluginContext, person: PersonId) {
    let infection_duration = context
        .get_person_rate_fn(person, InfectionRate)
        .infection_duration();
    let recovery_time = context.get_current_time() + infection_duration;
    context.add_plan(recovery_time, move |context| {
        context.set_recovery_status(person);
    });
}
