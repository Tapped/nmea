use arrayvec::ArrayString;
use chrono::{Duration, NaiveTime};
use nom::{bytes::complete::is_not, character::complete::char, combinator::opt};
use serde::Serialize;

use crate::{
    parse::{NmeaSentence, TEXT_PARAMETER_MAX_LEN},
    sentences::utils::{parse_duration_hms, parse_hms},
    Error, SentenceType,
};

use super::utils::array_string;

/// ZTG - UTC & Time to Destination Waypoint
///```text
///        1         2         3    4
///        |         |         |    |
/// $--ZTG,hhmmss.ss,hhmmss.ss,c--c*hh<CR><LF>
///```
/// Field Number:
/// 1. UTC of observation hh is hours, mm is minutes, ss.ss is seconds.
/// 2. Time Remaining
/// 3. Destination Waypoint ID
/// 4. Checksum
#[serde_with::serde_as]
#[derive(Serialize, Debug, PartialEq)]
pub struct ZtgData {
    pub fix_time: Option<NaiveTime>,
    #[serde_as(as = "Option<serde_with::DurationSeconds<i64>>")]
    pub fix_duration: Option<Duration>,
    pub waypoint_id: Option<ArrayString<TEXT_PARAMETER_MAX_LEN>>,
}

fn do_parse_ztg(i: &str) -> Result<ZtgData, Error> {
    // 1. UTC Time or observation
    let (i, fix_time) = opt(parse_hms)(i)?;
    let (i, _) = char(',')(i)?;
    // 2. Duration
    let (i, fix_duration) = opt(parse_duration_hms)(i)?;
    let (i, _) = char(',')(i)?;

    // 12. Waypoint ID
    let (_i, waypoint_id) = opt(is_not(",*"))(i)?;

    let waypoint_id = waypoint_id
        .map(array_string::<TEXT_PARAMETER_MAX_LEN>)
        .transpose()?;

    Ok(ZtgData {
        fix_time,
        fix_duration,
        waypoint_id,
    })
}

/// # Parse ZTG message
///
/// See: <https://gpsd.gitlab.io/gpsd/NMEA.html#_ztg_utc_time_to_destination_waypoint>
pub fn parse_ztg(sentence: NmeaSentence) -> Result<ZtgData, Error> {
    if sentence.message_id != SentenceType::ZTG {
        Err(Error::WrongSentenceHeader {
            expected: SentenceType::ZTG,
            found: sentence.message_id,
        })
    } else {
        Ok(do_parse_ztg(sentence.data)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse::parse_nmea_sentence, Error};

    fn run_parse_ztg(line: &str) -> Result<ZtgData, Error> {
        let s = parse_nmea_sentence(line).expect("ZTG sentence initial parse failed");
        assert_eq!(s.checksum, s.calc_checksum());
        parse_ztg(s)
    }

    #[test]
    fn test_parse_ztg() {
        assert_eq!(
            ZtgData {
                fix_duration: Some(
                    Duration::hours(4)
                        + Duration::minutes(23)
                        + Duration::seconds(59)
                        + Duration::milliseconds(170)
                ),
                fix_time: NaiveTime::from_hms_milli_opt(14, 58, 32, 120),
                waypoint_id: Some(ArrayString::from("WPT").unwrap()),
            },
            run_parse_ztg("$GPZTG,145832.12,042359.17,WPT*24").unwrap()
        );
        assert_eq!(
            ZtgData {
                fix_duration: None,
                fix_time: None,
                waypoint_id: None,
            },
            run_parse_ztg("$GPZTG,,,*72").unwrap()
        );
        assert_eq!(
            ZtgData {
                fix_duration: Some(
                    Duration::hours(4)
                        + Duration::minutes(23)
                        + Duration::seconds(59)
                        + Duration::milliseconds(170)
                ),
                fix_time: None,
                waypoint_id: None,
            },
            run_parse_ztg("$GPZTG,,042359.17,*53").unwrap()
        );
    }
    #[test]
    fn test_parse_ztg_with_too_long_waypoint() {
        assert_eq!(
            Error::ParameterLength { max_length: 64, parameter_length: 72 },
            run_parse_ztg("$GPZTG,145832.12,042359.17,ABCDEFGHIJKLMNOPRSTUWXYZABCDEFGHIJKLMNOPRSTUWXYZABCDEFGHIJKLMNOPRSTUWXYZ*6B").unwrap_err()
        );
    }
}
