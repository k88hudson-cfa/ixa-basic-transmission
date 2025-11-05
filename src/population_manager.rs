use anyhow::Result;
use ixa::prelude::*;
use rand_distr::weighted::WeightedIndex;

define_rng!(PopulationRng);

pub trait PopulationManagerExt: PluginContext {
    fn seed_weighted_population(
        &mut self,
        population_size: usize,
        proportions: Vec<(f64, fn(&mut Self, PersonId) -> Result<()>)>,
    ) -> Result<usize> {
        let (weights, assign_fns): (Vec<_>, Vec<_>) = proportions.into_iter().unzip();

        // Validate total weight
        let total_weight: f64 = weights.iter().sum();
        if total_weight > 1.0 {
            anyhow::bail!("Proportions must sum to 1.0 or less (got {total_weight:.3})");
        }

        let dist = WeightedIndex::new(&weights)?;

        for _ in 0..population_size {
            let person_id = self.add_person(())?;
            let apply = assign_fns[self.sample_distr(PopulationRng, &dist)];
            apply(self, person_id)?;
        }
        Ok(population_size)
    }
}

impl<C> PopulationManagerExt for C where C: PluginContext {}
