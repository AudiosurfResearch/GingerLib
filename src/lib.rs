#![warn(clippy::pedantic, clippy::nursery)]

pub mod channelgroups;
pub mod channels;
pub mod errors;
mod parser;

#[cfg(test)]
mod tests {
    use test_log::test;
    use tracing::trace;

    use crate::channelgroups::ChannelGroup;

    #[test]
    fn it_works() {
        let channelgroup = ChannelGroup::read_from_file("./test.cgr").unwrap();
        trace!("{channelgroup:?}");
    }
}
