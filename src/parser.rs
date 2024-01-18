use nom::bytes::complete::take;
use nom::multi::count;
use nom::IResult;
use std::io::{Cursor, Read};
use std::str;
use tracing::trace;

use crate::channelgroups::{ChannelGroup, Tag};
use crate::errors::ParseError;

pub fn parse_file(input: &[u8]) -> Result<ChannelGroup, ParseError> {
    trace!("Parsing file");
    let (input, (tag_name, _)) = parse_tag(input).map_err(|_| ParseError::NomError)?;

    match tag_name.as_str() {
        "ACTF" => {
            let decompressed_data = decompress(input).map_err(|_| ParseError::NomError)?;
            let unprotected_data =
                unprotect(&decompressed_data).map_err(|_| ParseError::NomError)?;

            let mut tags = Vec::new();
            // Parse the tags while there's still data left
            let mut input = unprotected_data.as_slice();
            while !input.is_empty() {
                let (input2, (tag_name, tag_data)) =
                    parse_tag(input).map_err(|_| ParseError::NomError)?;
                tags.push(Tag {
                    name: tag_name,
                    data: tag_data.to_vec(),
                });
                input = input2; // Update the input to the remaining data
            }

            Ok(ChannelGroup { tags })
        }
        "QVRS" => {
            let mut tags = Vec::new();
            // Parse the tags while there's still data left
            let mut input = input;
            while !input.is_empty() {
                let (input2, (tag_name, tag_data)) =
                    parse_tag(input).map_err(|_| ParseError::NomError)?;
                tags.push(Tag {
                    name: tag_name,
                    data: tag_data.to_vec(),
                });
                input = input2; // Update the input to the remaining data
            }

            Ok(ChannelGroup { tags })
        }
        _ => Err(ParseError::InvalidFileType),
    }
}

fn parse_tag(input: &[u8]) -> IResult<&[u8], (String, &[u8])> {
    let (input, tag_name) = take(4usize)(input)?; //skip tag name
    let tag_name = str::from_utf8(tag_name).unwrap().to_string();
    trace!("Parsing: {}", tag_name);

    let (input2, tag_size) = take(4usize)(input)?;
    // if the tag size is all valid ASCII characters, it's a tag with no length indicator
    if tag_size.iter().all(|&c| c.is_ascii_uppercase()) {
        return Ok((input, (tag_name, [0u8; 0].as_ref())));
    }
    let tag_size = u32::from_le_bytes(tag_size.try_into().unwrap());

    let (input, tag_data) = nom::bytes::complete::take(tag_size as usize)(input2)?;
    Ok((input, (tag_name, tag_data)))
}

pub fn decompress(input: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + '_>> {
    trace!("Decompressing channel group");
    let (input, _) = count(parse_tag, 2)(input)?; //skip two tags

    let (_, (tag_name, compressed_data)) = parse_tag(input)?; //ZICB
    if tag_name == "ZICB" {
        trace!("Decompressing!");
        let compressed_stream = Cursor::new(compressed_data);
        let mut data = Vec::new();
        let mut decoder = flate2::read::ZlibDecoder::new(compressed_stream);
        decoder.read_to_end(&mut data)?;
        Ok(data)
    } else {
        Err(Box::from("Invalid file type"))
    }
}

pub fn unprotect(input: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + '_>> {
    trace!("Unprotecting channel group");
    let (input2, _) = count(parse_tag, 4)(input)?; //skip four tags because we dont need them

    let (_, (tag_name, data)) = parse_tag(input2)?;
    if tag_name == "NECB" {
        let mut data = data.to_vec();
        // Decrypt by XORing every byte with 4
        // chosen by fair dice roll. guaranteed to be random.
        for i in &mut data {
            *i ^= 4u8;
        }

        Ok(data)
    } else {
        Ok(input.to_vec()) // if it's not protected, just return the input
    }
}
