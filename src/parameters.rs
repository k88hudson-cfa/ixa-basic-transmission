use crate::ixa_plus::{define_parameters, distr::gamma::*};
use anyhow::{Result, bail};
use ixa::prelude::*;
use serde::{Deserialize, Serialize};

define_parameters! {
    pub struct Params {
        population_size: usize {
            default: 1000,
            validate(value) {
                if *value == 0 {
                    bail!("population_size must be greater than 0");
                }
            }
        },
        /// The proportion of initial people who are infectious when we seed the population.
        /// as a number between 0 and 1.
        p_initial_incidence: f64 {
            default: 0.1,
            validate(value) {
                if *value < 0.0 || *value > 1.0 {
                    bail!("initial_incidence must be between 0 and 1");
                }
            }
        },

        /// The proportion of people that are initially recovered (fully immune to disease)
        /// as a number between 0 and 1.
        p_initial_recovered: f64 {
            default: 0.0,
            validate(value) {
                if *value < 0.0 || *value > 1.0 {
                    bail!("initial_recovered must be between 0 and 1");
                }
            }
        },

        /// The maximum run time of the simulation; even if there are still infections
        /// scheduled to occur, the simulation will stop at this time.
        max_time: f64 {
            default: 100.0,
            validate(value) {
                if *value < 0.0 {
                    bail!("max_time must be non-negative");
                }
            }
        },

        /// The random seed for the simulation.
        seed: u64 {
            default: 42,
            validate(value) {
                if *value == 0 {
                    bail!("seed must be non-zero");
                }
            }
        },

        // The distribution of infection rates across the population
        infection_rate: Gamma {
            default: gamma!(shape = 2.0, rate = 1.0)?,
        },

        // The distribution of infection rates across the population
        infection_duration: Gamma {
            default: gamma!(shape = 0.5, rate = 2.0)?,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_rate_distributions() {
        let params = Params::default();
        let mut rng = rand::rng();
        params.infection_duration.sample(&mut rng);
        assert_eq!(params.infection_duration.rate(), 2.0);
        assert_eq!(params.infection_duration.scale(), 1.0 / 2.0);
    }
}
