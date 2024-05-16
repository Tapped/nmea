use nom::{
    bytes::complete::take_until,
    character::{complete::char, streaming::one_of},
    combinator::{map_res, opt},
    IResult,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::utils::{parse_float_num, parse_num, parse_valid_status};
use crate::{Error, NmeaSentence, SentenceType};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpmSource {
    Shaft,
    Engine,
}

/// RPM - Revolution
///
/// Shaft or engine revolution rate and propeller pitch
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_rpm_revolutions>
///
/// ```text
/// 1 2 3   4   5 6
/// | | |   |   | |
/// $--RPM,a,x,x.x,x.x,A*hh<CR><LF>
/// ```
/// 1. Source, S = Shaft, E = Engine
/// 2. Engine or shaft number
/// 3. Speed, Revolutions per minute
/// 4. Propeller pitch, % of maximum, "-" means astern
/// 5. Status, A = Valid, V = Invalid
/// 6. Checksum
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[derive(Debug, PartialEq)]
pub struct RpmData {
    /// Source of reading
    pub source: Option<RpmSource>,

    /// Engine or shaft number, numbered from centre-line
    /// odd = starboard, even = port
    /// 0 = single or on centre-line
    pub engine_or_shaft_number: Option<i8>,

    /// Speed in revolutions / min.
    /// It rotates counter clockwise if the value is negative
    pub speed: Option<f32>,

    /// Propeller pitch in percentage of maximum. It can be negative if the pitch is towards astern.
    pub propeller_pitch: Option<f32>,

    pub valid: bool,
}

/// # Parse RPM message
pub fn parse_rpm(sentence: NmeaSentence) -> Result<RpmData, Error> {
    if sentence.message_id != SentenceType::RPM {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::RPM,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_rpm(sentence.data)?.1)
    }
}

fn do_parse_rpm(i: &str) -> IResult<&str, RpmData> {
    let (i, source_char) = opt(one_of("SE"))(i)?;
    let (i, _) = char(',')(i)?;
    let source = source_char.map(|char| match char {
        'S' => RpmSource::Shaft,
        'E' => RpmSource::Engine,
        _ => unreachable!(),
    });

    let (i, engine_or_shaft_number) = opt(map_res(take_until(","), parse_num::<i8>))(i)?;
    let (i, _) = char(',')(i)?;

    let (i, speed) = opt(map_res(take_until(","), parse_float_num::<f32>))(i)?;
    let (i, _) = char(',')(i)?;

    let (i, propeller_pitch) = opt(map_res(take_until(","), parse_float_num::<f32>))(i)?;
    let (i, _) = char(',')(i)?;

    let (i, valid) = parse_valid_status(i)?;
    Ok((
        i,
        RpmData {
            source,
            engine_or_shaft_number,
            speed,
            propeller_pitch,
            valid,
        },
    ))
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse_rpm_full() {
        let data = parse_rpm(NmeaSentence {
            talker_id: "II",
            message_id: SentenceType::RPM,
            data: "S,1,31,100,A",
            checksum: 0x73,
        })
        .unwrap();

        println!("{:?}", data);
    }
}
