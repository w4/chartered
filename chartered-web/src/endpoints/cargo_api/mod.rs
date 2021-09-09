macro_rules! get_crate {
    ($db:expr, $name:expr; || -> $error:expr) => {
        Crate::find_by_name($db.clone(), $name)
            .await?
            .ok_or($error)
            .map(std::sync::Arc::new)?
    };
}

macro_rules! ensure_has_crate_perm {
    ($db:expr, $user:expr, $crate_expr:expr, $permissions:expr; || -> $error:expr) => {{
        if !$user
            .has_crate_permission($db.clone(), $crate_expr.id, $permissions)
            .await?
        {
            return Err($error);
        }
    }};
}

mod download;
mod owners;
mod publish;

pub use download::handle as download;
pub use owners::handle_get as get_owners;
pub use publish::handle as publish;
