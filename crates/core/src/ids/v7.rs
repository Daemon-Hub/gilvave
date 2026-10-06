use serde::{Deserialize, Serialize};
use std::fmt::Display;
use uuid::{Error, Uuid};

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(pub Uuid);

        impl Default for $name {
            fn default() -> Self {
                Self(Uuid::now_v7())
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl TryFrom<String> for $name {
            type Error = Error;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                match Uuid::parse_str(&value) {
                    Ok(uuid) => Ok(Self(uuid)),
                    Err(e) => Err(e),
                }
            }
        }

        impl TryFrom<&str> for $name {
            type Error = Error;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                match Uuid::parse_str(value) {
                    Ok(uuid) => Ok(Self(uuid)),
                    Err(e) => Err(e),
                }
            }
        }

        impl AsRef<Uuid> for $name {
            fn as_ref(&self) -> &Uuid {
                &self.0
            }
        }

        impl From<Uuid> for $name {
            fn from(value: Uuid) -> Self {
                Self(value)
            }
        }
    };
}

id_type!(UserId);
id_type!(MessageId);
id_type!(RefreshTokenId);
