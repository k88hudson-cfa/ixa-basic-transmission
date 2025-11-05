mod infection_manager;
pub mod ixa_plus;
mod parameters;
mod population_manager;
mod total_infectiousness_multiplier;
mod transmission_manager;

use infection_manager::*;
use ixa::prelude::*;
use parameters::*;
use population_manager::*;

// Helper for importing all extensions
pub mod ext {
    pub use crate::infection_manager::InfectionManagerExt;
    pub use crate::parameters::ParametersExt;
    pub use crate::transmission_manager::TransmissionManagerExt;
}

define_rng!(PopulationRng);

fn main() {
    let mut context = Context::new();

    let &Params {
        max_time,
        seed,
        population_size,
        p_initial_recovered,
        p_initial_incidence,
        ..
    } = context.params();

    // Set the random seed.
    context.init_random(seed);

    // Add a plan to shut down the simulation after `max_time`, regardless of
    // what else is happening in the model.
    context.add_plan(max_time, |context| {
        context.shutdown();
    });

    // Seed the population with initial infected/recovered individuals
    context
        .seed_weighted_population(
            population_size,
            vec![
                // Recovered people can't be infected again
                (p_initial_recovered, |context, person_id| {
                    context.recover_person(person_id, None)?;
                    Ok(())
                }),
                // Infecting each person will kick off an infection loop which schedules their
                // next forecasted infection, as well as their recovery time.
                // See infection_manager.rs for details
                (p_initial_incidence, |context, person_id| {
                    context.infect_person(person_id, None, None);
                    Ok(())
                }),
            ],
        )
        .expect("Failed to seed population");

    // Run the simulation
    context.execute();
}
