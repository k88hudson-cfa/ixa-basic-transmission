use ixa::{impl_property, preludev2::*};
use serde::{Deserialize, Serialize};
use anyhow::Result;
define_entity!(Person);
define_entity!(Group);

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct InfectionData {
    pub infection_time: Option<f64>,
    pub infected_by: Option<PersonId>,
    pub recovery_time: Option<f64>,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
pub enum InfectionStatus {
    Susceptible, 
    Infectious(InfectionData),  
    Recovered(InfectionData),  
}

#[allow(dead_code)]
impl InfectionStatus {
    pub fn is_susceptible(&self) -> bool {
        self == &InfectionStatus::Susceptible
    }
    pub fn is_incidence(&self) -> bool {
        self.is_infectious() && self.infection_time().is_some()
    }
    pub fn infection_time(&self) -> Option<f64> {
        match self {
            InfectionStatus::Infectious(InfectionData { infection_time, .. }) => *infection_time,
            InfectionStatus::Recovered(InfectionData { infection_time, .. }) => *infection_time,
            InfectionStatus::Susceptible => None,
        }
    }
    pub fn infected_by(&self) -> Option<PersonId> {
        match self {
            InfectionStatus::Infectious(InfectionData { infected_by, .. }) => *infected_by,
            InfectionStatus::Recovered(InfectionData { infected_by, .. }) => *infected_by,
            InfectionStatus::Susceptible => None,
        }
    }
    pub fn is_infectious(&self) -> bool {
        matches!(self, InfectionStatus::Infectious { .. })
    }
    pub fn is_recovered(&self) -> bool {
        matches!(self, InfectionStatus::Recovered { .. })
    }
    pub fn to_recovered(self, recovery_time: f64) -> Result<Self> {
        match self {
            InfectionStatus::Infectious(InfectionData {
                infection_time,
                infected_by,
                ..
            }) => Ok(InfectionStatus::Recovered(InfectionData {
                infection_time,
                infected_by,
                recovery_time: Some(recovery_time),
            })),
            InfectionStatus::Recovered { .. } => anyhow::bail!("Person is already recovered"),
            InfectionStatus::Susceptible => anyhow::bail!("Person is not infectious"),
        }
    }
}


impl_property!(
    InfectionStatus,
    Person,
    default_const = InfectionStatus::Susceptible
);

// impl_property!(
//     InfectionStatus,
//     Group,
//     default_const = InfectionStatus::Susceptible 
// );