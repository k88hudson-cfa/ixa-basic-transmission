mod contact_manager;
mod distr;
mod forecast;
mod infection_manager;
mod parameters;
mod rate_fn;

use infection_manager::*;
use ixa::prelude::*;
use parameters::*;

pub trait ModelContext: PluginContext {}
impl ModelContext for Context {}

fn main() {
    let mut context = Context::new();

    let &Params {
        max_time,
        seed,
        initial_incidence,
        initial_recovered,
        ..
    } = context.params();

    // Set the random seed.
    context.init_random(seed);

    // Add a plan to shut down the simulation after `max_time`, regardless of
    // what else is happening in the model.
    context.add_plan(max_time, |context| {
        context.shutdown();
    });

    // Initialize initial infections
    context.seed_initial_infection_status(initial_incidence, initial_recovered);
    context.start_infection_propagation_loop();

    // Run the simulation
    context.execute();
}

// Age -->
