use nom::{character::complete::char, combinator::opt, number::complete::float, IResult};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{Error, NmeaSentence, SentenceType};

/// DPT - Depth of Water
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_dpt_depth_of_water>
///
/// ```text
///        1   2   3   4
///        |   |   |   |
/// $--DPT,x.x,x.x,x.x*hh<CR><LF>
/// ```
/// 1. Water depth relative to transducer, meters
/// 2. Offset from transducer, meters positive means distance from transducer to water line negative means distance from transducer to keel
/// 3. Maximum range scale in use (NMEA 3.0 and above)
/// 4. Checksum
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[derive(Debug, PartialEq)]
pub struct DptData {
    /// Water depth relative to transducer, meters
    pub relative_depth: Option<f32>,
    /// Offset from transducer, meters
    /// If positive, it measures the distance from the transducer to the water line.
    /// If negative, it measures the distance from the transducer to the keel
    pub offset_from_transducer: Option<f32>,
    /// Maximum range scale in use
    pub max_scale_in_use: Option<f32>,
}

/// # Parse DPT message
///
/// ```text
/// $INDPT,2.3,0.0*46
///
/// DPT,x.x,x.x,x.x*hh<CR><LF>
/// ```
pub fn parse_dpt(sentence: NmeaSentence) -> Result<DptData, Error> {
    if sentence.message_id != SentenceType::DPT {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::DPT,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_dpt(sentence.data)?.1)
    }
}

fn do_parse_dpt(i: &str) -> IResult<&str, DptData> {
    let (i, relative_depth) = opt(float)(i)?;
    let (i, _) = char(',')(i)?;
    let (i, offset_from_transducer) = opt(float)(i)?;

    // Optionally NMEA v3 data
    let (i, _) = opt(char(','))(i)?;
    let (i, max_scale_in_use) = opt(float)(i)?;
    Ok((
        i,
        DptData {
            relative_depth,
            offset_from_transducer,
            max_scale_in_use,
        },
    ))
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::parse::parse_nmea_sentence;

    #[test]
    fn test_parse_dpt_v2_full() {
        let s = parse_nmea_sentence("$INDPT,2.3,0.0*46").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());

        let DptData {
            relative_depth,
            offset_from_transducer,
            max_scale_in_use,
        } = parse_dpt(s).unwrap();
        assert_relative_eq!(relative_depth.unwrap(), 2.3);
        assert_relative_eq!(offset_from_transducer.unwrap(), 0.0);
        assert!(max_scale_in_use.is_none());
    }

    #[test]
    fn test_parse_dpt_v3_full() {
        let s = parse_nmea_sentence("$INDPT,2.3,0.0,32.5*70").unwrap();
        assert_eq!(s.checksum, s.calc_checksum());

        let DptData {
            relative_depth,
            offset_from_transducer,
            max_scale_in_use,
        } = parse_dpt(s).unwrap();
        assert_relative_eq!(relative_depth.unwrap(), 2.3);
        assert_relative_eq!(offset_from_transducer.unwrap(), 0.0);
        assert_relative_eq!(max_scale_in_use.unwrap(), 32.5);
    }
}
