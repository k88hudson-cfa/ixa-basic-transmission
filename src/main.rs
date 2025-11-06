mod infection_manager;
pub mod ixa_plus;
mod model;
mod output_manager;
mod parameters;
mod population_manager;
mod simulation_event;
mod total_infectiousness_multiplier;
mod transmission_manager;

// Helper for importing all extensions
// use crate::ext::*;
pub mod ext {
    pub use crate::infection_manager::InfectionManagerExt;
    pub use crate::output_manager::OutputManagerExt;
    pub use crate::parameters::ParametersExt;
    pub use crate::population_manager::PopulationManagerExt;
    pub use crate::transmission_manager::TransmissionManagerExt;
}

use crate::ixa_plus::params_macro::IxaParameters;
use crate::output_manager::OutputManagerExt;
use anyhow::Result;
use ixa::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    #[cfg(debug_assertions)]
    crate::ixa_plus::log::set_log_level(crate::ixa_plus::log::LevelFilter::Debug);

    #[cfg(not(debug_assertions))]
    crate::ixa_plus::log::init_default();

    // Use mise run --params <file> to override default parameters
    let params = parameters::Params::from_args();
    let mut context = model::setup(params)?;
    context.execute();
    context.log_stats();

    Ok(())
}
