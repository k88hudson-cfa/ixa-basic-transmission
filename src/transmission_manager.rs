use crate::ext::*;
use ixa::prelude::*;

define_rng!(ContactRng);
define_rng!(TransmissionRng);

pub trait TransmissionManagerExt: PluginContext {
    fn get_next_contact(&self, person_id: PersonId) -> Option<PersonId> {
        let mut contact_id = None;
        // Ensure we don't return the same id
        let total_people = self.get_current_population();
        if total_people > 1 {
            loop {
                contact_id = self.sample_person(ContactRng, ());
                if contact_id != Some(person_id) {
                    break;
                }
            }
        }
        contact_id
    }
    // Infection attempt function for a context and given `PersonId`
    fn infection_attempt(&mut self, infected: PersonId) -> Option<PersonId> {
        // Get a contact
        let next_contact = self.get_next_contact(infected)?;

        // if the person is not susceptible, fail the attempt.
        if !self.is_susceptible(next_contact) {
            return None;
        }

        // Reject based on relative transmission modifiers
        if !self.sample_bool(
            TransmissionRng,
            self.get_relative_total_transmission(infected, next_contact),
        ) {
            // If the rejection sample fails, return None
            return None;
        }

        // Infection succeeds
        self.set_infection_status(next_contact, infected);
        // Return the ID of the newly infected person
        Some(next_contact)
    }
    fn get_relative_total_transmission(&self, infected: PersonId, infectee: PersonId) -> f64 {
        1.0
    }
}
