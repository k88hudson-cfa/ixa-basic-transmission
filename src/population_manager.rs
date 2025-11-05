use std::fmt::Display;

use crate::format_iter;
use anyhow::Result;
use ixa::log;
use ixa::prelude::*;
use rand_distr::weighted::WeightedIndex;

define_rng!(PopulationRng);

type AssignFn<C> = fn(&mut C, PersonId) -> Result<()>;

pub trait PopulationManagerExt: PluginContext {
    fn seed_weighted_population<T: Display>(
        &mut self,
        population_size: usize,
        proportions: Vec<(f64, (T, AssignFn<Self>))>,
    ) -> Result<usize> {
        log::info!(
            "\nSeeding population of size {} with proportions:\n{}",
            population_size,
            format_iter!(proportions, |(weight, (name, _))| "{name}={weight:.2}"),
        );

        let (weights, assign_fns): (Vec<_>, Vec<_>) = proportions.into_iter().unzip();

        // Validate total weight
        let total_weight: f64 = weights.iter().sum();
        if total_weight > 1.0 {
            anyhow::bail!("Proportions must sum to 1.0 or less (got {total_weight:.3})");
        }

        let dist = WeightedIndex::new(&weights)?;

        for _ in 0..population_size {
            let person_id = self.add_person(())?;
            let (_, apply) = assign_fns[self.sample_distr(PopulationRng, &dist)];
            apply(self, person_id)?;
        }

        Ok(population_size)
    }
}

impl<C> PopulationManagerExt for C where C: PluginContext {}
