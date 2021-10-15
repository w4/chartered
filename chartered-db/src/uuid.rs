use diesel::sql_types::Binary;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::prelude::*;
pub use uuid::Uuid;

#[derive(Debug, Clone, Copy, FromSqlRow, AsExpression, Hash, Eq, PartialEq)]
#[sql_type = "Binary"]
pub struct SqlUuid(pub uuid::Uuid);

impl SqlUuid {
    #[must_use]
    pub fn random() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl From<SqlUuid> for uuid::Uuid {
    fn from(s: SqlUuid) -> Self {
        s.0
    }
}

impl Display for SqlUuid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<B: diesel::backend::Backend> diesel::deserialize::FromSql<Binary, B> for SqlUuid
where
    Vec<u8>: diesel::deserialize::FromSql<Binary, B>,
{
    fn from_sql(bytes: Option<&B::RawValue>) -> diesel::deserialize::Result<Self> {
        let value = <Vec<u8>>::from_sql(bytes)?;
        uuid::Uuid::from_slice(&value)
            .map(SqlUuid)
            .map_err(Into::into)
    }
}

impl<B: diesel::backend::Backend> diesel::serialize::ToSql<Binary, B> for SqlUuid
where
    [u8]: diesel::serialize::ToSql<Binary, B>,
{
    fn to_sql<W: Write>(
        &self,
        out: &mut diesel::serialize::Output<'_, W, B>,
    ) -> diesel::serialize::Result {
        out.write_all(self.0.as_bytes())
            .map(|_| diesel::serialize::IsNull::No)
            .map_err(Into::into)
    }
}
