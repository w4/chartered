mod crud;
mod info;
mod list;
mod members;

pub use crud::handle_put as create;
pub use info::handle_get as info;
pub use list::handle_get as list;
pub use members::{
    handle_delete as delete_member, handle_patch as update_member, handle_put as insert_member,
};
