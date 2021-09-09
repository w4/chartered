macro_rules! ensure_has_crate_perm {
    ($db:expr, $user:expr, $crate_expr:expr, $($permission:path | -> $error:expr$(,)?),*) => {{
        let perms = $user.get_crate_permissions($db.clone(), $crate_expr.id).await?;

        $(
            if !perms.contains($permission) {
                return Err($error);
            }
        )*
    }};
}

mod download;
mod owners;
mod publish;

pub use download::handle as download;
pub use owners::handle_get as get_owners;
pub use publish::handle as publish;
