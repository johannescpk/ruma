//! Matrix identifiers for places where a room ID or room alias ID are used interchangeably.

use std::{
    convert::{TryFrom, TryInto},
    fmt,
    hint::unreachable_unchecked,
    num::NonZeroU8,
};

use crate::{server_name::ServerName, RoomAliasId, RoomId};

/// A Matrix room ID or a Matrix room alias ID.
///
/// `RoomIdOrAliasId` is useful for APIs that accept either kind of room identifier. It is converted
/// from a string slice, and can be converted back into a string as needed. When converted from a
/// string slice, the variant is determined by the leading sigil character.
///
/// ```
/// # use std::convert::TryFrom;
/// # use ruma_identifiers::RoomIdOrAliasId;
/// assert_eq!(
///     RoomIdOrAliasId::try_from("#ruma:example.com").unwrap().as_ref(),
///     "#ruma:example.com"
/// );
///
/// assert_eq!(
///     RoomIdOrAliasId::try_from("!n8f893n9:example.com").unwrap().as_ref(),
///     "!n8f893n9:example.com"
/// );
/// ```
#[derive(Clone)]
pub struct RoomIdOrAliasId {
    full_id: Box<str>,
    colon_idx: NonZeroU8,
}

impl fmt::Debug for RoomIdOrAliasId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.full_id.fmt(f)
    }
}

impl RoomIdOrAliasId {
    /// Returns the local part (everything after the `!` or `#` and before the first colon).
    pub fn localpart(&self) -> &str {
        &self.full_id[1..self.colon_idx.get() as usize]
    }

    /// Returns the server name of the room (alias) ID.
    pub fn server_name(&self) -> &ServerName {
        self.full_id[self.colon_idx.get() as usize + 1..].try_into().unwrap()
    }

    /// Whether this is a room id (starts with `'!'`)
    pub fn is_room_id(&self) -> bool {
        self.variant() == Variant::RoomId
    }

    /// Whether this is a room alias id (starts with `'#'`)
    pub fn is_room_alias_id(&self) -> bool {
        self.variant() == Variant::RoomAliasId
    }

    /// Turn this `RoomIdOrAliasId` into `Either<RoomId, RoomAliasId>`
    #[cfg(feature = "either")]
    pub fn into_either(self) -> either::Either<Box<RoomId>, Box<RoomAliasId>> {
        match self.variant() {
            Variant::RoomId => either::Either::Left(self.as_str().try_into().unwrap()),
            Variant::RoomAliasId => either::Either::Right(self.as_str().try_into().unwrap()),
        }
    }

    fn variant(&self) -> Variant {
        match self.full_id.bytes().next() {
            Some(b'!') => Variant::RoomId,
            Some(b'#') => Variant::RoomAliasId,
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

#[derive(PartialEq)]
enum Variant {
    RoomId,
    RoomAliasId,
}

/// Attempts to create a new Matrix room ID or a room alias ID from a string representation.
///
/// The string must either include the leading ! sigil, the localpart, a literal colon, and a
/// valid homeserver host or include the leading # sigil, the alias, a literal colon, and a
/// valid homeserver host.
fn try_from<S>(room_id_or_alias_id: S) -> Result<RoomIdOrAliasId, crate::Error>
where
    S: AsRef<str> + Into<Box<str>>,
{
    let colon_idx =
        ruma_identifiers_validation::room_id_or_alias_id::validate(room_id_or_alias_id.as_ref())?;
    Ok(RoomIdOrAliasId { full_id: room_id_or_alias_id.into(), colon_idx })
}

common_impls!(RoomIdOrAliasId, try_from, "a Matrix room ID or room alias ID");

impl From<Box<RoomId>> for RoomIdOrAliasId {
    fn from(room_id: Box<RoomId>) -> Self {
        Self::try_from(room_id.as_str()).unwrap()
    }
}

impl From<Box<RoomAliasId>> for RoomIdOrAliasId {
    fn from(room_alias_id: Box<RoomAliasId>) -> Self {
        Self::try_from(room_alias_id.as_str()).unwrap()
    }
}

impl TryFrom<RoomIdOrAliasId> for Box<RoomId> {
    type Error = Box<RoomAliasId>;

    fn try_from(id: RoomIdOrAliasId) -> Result<Box<RoomId>, Box<RoomAliasId>> {
        match id.variant() {
            Variant::RoomId => Ok(id.as_str().try_into().unwrap()),
            Variant::RoomAliasId => Err(id.as_str().try_into().unwrap()),
        }
    }
}

impl TryFrom<RoomIdOrAliasId> for Box<RoomAliasId> {
    type Error = Box<RoomId>;

    fn try_from(id: RoomIdOrAliasId) -> Result<Box<RoomAliasId>, Box<RoomId>> {
        match id.variant() {
            Variant::RoomAliasId => Ok(id.as_str().try_into().unwrap()),
            Variant::RoomId => Err(id.as_str().try_into().unwrap()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use super::RoomIdOrAliasId;
    use crate::Error;

    #[test]
    fn valid_room_id_or_alias_id_with_a_room_alias_id() {
        assert_eq!(
            RoomIdOrAliasId::try_from("#ruma:example.com")
                .expect("Failed to create RoomAliasId.")
                .as_ref(),
            "#ruma:example.com"
        );
    }

    #[test]
    fn valid_room_id_or_alias_id_with_a_room_id() {
        assert_eq!(
            RoomIdOrAliasId::try_from("!29fhd83h92h0:example.com")
                .expect("Failed to create RoomId.")
                .as_ref(),
            "!29fhd83h92h0:example.com"
        );
    }

    #[test]
    fn missing_sigil_for_room_id_or_alias_id() {
        assert_eq!(
            RoomIdOrAliasId::try_from("ruma:example.com").unwrap_err(),
            Error::MissingLeadingSigil
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_valid_room_id_or_alias_id_with_a_room_alias_id() {
        assert_eq!(
            serde_json::to_string(
                &RoomIdOrAliasId::try_from("#ruma:example.com")
                    .expect("Failed to create RoomAliasId.")
            )
            .expect("Failed to convert RoomAliasId to JSON."),
            r##""#ruma:example.com""##
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_valid_room_id_or_alias_id_with_a_room_id() {
        assert_eq!(
            serde_json::to_string(
                &RoomIdOrAliasId::try_from("!29fhd83h92h0:example.com")
                    .expect("Failed to create RoomId.")
            )
            .expect("Failed to convert RoomId to JSON."),
            r#""!29fhd83h92h0:example.com""#
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_valid_room_id_or_alias_id_with_a_room_alias_id() {
        assert_eq!(
            serde_json::from_str::<RoomIdOrAliasId>(r##""#ruma:example.com""##)
                .expect("Failed to convert JSON to RoomAliasId"),
            RoomIdOrAliasId::try_from("#ruma:example.com").expect("Failed to create RoomAliasId.")
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_valid_room_id_or_alias_id_with_a_room_id() {
        assert_eq!(
            serde_json::from_str::<RoomIdOrAliasId>(r##""!29fhd83h92h0:example.com""##)
                .expect("Failed to convert JSON to RoomId"),
            RoomIdOrAliasId::try_from("!29fhd83h92h0:example.com")
                .expect("Failed to create RoomAliasId.")
        );
    }
}
