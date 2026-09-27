use std::{borrow::Cow, fmt};

macro_rules! key_type {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        ///
        /// Conversion preserves the exact string. A key is validated when the
        /// model is built or a note is added, not during conversion.
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            /// Returns the original, unmodified key.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }

        impl From<&String> for $name {
            fn from(value: &String) -> Self {
                Self(value.clone())
            }
        }

        impl From<Cow<'_, str>> for $name {
            fn from(value: Cow<'_, str>) -> Self {
                Self(value.into_owned())
            }
        }

        impl From<&$name> for $name {
            fn from(value: &$name) -> Self {
                value.clone()
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

key_type!(
    FieldKey,
    "A stable field symbol, independent of its display name."
);
key_type!(
    TemplateKey,
    "A stable card-template symbol, independent of its display name."
);
