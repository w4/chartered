//! [ls-refs][lsr] is sent from the client when they want to see what refs we have
//! on the server, we're generating our commits on the fly though so we'll just tell
//! them we have a master branch with whatever the generated commit hash is.
//!
//! [lsr]: https://git-scm.com/docs/protocol-v2/2.19.0#_ls_refs

use bytes::Bytes;
use thrussh::{server::Session, ChannelId};

use crate::{
    git::{packfile::low_level::HashOutput, PktLine},
    Handler,
};

pub(crate) async fn handle(
    handle: &mut Handler,
    session: &mut Session,
    channel: ChannelId,
    _metadata: Vec<Bytes>,
    commit_hash: &HashOutput,
) -> Result<(), anyhow::Error> {
    let commit_hash = hex::encode(&commit_hash);

    handle.write(PktLine::Data(
        format!("{} HEAD symref-target:refs/heads/master\n", commit_hash).as_bytes(),
    ))?;
    handle.write(PktLine::Flush)?;
    handle.flush(session, channel);

    Ok(())
}
