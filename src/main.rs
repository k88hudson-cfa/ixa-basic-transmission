pub mod ixa_plus;
use ixa::prelude::*;

mod infection_manager;
mod model;
mod output_manager;
mod parameters;
mod population_manager;
mod total_infectiousness_multiplier;
mod transmission_manager;

// Helper for importing all extensions
// use crate::ext::*;
pub mod ext {
    pub use crate::infection_manager::InfectionManagerExt;
    pub use crate::parameters::ParametersExt;
    pub use crate::population_manager::PopulationManagerExt;
    pub use crate::transmission_manager::TransmissionManagerExt;
}

fn main() {
    // Set log level to debug in debug builds
    #[cfg(debug_assertions)]
    ixa::log::set_log_level(ixa::log::LevelFilter::Debug);

    // Run the model
    let mut context = model::setup().expect("Model setup failed");
    context.execute();
}
