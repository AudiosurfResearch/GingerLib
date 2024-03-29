use std::{
    fs::File,
    io::{self, Cursor, Read, Seek, SeekFrom, Write},
};

#[derive(Debug)]
pub struct Tag {
    name: String,
    data: Vec<u8>,
}

impl Tag {
    /// Reads tags from a stream
    /// 
    /// # Returns
    /// A ``Vec<Tag>`` containing all tags from the stream.
    /// 
    /// # Errors
    /// Returns an ``std::io::Error`` if the stream could not be read.
    /// Returns a ``std::string::FromUtf8Error`` if the tag name could not be converted to a string.
    pub fn from_stream<S>(stream: &mut S) -> Result<Vec<Tag>, Box<dyn std::error::Error>>
    where
        S: Read + Seek,
    {
        let mut tags: Vec<Tag> = Vec::new();

        // Seek to the end to get the length
        let pos = stream.stream_position()?;
        let len = stream.seek(SeekFrom::End(0))?;

        // Seek back to the original position
        stream.seek(SeekFrom::Start(pos))?;

        while stream.stream_position()? < len {
            let mut name = [0u8; 4];
            stream.read_exact(&mut name)?;
            let name = String::from_utf8(name.to_vec())?;
            // The A3DG tag is not followed by any data. It's the magic number.
            // Curiously, it's not the first tag in the file, it's preceded by QVRS which denotes the engine version.
            if name == "A3DG" {
                tags.push(Tag::new(name, vec![0; 0]));
                continue;
            }

            let mut size = [0u8; 4];
            stream.read_exact(&mut size)?;
            let size = u32::from_le_bytes(size);

            let mut data = vec![0; size as usize];
            stream.read_exact(&mut data)?;

            tags.push(Tag::new(name, data));
        }
        Ok(tags)
    }

    pub fn new(name: String, data: Vec<u8>) -> Self {
        Self { name, data }
    }
}

#[derive(Debug)]
/// Struct representing a channel group file, which is used by the Quest3D engine to store any kind of data.
/// It contains several "tags", which consist of a 4 character name and the data.
/// The actual file will contain a 4-byte-long number indicating after the name, but this is not stored in the struct.
/// Tags may also contain no data at all, which is the case for the A3DG tag, since it's used as the magic number.
pub struct Quest3DFile {
    pub tags: Vec<Tag>,
}

impl Quest3DFile {
    /// Reads a file from the specified path.
    ///
    /// **This works with compressed and protected files too,**
    /// in those cases it will automatically decompress it and remove the protection.
    ///
    /// # Example
    /// ```rust
    /// use gingerlib::Quest3DFile;
    ///
    /// let file = Quest3DFile::read("./test.cgr").unwrap();
    /// assert_eq!(file.tags.len(), 150);
    /// ```
    ///
    /// # Returns
    /// The loaded `Quest3DFile`.
    ///
    /// # Errors
    /// Returns an `std::io::Error` if the file could not be opened.
    pub fn read(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let tags = Tag::from_stream(&mut file)?;

        //Check if file is compressed
        if tags[3].name == "ZICB" {
            //Decompress!
            let mut data = Vec::new();
            let mut decoder = flate2::read::ZlibDecoder::new(tags[3].data.as_slice());
            decoder.read_to_end(&mut data)?;
            let mut data = Cursor::new(data);

            let tags = Tag::from_stream(&mut data)?;

            //Check if file is protected
            if tags[4].name == "NECB" {
                let mut data = tags[4].data.clone();
                //Decrypt by XORing every byte with 4
                //chosen by fair dice roll. guaranteed to be random.
                for i in &mut data {
                    *i ^= 4u8;
                }
                let mut data = Cursor::new(data);

                let tags = Tag::from_stream(&mut data)?;
                return Ok(Self { tags });
            }

            return Ok(Self { tags });
        }

        Ok(Self { tags })
    }

    /// Converts the tags of the file to a byte vector.
    ///
    /// # Returns
    /// A byte vector containing the tags.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();

        for tag in &self.tags {
            data.extend(tag.name.as_bytes());
            data.extend(&(tag.data.len() as u32).to_le_bytes());
            data.extend(&tag.data);
        }

        data
    }

    /// Saves the file to the specified path.
    ///
    /// # Returns
    /// The `File`.
    ///
    /// # Errors
    /// Returns an `std::io::Error` if the file could not be created.
    pub fn save_to_file(&self, path: &str) -> Result<File, io::Error> {
        let mut file = File::create(path)?;

        for tag in &self.tags {
            file.write_all(tag.name.as_bytes())?;
            //as stated previously, the A3DG tag is just the magic number
            if tag.name == "A3DG" {
                continue;
            }
            file.write_all(&(tag.data.len() as u32).to_le_bytes())?;
            file.write_all(&tag.data)?;
        }

        Ok(file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, path::Path};

    #[test]
    fn test_load() {
        let file = Quest3DFile::read(
            Path::new(&env::var("AUDIOSURF_ENGINE_DIR").unwrap())
                .join("progress calculator.cgr")
                .to_str()
                .unwrap(),
        )
        .unwrap();

        //test if all tags are read
        assert_eq!(file.tags.len(), 150);
    }

    #[test]
    fn test_save() {
        let file = Quest3DFile::read(
            Path::new(&env::var("AUDIOSURF_ENGINE_DIR").unwrap())
                .join("progress calculator.cgr")
                .to_str()
                .unwrap(),
        )
        .unwrap();

        file.save_to_file("./test.cgr").unwrap();
    }
}
