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
