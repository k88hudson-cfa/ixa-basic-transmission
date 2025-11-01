use crate::ModelContext;
use crate::distr::gamma::*;
use anyhow::Result;
use ixa::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

macro_rules! define_parameters {
    (
        $(#[$meta:meta])*
        $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_name:ident : $field_type:ty $({
                    $( try_from : $from_type:ty, )?
                    $( default : $default_value:expr, )?
                    $( validate($validate_arg:ident) $validate_body:block )?
                })?
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        pub struct $name {
            $(
                $(#[$field_meta])*
                pub $field_name: $field_type,
            )*
        }

        paste::paste! {
            pub struct [<$name Builder>] {
                $(
                    $(#[$field_meta])*
                    $field_name: Option<$field_type>,
                )*
            }

            impl [<$name Builder>] {
                #[inline]
                pub fn new() -> Self {
                    Self {
                        $(
                            $field_name: None,
                        )*
                    }
                }


                $(
                    #[inline]
                    pub fn $field_name(mut self, value: $field_type) -> Self {
                        self.$field_name = Some(value);
                        self
                    }
                )*


                $(

                    #[inline]
                    fn [<validate_ $field_name>](_value: &$field_type) -> Result<()> {
                        $($(
                        let $validate_arg = _value;
                        $validate_body
                        )?)?
                        Ok(())
                    }

                )*


                $(
                    #[inline]
                    fn [<build_ $field_name>](value: Option<$field_type>) -> Result<$field_type, $crate::IxaError> {
                        if let Some(value) = value {
                            Self::[<validate_ $field_name>](&value)
                                .map_err(|e| $crate::IxaError::IxaError(
                                    concat!("Validation failed for parameter ", stringify!($field_name), ": ",)
                                        .to_string() + &e.to_string()
                                ))?;

                            return Ok(value);
                        }

                        $($(
                            return Ok($default_value);
                        )?)?

                        #[allow(unused)]
                        Err($crate::IxaError::IxaError(concat!(
                            "Missing required parameter: ",
                            stringify!($field_name)
                        ).into()))
                    }
                )*

                #[inline]
                pub fn build(self) -> Result<$name, $crate::IxaError> {
                    Ok($name {
                        $(
                            $field_name: Self::[<build_ $field_name>](self.$field_name)?,
                        )*
                    })
                }
            }

             define_global_property!(GlobalParams, $name);

            pub trait ParametersExt: ModelContext {
                fn params(&self) -> &Params {
                    self.get_global_property_value(GlobalParams)
                        .expect("Expected GlobalParams to be set")
                }
                $(
                    fn [<param_ $field_name>](&self) -> &$field_type {
                        &self.params().$field_name
                    }
                )*
            }
            impl<T> ParametersExt for T where T: ModelContext {}

        }


    };
}

define_parameters!(
    #[derive(Debug, Clone, Serialize, Deserialize)]
    Params {
        /// The proportion of initial people who are infectious when we seed the population.
        /// as a number between 0 and 1.
        initial_incidence: f64 {
            default: 1.0,
            validate(value) {
                if *value < 0.0 || *value > 1.0 {
                    anyhow::bail!("initial_incidence must be between 0 and 1");
                }
            }
        },

        /// The proportion of people that are initially recovered (fully immune to disease)
        /// as a number between 0 and 1.
        initial_recovered: f64 {
            default: 0.0,
            validate(value) {
                if *value < 0.0 || *value > 1.0 {
                    anyhow::bail!("initial_recovered must be between 0 and 1");
                }
            }
        },

        /// The maximum run time of the simulation; even if there are still infections
        /// scheduled to occur, the simulation will stop at this time.
        max_time: f64 {
            default: 100.0,
            validate(value) {
                if *value < 0.0 {
                    anyhow::bail!("max_time must be non-negative");
                }
            }
        },

        /// The random seed for the simulation.
        seed: u64 {
            default: 42,
            validate(value) {
                if *value == 0 {
                    anyhow::bail!("seed must be non-zero");
                }
            }
        },

        // The distribution of infection rates as a Gamma distribution
        infection_rate: Gamma {
            try_from: GammaParams,
            default: GammaParams::Rate { shape: 2.0, rate: 1.0 },
        },

        // The distribution of infection durations as a Gamma distribution
        infection_duration: Gamma {
            default: Gamma::from_shape_rate(5.0, 1.0).unwrap(),
        }
    }

);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RateFnType {
    /// A constant rate of infectiousness (constant hazard -> exponential waiting times) for a given
    /// duration.
    Constant { rate: f64, duration: f64 },
    /// A library of empirical rate functions read in from a file.
    EmpiricalFromFile {
        /// The path to the library of empirical rates with columns, `id`, `time`, and `value`.
        file: PathBuf,
        /// Empirical rate functions are specified as hazard rates. However, the specified hazard
        /// rates are relative rather than absolute (unlike the constant rate of infectiousness
        /// which has an absolute rate of infection). We need a scale factor (that is often
        /// calibrated) to convert the relative hazard rates to absolute rates of infection.
        scale: f64,
    },
}
