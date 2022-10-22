use bytes::Bytes;
use packfile::{
    low_level::{PackFile, PackFileEntry},
    PktLine,
};
use thrussh::{server::Session, ChannelId};

use crate::Handler;

pub(crate) fn handle(
    handle: &mut Handler,
    session: &mut Session,
    channel: ChannelId,
    metadata: Vec<Bytes>,
    packfile_entries: Vec<PackFileEntry>,
) -> Result<(), anyhow::Error> {
    // the client sending us `done` in the metadata means they know there's no negotiation
    // required for which commits we need to send, they just want us to send whatever we
    // have.
    let done = metadata.iter().any(|v| v.as_ref() == b"done");

    // the client thinks we can negotiate some commits with them, but we don't want to so
    // we'll just say we've got nothing in common and continue on as we were.
    if !done {
        handle.write(PktLine::Data(b"acknowledgments\n"))?;
        handle.write(PktLine::Data(b"ready\n"))?;
        handle.write(PktLine::Delimiter)?;
    }

    // magic header
    handle.write(PktLine::Data(b"packfile\n"))?;

    // send a welcome message
    handle.write(PktLine::SidebandMsg(b"Hello from chartered!\n"))?;
    handle.flush(session, channel);

    // send the complete packfile
    let packfile = PackFile::new(&packfile_entries);
    handle.write(PktLine::SidebandData(packfile))?;
    handle.write(PktLine::Flush)?;
    handle.flush(session, channel);

    // tell the client we exited successfully and close the channel
    session.exit_status_request(channel, 0);
    session.eof(channel);
    session.close(channel);

    Ok(())
}
