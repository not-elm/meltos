pub mod compression;
pub mod error;
pub mod serde;

pub mod fs;
pub mod hash;
pub mod wasm;
pub mod path;
pub mod data_size;

pub mod macros {
    pub use meltos_macros::*;
}

#[macro_export]
macro_rules! impl_string_new_type {
    ($name: ident) => {
        impl $name {
            pub fn to_string(&self) -> String {
                self.0.clone()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_fmt(std::format_args!("{}", self.0))
            }
        }

        impl std::ops::Deref for $name {
            type Target = String;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<'a> From<&'a str> for $name {
            fn from(value: &'a str) -> Self {
                Self(value.to_string())
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }
    };
}
