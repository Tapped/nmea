use nom::{character::complete::char, combinator::opt, IResult};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{Error, NmeaSentence, SentenceType};

use super::utils::parse_valid_status;

/// ROT - Rate of Turn
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_rot_rate_of_turn>
///
/// ```text
///         1   2 3
///         |   | |
///  $--ROT,x.x,A*hh<CR><LF>
/// ```
/// 1. Rate Of Turn, degrees per minute, "-" means bow turns to port
/// 2. Status, A means data is valid
/// 3. Checksum
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[derive(Debug, PartialEq)]
pub struct RotData {
    /// Rate Of Turn, degrees per minute, "-" means bow turns to port
    pub rot: Option<f32>,
    pub valid: bool,
}

/// # Parse ROT message
///
/// ```text
/// $HEROT,0.0,A*2B
///
/// ROT,x.x,A*hh<CR><LF>
/// ```
pub fn parse_rot(sentence: NmeaSentence) -> Result<RotData, Error> {
    if sentence.message_id != SentenceType::ROT {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::ROT,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_rot(sentence.data)?.1)
    }
}

fn do_parse_rot(i: &str) -> IResult<&str, RotData> {
    let (i, rot) = opt(super::utils::number::<f32>)(i)?;
    let (i, _) = char(',')(i)?;
    let (i, valid) = parse_valid_status(i)?;
    Ok((i, RotData { rot, valid }))
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::parse::parse_nmea_sentence;

    #[test]
    fn test_parse_rot_v2_full() {
        let s = parse_nmea_sentence("$TIROT,-42.1,A*21").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());

        let RotData { rot, valid } = parse_rot(s).unwrap();
        assert_relative_eq!(rot.unwrap(), -42.1);
        assert!(valid);
    }
}
