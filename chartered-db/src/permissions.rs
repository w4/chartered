use bitflags::bitflags;
use option_set::{option_set, OptionSet};

option_set! {
    #[derive(FromSqlRow, AsExpression)]
    pub struct UserPermission: Identity + i32 {
        const VISIBLE         = 0b0000_0000_0000_0000_0000_0000_0000_0001;
        const PUBLISH_VERSION = 0b0000_0000_0000_0000_0000_0000_0000_0010;
        const YANK_VERSION    = 0b0000_0000_0000_0000_0000_0000_0000_0100;
        const MANAGE_USERS    = 0b0000_0000_0000_0000_0000_0000_0000_1000;
        const CREATE_CRATE    = 0b0000_0000_0000_0000_0000_0000_0001_0000;
    }
}

impl UserPermission {
    #[must_use]
    pub fn names() -> &'static [&'static str] {
        Self::NAMES
    }
}

impl<B: diesel::backend::Backend> diesel::deserialize::FromSql<diesel::sql_types::Integer, B>
    for UserPermission
where
    i32: diesel::deserialize::FromSql<diesel::sql_types::Integer, B>,
{
    fn from_sql(
        bytes: Option<&B::RawValue>,
    ) -> std::result::Result<UserPermission, Box<dyn std::error::Error + Send + Sync>> {
        let val = i32::from_sql(bytes)?;
        Ok(UserPermission::from_bits_truncate(val))
    }
}
