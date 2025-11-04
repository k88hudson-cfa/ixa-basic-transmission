mod forecast;
mod infection_manager;
pub mod ixa_plus;
mod parameters;
mod transmission_manager;

use infection_manager::*;
use ixa::prelude::*;
use parameters::*;
use transmission_manager::*;

// #[macro_export]
// macro_rules! define_ext {
//     (
//         $(#[$meta:meta])*
//         $vis:vis trait $name:ident $(<$($generics:tt)*>)? $body:tt
//         $(where $($bounds:tt)*)?
//     ) => {
//         $(#[$meta])*
//         $vis trait $name: $($($bounds)*)? $(<$($generics)*>)? {
//             fn describe() -> &'static str {
//                 stringify!($name)
//             }
//             $body
//         }
//         impl <T$($(, $generics)*)?> $name$(<$($generics)*>)? for T where T: ModelContext {}
//     };
// }

// impl<T: PluginContext + ParametersExt + TransmissionManagerExt> ModelContext for T {}

pub mod ext {
    pub use super::*;
    pub use infection_manager::InfectionManagerExt;
    pub use parameters::ParametersExt;
    pub use transmission_manager::TransmissionManagerExt;
    pub trait ModelContext {}
    impl<T: PluginContext + ParametersExt + TransmissionManagerExt + InfectionManagerExt>
        ModelContext for T
    {
    }
}

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
