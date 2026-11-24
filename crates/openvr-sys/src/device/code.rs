//! `SteamVR` C enums as exhaustive Rust enums, each variant tied to its raw value once.

/// Declares an enum over a C `int` enum, with `code` and `from_code` generated from one list.
macro_rules! c_enum {
    (
        $(#[$doc:meta])*
        $name:ident { $($(#[$variant_doc:meta])* $variant:ident = $code:literal,)* }
    ) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum $name {
            $($(#[$variant_doc])* $variant,)*
        }

        impl $name {
            /// The raw value `SteamVR` uses.
            pub const fn code(self) -> i32 {
                match self {
                    $(Self::$variant => $code,)*
                }
            }

            /// The variant for a raw value, or `None` for one this binding does not know.
            pub const fn from_code(code: i32) -> Option<Self> {
                match code {
                    $($code => Some(Self::$variant),)*
                    _ => None,
                }
            }
        }
    };
}

pub(crate) use c_enum;
