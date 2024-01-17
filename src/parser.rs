use nom::multi::count;
use nom::{number::complete::le_u32, sequence::tuple, IResult};
use std::io::{Cursor, Read};
use std::str;
use tracing::trace;
use uuid::Uuid;

use crate::errors::ParseError;
use crate::{channelgroups::ChannelGroup, channels::Channel};

pub fn parse_file(input: &[u8]) -> Result<ChannelGroup, ParseError> {
    let (input, (tag_name, _)) = parse_tag(input).map_err(|_| ParseError::NomError)?;

    match tag_name.as_str() {
        "ACTF" => {
            let decompressed_data = decompress(input).map_err(|_| ParseError::NomError)?;
            let unprotected_data =
                unprotect(&decompressed_data).map_err(|_| ParseError::NomError)?;

            let (_, header) = parse_group_header(unprotected_data.as_slice())
                .map_err(|_| ParseError::NomError)?;

            Ok(ChannelGroup {
                engine_version: header.engine_version,
                guid: header.guid,
                name: String::new(),
                channels: Vec::new(),
            })
        }
        "QVRS" => Ok(ChannelGroup {
            engine_version: 60,
            guid: Uuid::nil(),
            name: String::new(),
            channels: Vec::new(),
        }),
        _ => Err(ParseError::InvalidFileType),
    }
}

fn parse_tag(input: &[u8]) -> IResult<&[u8], (String, &[u8])> {
    let (input, (tag_name, tag_size)) = tuple((nom::bytes::complete::take(4usize), le_u32))(input)?;
    let tag_name = str::from_utf8(tag_name).unwrap().to_string();
    let (input, tag_data) = nom::bytes::complete::take(tag_size as usize)(input)?;
    Ok((input, (tag_name, tag_data)))
}

pub fn decompress(input: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + '_>> {
    count(parse_tag, 2)(input)?; //skip two tags

    trace!("Decompressing channel group");
    let (_, (tag_name, compressed_data)) = parse_tag(input)?; //ZICB
    if tag_name == "ZICB" {
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
    count(parse_tag, 4)(input)?; //skip four tags because we dont need them
    let (_, (tag_name, data)) = parse_tag(input)?;
    if tag_name == "NECB" {
        let mut data = data.to_vec();
        // Decrypt by XORing every byte with 4
        // chosen by fair dice roll. guaranteed to be random.
        for i in &mut data {
            *i ^= 4u8;
        }

        Ok(data)
    } else {
        Err(Box::from("Invalid file type"))
    }
}

#[derive(Debug)]
pub struct GroupHeader {
    pub engine_version: u32,
    pub guid: Uuid,
}

pub fn parse_group_header(input: &[u8]) -> IResult<&[u8], GroupHeader> {
    let (input, (tag_name, tag_data)) = parse_tag(input)?;
    if tag_name != "QVRS" {
        return Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }
    let (input, engine_version) = le_u32(tag_data)?;

    let (input, (tag_name, tag_data)) = parse_tag(input)?;
    if tag_name != "A3DG" {
        return Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    let (input, (tag_name, _)) = parse_tag(input)?;
    if tag_name != "CGGG" {
        return Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }
    let (input, (_, tag_data)) = parse_tag(input)?;
    let guid = Uuid::from_slice(tag_data).map_err(|_| {
        nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Tag))
    })?;

    count(parse_tag, 2)(input)?; //CGUC, CHCO

    let (input, (_, tag_data)) = parse_tag(input)?;
    let (input, channel_count) = le_u32(tag_data)?;

    Ok((
        input,
        GroupHeader {
            engine_version,
            guid,
        },
    ))
}
