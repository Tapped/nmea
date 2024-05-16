use nom::{
    bytes::complete::take_until,
    character::{complete::char},
    combinator::{map_res, opt},
    IResult,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::utils::{parse_float_num, parse_valid_status};
use crate::{Error, NmeaSentence, SentenceType};

/// RSA - Rudder sensor angle
///
/// Shaft or engine revolution rate and propeller pitch
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_rsa_rudder_sensor_angle>
///
/// ```text
///        1   2 3   4 5
///        |   | |   | |
/// $--RSA,x.x,A,x.x,A*hh<CR><LF>
/// ```
/// 1. Starboard (or single) rudder sensor, "-" means Turn To Port
/// 2. Status, A = valid, V = Invalid
/// 3. Port rudder sensor
/// 4. Status, A = valid, V = Invalid
/// 5. Checksum
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[derive(Debug, PartialEq)]
pub struct RsaData {
    /// Starboard (or single) rudder sensor value. Negative represents Turn To Port. The value is proportional to the angle, but not necessarily equal.
    pub starboard_rudder_sensor: Option<f32>,
    /// Status of starboard sensor
    pub starboard_rudder_valid: bool,

    /// Port ruder sensor value. Negative represents Turn To Port. The value is proportional to the angle, but not necessarily equal.
    pub port_rudder_sensor: Option<f32>,
    /// Status of port sensor
    pub port_rudder_valid: bool,
}

/// # Parse RSA message
pub fn parse_rsa(sentence: NmeaSentence) -> Result<RsaData, Error> {
    if sentence.message_id != SentenceType::RSA {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::RSA,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_rsa(sentence.data)?.1)
    }
}

fn do_parse_rsa(i: &str) -> IResult<&str, RsaData> {
    let (i, starboard_rudder_sensor) = opt(map_res(take_until(","), parse_float_num::<f32>))(i)?;
    let (i, _) = char(',')(i)?;

    let (i, starboard_rudder_valid) = parse_valid_status(i)?;
    let (i, _) = char(',')(i)?;

    let (i, port_rudder_sensor) = opt(map_res(take_until(","), parse_float_num::<f32>))(i)?;
    let (i, _) = char(',')(i)?;

    let (i, port_rudder_valid) = parse_valid_status(i)?;

    Ok((
        i,
        RsaData {
            starboard_rudder_sensor,
            starboard_rudder_valid,
            port_rudder_sensor,
            port_rudder_valid,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rsa_full() {
        let data = parse_rsa(NmeaSentence {
            talker_id: "II",
            message_id: SentenceType::RSA,
            data: "8.0,A,-2,A",
            checksum: 0x79,
        })
        .unwrap();

        println!("{:?}", data);
    }
}
