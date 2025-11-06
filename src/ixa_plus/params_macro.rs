use crate::ixa_plus::log;

#[macro_export]
macro_rules! define_parameters {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_name:ident : $field_type:ty $({
                    $( default : $default_value:expr, )?
                    $( validate($validate_arg:ident) $validate_body:block )?
                })?
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Serialize, Deserialize)]
        pub struct $name {
            $(
                $(#[$field_meta])*
                pub $field_name: $field_type,
            )*
        }

        impl $crate::ixa_plus::params_macro::IxaParameters for $name {}

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                let json = toml::to_string(&self).unwrap();
                write!(f, "{}", json)
            }
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
                    #[allow(dead_code)]
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
                    #[allow(unreachable_code)]
                    fn [<build_ $field_name>](value: Option<$field_type>) -> anyhow::Result<$field_type> {
                        if let Some(value) = value {
                            Self::[<validate_ $field_name>](&value)
                                .map_err(|e| ixa::IxaError::IxaError(
                                    concat!("Validation failed for parameter ", stringify!($field_name), ": ",)
                                        .to_string() + &e.to_string()
                                ))?;

                            return Ok(value);
                        }

                        $($(
                            return Ok($default_value);
                        )?)?

                        anyhow::bail!(concat!("Missing value for parameter ", stringify!($field_name)).to_string())
                    }
                )*

                #[inline]
                pub fn build(self) -> Result<$name, $crate::IxaError> {
                    Ok($name {
                        $(
                            $field_name: Self::[<build_ $field_name>](self.$field_name).map_err(|e| $crate::IxaError::IxaError(e.to_string()))?,
                        )*
                    })
                }
            }


            impl Default for $name {
                fn default() -> Self {
                    let builder = [< $name Builder >]::new();
                    builder.build().unwrap()
                }
            }

            ixa::define_global_property!(GlobalParams, $name);

            pub trait ParametersExt: ixa::PluginContext {
                fn use_default_params(&mut self) -> &Params {
                    self.set_global_property_value(GlobalParams, $name::default()).expect("Failed to set GlobalParams to default");
                    self.params()
                }
                fn set_params(&mut self, params: $name) -> &Params {
                    self.set_global_property_value(GlobalParams, params).expect("Failed to set GlobalParams");
                    self.params()
                }
                fn params(&self) -> &Params {
                    self.get_global_property_value(GlobalParams)
                        .expect("Expected GlobalParams to be set")
                }
                $(
                    #[inline]
                    #[allow(dead_code)]
                    fn [<param_ $field_name>](&self) -> &$field_type {
                        &self.params().$field_name
                    }
                )*
            }
            impl<C> ParametersExt for C where C: PluginContext {}
        }
    };
}

pub trait IxaParameters: Sized + serde::Serialize + serde::de::DeserializeOwned {
    fn from_args() -> Option<Self> {
        let args = std::env::args().collect::<Vec<_>>();
        // File --params <path>
        let mut prev_arg: Option<&str> = None;
        for arg in &args[1..] {
            if let Some(prev) = prev_arg {
                if prev == "--params" {
                    return Some(
                        Self::try_from_file(arg).expect("Could not parse parameters from file"),
                    );
                }
            }
            prev_arg = Some(arg);
        }
        None
    }
    // Parse parameters from toml or json
    fn try_from_file<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("{}: {}", path.as_ref().display(), e))?;
        let params: Self = if path.as_ref().extension().and_then(|s| s.to_str()) == Some("json") {
            serde_json::from_str(&contents)?
        } else if path.as_ref().extension().and_then(|s| s.to_str()) == Some("toml") {
            log::info!("Loading parameters from file {}", path.as_ref().display());
            toml::from_str(&contents)?
        } else {
            anyhow::bail!("Unsupported config file format. Use .toml or .json");
        };
        Ok(params)
    }
}
