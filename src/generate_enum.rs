#[macro_export]
macro_rules! map_handlers {
    ($outer_enum_name:ident { $($module:ident { $($variant:ident => $handler:ident),* $(,)? }),* $(,)? }, $state_type:ty) => {
        // Generate the outer enum (module level)
        #[derive(Debug, Clone)]
        pub enum $outer_enum_name {
            $($module($module),)*
        }

        // Generate each inner enum (event level) and implement their handlers
        $(
            #[derive(Debug, Clone)]
            pub enum $module {
                $($variant,)*
            }

            // Adjusted to pass $module as the handler type H and $state_type as the state type S
            impl $crate::handler::EventHandler<$outer_enum_name, $state_type> for $module {
                fn handle(
                    &self,
                    app_state: &mut $state_type,
                    event: &$crate::streaming::Event,
                    transaction: &$crate::streaming::Transaction,
                    handler_registry: &mut $crate::handler::HandlerRegistry<$outer_enum_name, $state_type>
                ) {
                    match self {
                        $(
                            $module::$variant => {
                                let decoded = scrypto::prelude::scrypto_decode::<$variant>(&event.binary_sbor_data).expect("Should be able to decode");
                                let input = $crate::streaming::EventHandlerInput {
                                    app_state,
                                    event: &decoded,
                                    transaction,
                                    handler_registry,
                                };
                                $handler(input);
                            },
                        )*
                    }
                }

                fn match_variant(&self, name: &str) -> bool {
                    match self {
                        $(
                            $module::$variant => name == stringify!($variant),
                        )*
                    }
                }
            }
        )*

        // Implement the outer enum's methods for module-level handling
        // Here, we also need to pass the correct type parameters to EventHandler
        impl $crate::handler::EventHandler<$outer_enum_name, $state_type> for $outer_enum_name {
            fn handle(
                &self,
                app_state: &mut $state_type,
                event: &$crate::streaming::Event,
                transaction: &$crate::streaming::Transaction,
                handler_registry: &mut $crate::handler::HandlerRegistry<$outer_enum_name, $state_type>
            ) {
                match self {
                    $(
                        $outer_enum_name::$module(inner) => inner.handle(app_state, event, transaction, handler_registry),
                    )*
                }
            }

            fn match_variant(&self, name: &str) -> bool {
                match self {
                    $(
                        $outer_enum_name::$module(inner) => inner.match_variant(name),
                    )*
                }
            }
        }
    }
}
