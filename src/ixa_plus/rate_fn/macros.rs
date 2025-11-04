#[macro_export]
macro_rules! define_rate {
    (
        $name:ident {
            fn assign$(<$context_generic:ident: $context_trait:path>)?(&self, $context_ident:ident: &$c:ident, $($rest_assign:tt)*) ->  Self::RateFnInstance $body:block
            $( $impl_other_items:item )*
        }
    ) => {
        #[derive(Clone)]
        pub struct $name;

        impl$(<C: $context_trait>)? crate::ixa_plus::rate_fn::RateFnGenerator<C> for $name {

            type RateFnInstance = crate::ixa_plus::rate_fn::RateFn;
            fn name(&self) -> &'static str {
                stringify!($name)
            }
            fn assign(&self, $context_ident: &C, $($rest_assign)*) -> Self::RateFnInstance $body
            $( $impl_other_items )*
        }
    };
}
