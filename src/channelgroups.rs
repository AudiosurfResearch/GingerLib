use crate::{errors::ParseError, parser::parse_file};
use std::{fs, io};
use tracing::trace;

#[derive(Debug)]
pub struct Tag {
    pub name: String,
    pub data: Vec<u8>,
}


#[derive(Debug)]
/// Struct representing a channel group file, which is used by the engine to store any kind of data.
/// It contains several "tags", which consist of a 4 character name and the data.
/// The actual file will contain a 4-byte-long number indicating after the name, but this is not stored in the struct.
/// Tags may also contain no data at all, which is the case for the A3DG tag, since it's used as the magic number.
pub struct ChannelGroup {
    pub tags: Vec<Tag>,
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

    /// Saves the `ChannelGroup` to a file at the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to save the file.
    ///
    /// # Errors
    ///
    /// Returns an `std::io::Error` if something goes wrong.
    pub fn save_to_file(&self, path: &str) -> Result<(), io::Error> {
        trace!("Saving file to: {:?}", path);
        let mut buffer = Vec::new();

        for tag in &self.tags {
            trace!("Writing tag: {}", tag.name);
            buffer.extend(tag.name.as_bytes());
            if tag.data.is_empty() {
                continue;
            }
            //Quest3D is 32-bit!
            #[allow(clippy::cast_possible_truncation)]
            let length = tag.data.len() as u32;
            buffer.extend(length.to_le_bytes());
            buffer.extend(tag.data.clone());
        }

        fs::write(path, buffer)?;

        Ok(())
    }
}
