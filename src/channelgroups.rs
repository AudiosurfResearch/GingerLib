use crate::{channels::Channel, parser::parse_file, errors::ParseError};
use std::fs;
use tracing::trace;
use uuid::Uuid;

#[derive(Debug)]
/// Struct representing a channel group file, which is used by the engine to store any kind of data.
/// It contains several "tags", which consist of a 4 character name and the data.
/// The actual file will contain a 4-byte-long number indicating after the name, but this is not stored in the struct.
/// Tags may also contain no data at all, which is the case for the A3DG tag, since it's used as the magic number.
pub struct ChannelGroup {
    pub engine_version: u32,
    pub guid: Uuid,
    pub name: String,
    pub channels: Vec<Channel>,
}

impl ChannelGroup {
    /// Reads a file from the specified path.
    ///
    /// **This works with compressed and protected files too,**
    /// in those cases it will automatically decompress it and remove the protection.
    ///
    /// # Example
    /// ```rust
    /// use gingerlib::channelgroups::ChannelGroup;
    ///
    /// let file = ChannelGroup::read_from_file("./test.cgr").unwrap();
    /// ```
    ///
    /// # Returns
    /// The loaded `ChannelGroup`.
    ///
    /// # Errors
    /// Returns an `std::io::Error` if the file could not be opened.
    pub fn read_from_file(path: &str) -> Result<Self, ParseError> {
        trace!("Opening file: {:?}", path);
        let buffer = fs::read(path).map_err(ParseError::IoError)?;

        parse_file(&buffer)
    }
}
