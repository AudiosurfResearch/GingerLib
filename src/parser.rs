use nom::multi::count;
use nom::AsBytes;
use nom::{combinator::map_res, number::complete::le_u32, sequence::tuple, IResult};
use core::slice::SlicePattern;
use std::io::Read;
use std::str;
use tracing::trace;
use uuid::Uuid;

use crate::{channelgroups::ChannelGroup, channels::Channel};

pub fn parse_file(input: &Vec<u8>) -> IResult<&[u8], ChannelGroup> {
    let (input, (tag_name, data)) = parse_tag(input)?;

    match tag_name.as_str() {
        "ACTF" => {
            let (input, (tag_name, data)) = parse_tag(input)?; // contains the GUID, we don't need it though
            let (input, (decompressed_data, unprotected_data)) =
                map_res(decompress, unprotect)(input.to_vec())?;

            let (input, header) = parse_group_header(unprotected_data)?;

            Ok((
                input,
                ChannelGroup {
                    engine_version: header.engine_version,
                    guid: header.guid,
                    name: String::new(),
                    channels: Vec::new(),
                },
            ))
        }
        "QVRS" => Ok((
            input,
            ChannelGroup {
                engine_version: 60,
                guid: Uuid::nil(),
                name: String::new(),
                channels: Vec::new(),
            },
        )),
        _ => Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

fn parse_tag(input: &Vec<u8>) -> IResult<&Vec<u8>, (String, &Vec<u8>)> {
    let (input, (tag_name, tag_size)) = tuple((nom::bytes::complete::take(4usize), le_u32))(&*input.as_slice())?;
    let tag_name = str::from_utf8(tag_name).unwrap().to_string();
    let (input, tag_data) = nom::bytes::complete::take(tag_size as usize)(&*input.as_slice())?;
    Ok((input, (tag_name, tag_data)))
}

pub fn decompress(input: &Vec<u8>) -> IResult<&Vec<u8>, &Vec<u8>> {
    count(parse_tag, 2)(&input)?; //skip two tags

    trace!("Decompressing channel group");
    let (input, (tag_name, compressed_data)) = parse_tag(&input)?; //ZICB
    if tag_name == "ZICB" {
        let mut data = Vec::new();
        let mut decoder = flate2::read::ZlibDecoder::new(compressed_data);
        decoder.read_to_end(&mut data).map_err(|e| {
            nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Fail)).into()
        })?;
        Ok((input, data.as_slice()))
    } else {
        Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Fail,
        )))
    }
}

pub fn unprotect(input: &[u8]) -> IResult<&[u8], &[u8]> {
    trace!("Unprotecting channel group");
    count(parse_tag, 4)(input)?; //skip four tags because we dont need them
    let (input, (tag_name, data)) = parse_tag(input)?;
    if tag_name == "NECB" {
        let mut data = data.to_vec();
        // Decrypt by XORing every byte with 4
        // chosen by fair dice roll. guaranteed to be random.
        for i in &mut data {
            *i ^= 4u8;
        }

        Ok((input, &data))
    } else {
        Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Fail,
        )))
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

    let (input, (tag_name, tag_data)) = parse_tag(input)?;
    if tag_name != "CGGG" {
        return Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }
    let (input, (tag_name, tag_data)) = parse_tag(input)?;
    let guid = Uuid::from_slice(tag_data).map_err(|_| {
        nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Tag))
    })?;

    count(parse_tag, 2)(input)?; //CGUC, CHCO

    let (input, (tag_name, tag_data)) = parse_tag(input)?;
    let tag_data_array: [u8; 4] = tag_data.try_into().map_err(|_| {
        nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Tag))
    })?;
    let (input, channel_count) = le_u32(tag_data)?;

    Ok((
        input,
        GroupHeader {
            engine_version,
            guid,
        },
    ))
}
