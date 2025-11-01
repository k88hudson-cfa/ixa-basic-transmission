use ixa::prelude::*;

use crate::ModelContext;

define_rng!(ContactRng);

pub trait ContactManagerExt: ModelContext {
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
}
impl<T> ContactManagerExt for T where T: ModelContext {}
