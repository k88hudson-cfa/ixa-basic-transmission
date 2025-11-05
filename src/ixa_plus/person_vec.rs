use ixa::prelude::*;

pub struct PersonVec<T>(Vec<Option<T>>);

impl<T> PersonVec<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }
    pub fn get(&self, person_id: PersonId) -> Option<&T> {
        self.0.get(person_id.index()).and_then(|v| v.as_ref())
    }
    pub fn get_mut(&mut self, person_id: PersonId) -> Option<&mut T> {
        self.0.get_mut(person_id.index()).and_then(|v| v.as_mut())
    }
    pub fn set(&mut self, person_id: PersonId, value: T) {
        let index = person_id.index();
        if self.0.len() <= index {
            self.0.resize_with(index + 1, || None);
        }
        self.0[index] = Some(value);
    }
}
