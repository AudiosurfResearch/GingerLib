use std::{
    fs::File,
    io::{Cursor, Read, Seek, SeekFrom},
};

#[derive(Debug)]
pub struct Tag {
    name: String,
    data: Vec<u8>,
}

impl Tag {
    pub fn new(name: String, data: Vec<u8>) -> Self {
        Self { name, data }
    }
}

#[derive(Debug)]
pub struct Quest3DFile {
    pub tags: Vec<Tag>,
}

/// Reads tags from a stream
fn read_tags<S>(stream: &mut S) -> Result<Vec<Tag>, Box<dyn std::error::Error>>
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

        let mut size = [0u8; 4];
        stream.read_exact(&mut size)?;
        let size = u32::from_le_bytes(size);

        let mut data = vec![0; size as usize];
        stream.read_exact(&mut data)?;

        tags.push(Tag::new(name, data));
    }
    Ok(tags)
}

impl Quest3DFile {
    pub fn read(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let tags = read_tags(&mut file)?;

        //Check if file is compressed
        if tags[3].name == "ZICB" {
            //Decompress!
            let mut data = Vec::new();
            let mut decoder = flate2::read::ZlibDecoder::new(tags[3].data.as_slice());
            decoder.read_to_end(&mut data)?;
            let mut data = Cursor::new(data);

            let tags = read_tags(&mut data)?;
            return Ok(Self { tags });
        }

        Ok(Self { tags })
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
        assert_eq!(file.tags.len(), 5);
        assert_eq!(file.tags[0].name, "ACTF");
        assert_eq!(file.tags[1].name, "NECL");
        assert_eq!(file.tags[2].name, "NECT");
        assert_eq!(file.tags[3].name, "NEOS");
        assert_eq!(file.tags[4].name, "NECB");
    }
}
