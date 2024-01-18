#![warn(clippy::pedantic, clippy::nursery)]

pub mod channelgroups;
pub mod errors;
mod parser;

#[cfg(test)]
mod tests {
    use test_log::test;

    use crate::channelgroups::ChannelGroup;

    #[test]
    fn reading() {
        let _channelgroup = ChannelGroup::read_from_file("./samples/ASR_PedroCamacho_AudiosurfOverture.cgr").unwrap();
    }

    #[test]
    fn writing() {
        let channelgroup = ChannelGroup::read_from_file("./samples/ASR_PedroCamacho_AudiosurfOverture.cgr").unwrap();
        channelgroup.save_to_file("./test.cgr").unwrap();
    }
}
