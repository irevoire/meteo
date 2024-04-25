use std::{cmp::Ordering, ops::Range, str::FromStr};

use logos::Logos;
use thiserror::Error;
use time::{Date, Month, PrimitiveDateTime};

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t]+")] // Ignore this regex pattern between tokens
enum Token {
    #[token("MONTHLY CLIMATOLOGICAL SUMMARY for ")]
    MonthlyClimatologicalSummary,

    // Month
    #[token("JAN")]
    January,
    #[regex("FE(B|V)", priority = 3)]
    February,
    #[token("MAR")]
    March,
    #[token("APR")]
    April,
    #[token("MAY")]
    May,
    #[token("JUN")]
    June,
    #[token("JUL")]
    July,
    #[token("AUG")]
    August,
    #[token("SEP")]
    September,
    #[token("OCT")]
    October,
    #[token("NOV")]
    November,
    #[token("DEC")]
    December,

    #[regex(r"-?[0-9]+(\.[0-9]+)?")]
    Number,
    #[regex("[a-zA-Z]+")]
    String,

    #[token("NAME:")]
    Name,
    #[token("CITY:")]
    City,
    #[token("STATE:")]
    State,
    #[token("ELEV:")]
    Elevation,
    #[token("LAT:")]
    Latitude,
    #[token("LONG:")]
    Longitude,

    #[token("\n")]
    Crlf,
    #[token("---")]
    MissingData,
    #[token(":")]
    Colon,
    #[token(".")]
    Dot,
}

#[derive(Debug, Clone)]
pub struct Report {
    pub metadata: Metadata,
    // Days should be sorted by date
    pub days: Vec<Day>,
}

impl PartialEq for Report {
    fn eq(&self, other: &Self) -> bool {
        self.metadata.date == other.metadata.date
    }
}

impl Eq for Report {}

impl PartialOrd for Report {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.metadata.date.partial_cmp(&other.metadata.date)
    }
}

impl Ord for Report {
    fn cmp(&self, other: &Self) -> Ordering {
        self.metadata.date.cmp(&other.metadata.date)
    }
}

impl Report {
    pub fn first_date(&self) -> Date {
        self.days.first().unwrap().date
    }

    pub fn last_date(&self) -> Date {
        self.days.last().unwrap().date
    }

    pub fn range<T>(&self, retrieve: fn(&Day) -> T, compare: fn(&T, &T) -> Ordering) -> Range<T> {
        self.days.iter().map(retrieve).min_by(compare).unwrap()
            ..self.days.iter().map(retrieve).max_by(compare).unwrap()
    }

    pub fn temperature_range(&self) -> Range<f32> {
        self.days
            .iter()
            .map(|day| day.low_temp)
            .min_by(|left, right| left.total_cmp(right))
            .unwrap()
            ..self
                .days
                .iter()
                .map(|day| day.high_temp)
                .max_by(|left, right| left.total_cmp(right))
                .unwrap()
    }

    /// Can only merge reports from the exact same header
    /// Except the date which **must** change or it'll return an error.
    pub fn merge(&mut self, mut other: Self) -> Result<(), String> {
        if self.metadata != other.metadata {
            return Err(String::from("Metada differs"));
        }

        if self.metadata.date < other.metadata.date {
            self.days.append(&mut other.days);
        } else {
            self.metadata.date = other.metadata.date;
            other.days.append(&mut self.days);
            self.days = other.days;
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error(transparent)]
    MetadataError(#[from] MetadataError),
    #[error(transparent)]
    ParseDayError(#[from] ParseDayError),
}

impl FromStr for Report {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let metadata = Metadata::parse(&mut lines)?;

        // Skip everything until the big bar
        loop {
            let line = lines.next().unwrap();
            if !line.is_empty() && line.chars().all(|c| c == '-') {
                break;
            }
        }

        let mut days: Vec<Day> = Vec::new();

        // Parse all days until the next big bar
        loop {
            let line = lines.next().unwrap();
            if line.chars().all(|c| c == '-') {
                break;
            }

            let day = match Day::parse(metadata.date, line) {
                Ok(day) => day,
                Err(ParseDayError::EmptyDay) => continue,
                Err(e) => return Err(e.into()),
            };

            if let Some(d) = days.last() {
                if d.date >= day.date {
                    eprintln!("days are not ordered");
                }
            }
            days.push(day);
        }

        Ok(Self { metadata, days })
    }
}

#[derive(Debug, Clone, Eq)]
pub struct Metadata {
    // Date of the beginning of the month, doesn't take into account the fact
    // that days may be missing. Do not rely on it
    pub date: Date,

    pub name: String,
    pub city: String,
    pub state: String,

    pub elevation: usize,
    pub lat: (u8, u8, u8),
    pub long: (u8, u8, u8),
}

impl PartialEq for Metadata {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
            && self.city.eq(&other.city)
            && self.state.eq(&other.state)
            && self.elevation.eq(&other.elevation)
            && self.lat.eq(&other.lat)
            && self.long.eq(&other.long)
    }
}

#[derive(Debug, Error)]
pub enum MetadataError {
    #[error("Missing title")]
    MissingTitle,
    #[error("Bad title")]
    BadTitle,
    #[error("Bad month: {0}")]
    BadMonth(String),
    #[error("Bad header")]
    BadHeader,
}

impl Metadata {
    pub fn parse<'a>(mut lines: impl Iterator<Item = &'a str>) -> Result<Self, MetadataError> {
        let title = lines.next().ok_or(MetadataError::MissingTitle)?;
        let mut title = Token::lexer(title);
        match title.next() {
            Some(Ok(Token::MonthlyClimatologicalSummary)) => (),
            _ => return Err(MetadataError::BadTitle),
        };
        let month = match title.next() {
            Some(Ok(Token::January)) => Month::January,
            Some(Ok(Token::February)) => Month::February,
            Some(Ok(Token::March)) => Month::March,
            Some(Ok(Token::April)) => Month::April,
            Some(Ok(Token::May)) => Month::May,
            Some(Ok(Token::June)) => Month::June,
            Some(Ok(Token::July)) => Month::July,
            Some(Ok(Token::August)) => Month::August,
            Some(Ok(Token::September)) => Month::September,
            Some(Ok(Token::October)) => Month::October,
            Some(Ok(Token::November)) => Month::November,
            Some(Ok(Token::December)) => Month::December,
            _ => return Err(MetadataError::BadMonth(title.slice().to_string())),
        };

        match title.next() {
            Some(Ok(Token::Dot)) => (),
            _ => {
                return Err(MetadataError::BadMonth(String::from(
                    "Missing dot after month",
                )))
            }
        };

        let year = match title.next() {
            Some(Ok(Token::Number)) => title.slice().parse().unwrap(),
            _ => {
                return Err(MetadataError::BadMonth(String::from(
                    "Missing year after month",
                )))
            }
        };

        let date = Date::from_calendar_date(year, month, 1).unwrap();

        let empty = lines.next().ok_or(MetadataError::BadHeader)?;
        assert!(empty.is_empty());

        // TODO: parse the rest of the headers

        Ok(Self {
            date,
            name: String::from("maxou"),
            city: String::from("LE VIGAN"),
            state: String::from("FRONCE"),
            elevation: 245,
            lat: (43, 59, 23),
            long: (3, 36, 4),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Day {
    pub date: Date,

    pub mean_temp: f32,
    pub high_temp: f32,
    pub high_temp_date: PrimitiveDateTime,
    pub low_temp: f32,
    pub low_temp_date: PrimitiveDateTime,

    pub rain: f32,

    pub avg_wind_speed: f32,
    pub high_wind_speed: f32,
    pub high_wind_speed_date: Option<PrimitiveDateTime>,
    pub wind_direction: Option<Direction>,
}

#[derive(Debug, Error)]
pub enum ParseDayError {
    #[error("Invalid day: {0}")]
    InvalidDay(#[from] time::error::ComponentRange),
    #[error("Empty day")]
    EmptyDay,
    #[error("Bad day")]
    BadDay,
    #[error("Bad thing: {0}")]
    BadThing(String),
}

impl Day {
    pub fn parse(date: Date, s: &str) -> Result<Self, ParseDayError> {
        let mut day = Token::lexer(s);
        let day_number = match day.next() {
            Some(Ok(Token::Number)) => day.slice().parse().unwrap(),
            _ => return Err(ParseDayError::BadDay),
        };

        let date = date.replace_day(day_number)?;

        let mean_temp = match day.next() {
            Some(Ok(Token::Number)) => day.slice().parse().unwrap(),
            Some(Ok(Token::Crlf)) => return Err(ParseDayError::EmptyDay),
            None => return Err(ParseDayError::EmptyDay),
            Some(Ok(token)) => {
                return Err(ParseDayError::BadThing(format!(
                    "Bad mean temp token: {:?}",
                    token
                )))
            }
            a => return Err(ParseDayError::BadThing(format!("Bad mean temp: {:?}", a))),
        };

        let high_temp = match day.next() {
            Some(Ok(Token::Number)) => day.slice().parse().unwrap(),
            _ => return Err(ParseDayError::BadThing(String::from("Bad high temp"))),
        };

        let hour = match day.next() {
            Some(Ok(Token::Number)) => day.slice().parse().unwrap(),
            _ => return Err(ParseDayError::BadThing(String::from("Bad high temp hour"))),
        };
        match day.next() {
            Some(Ok(Token::Colon)) => (),
            _ => return Err(ParseDayError::BadThing(String::from("Bad high temp colon"))),
        };
        let minute = match day.next() {
            Some(Ok(Token::Number)) => day.slice().parse().unwrap(),
            _ => {
                return Err(ParseDayError::BadThing(String::from(
                    "Bad high temp minute",
                )))
            }
        };
        let high_temp_date = date
            .with_hms(hour, minute, 0)
            .map_err(|e| ParseDayError::BadThing(e.to_string()))?;

        let low_temp = match day.next() {
            Some(Ok(Token::Number)) => day.slice().parse().unwrap(),
            _ => return Err(ParseDayError::BadThing(String::from("Bad low temp"))),
        };

        let hour = match day.next() {
            Some(Ok(Token::Number)) => day.slice().parse().unwrap(),
            _ => return Err(ParseDayError::BadThing(String::from("Bad low temp hour"))),
        };
        match day.next() {
            Some(Ok(Token::Colon)) => (),
            _ => return Err(ParseDayError::BadThing(String::from("Bad low temp colon"))),
        };
        let minute = match day.next() {
            Some(Ok(Token::Number)) => day.slice().parse().unwrap(),
            _ => return Err(ParseDayError::BadThing(String::from("Bad low temp minute"))),
        };
        let low_temp_date = date
            .with_hms(hour, minute, 0)
            .map_err(|e| ParseDayError::BadThing(e.to_string()))?;

        // skip the heat deg days and cool deg days
        match day.next() {
            Some(Ok(Token::Number)) => day.slice(),
            _ => return Err(ParseDayError::BadThing(String::from("heat truc"))),
        };
        match day.next() {
            Some(Ok(Token::Number)) => day.slice(),
            _ => return Err(ParseDayError::BadThing(String::from("heat truc"))),
        };

        let rain = match day.next() {
            Some(Ok(Token::Number)) => day.slice().parse().unwrap(),
            _ => return Err(ParseDayError::BadThing(String::from("Bad rain"))),
        };

        let avg_wind_speed = match day.next() {
            Some(Ok(Token::Number)) => day.slice().parse().unwrap(),
            _ => return Err(ParseDayError::BadThing(String::from("Bad avg wind speed"))),
        };

        let high_wind_speed = match day.next() {
            Some(Ok(Token::Number)) => day.slice().parse().unwrap(),
            _ => return Err(ParseDayError::BadThing(String::from("Bad high wind speed"))),
        };

        // high_wind_speed_date
        let high_wind_speed_date = match day.next() {
            Some(Ok(Token::Number)) => {
                let hour = day.slice().parse().unwrap();
                match day.next() {
                    Some(Ok(Token::Colon)) => (),
                    _ => {
                        return Err(ParseDayError::BadThing(String::from(
                            "Bad high wind speed colon",
                        )))
                    }
                };
                let minute = match day.next() {
                    Some(Ok(Token::Number)) => day.slice().parse().unwrap(),
                    _ => {
                        return Err(ParseDayError::BadThing(String::from(
                            "Bad high wind speed minute",
                        )))
                    }
                };
                let high_wind_speed_date = date
                    .with_hms(hour, minute, 0)
                    .map_err(|e| ParseDayError::BadThing(e.to_string()))?;
                Some(high_wind_speed_date)
            }
            Some(Ok(Token::MissingData)) => None,
            _ => {
                return Err(ParseDayError::BadThing(String::from(
                    "Bad high wind speed hour",
                )))
            }
        };

        let wind_direction = match day.next() {
            Some(Ok(Token::String)) => Some(day.slice().parse().unwrap()),
            Some(Ok(Token::MissingData)) => None,
            _ => return Err(ParseDayError::BadThing(String::from("Wind direction"))),
        };

        Ok(Self {
            date,
            mean_temp,
            high_temp,
            high_temp_date,
            low_temp,
            low_temp_date,
            rain,
            avg_wind_speed,
            high_wind_speed,
            high_wind_speed_date,
            wind_direction,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    N,
    NNE,
    NE,
    ENE,
    E,
    ESE,
    SE,
    SSE,
    S,
    SSW,
    SW,
    WSW,
    W,
    WNW,
    NW,
    NNW,
}

impl FromStr for Direction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "N" => Ok(Self::N),
            "NNE" => Ok(Self::NNE),
            "NE" => Ok(Self::NE),
            "ENE" => Ok(Self::ENE),
            "E" => Ok(Self::E),
            "ESE" => Ok(Self::ESE),
            "SE" => Ok(Self::SE),
            "SSE" => Ok(Self::SSE),
            "S" => Ok(Self::S),
            "SSW" => Ok(Self::SSW),
            "SW" => Ok(Self::SW),
            "WSW" => Ok(Self::WSW),
            "W" => Ok(Self::W),
            "WNW" => Ok(Self::WNW),
            "NW" => Ok(Self::NW),
            "NNW" => Ok(Self::NNW),
            s => Err(format!("Unknown wind direction: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemperatureUnit {
    Celsius,
}

impl FromStr for TemperatureUnit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ºC" => Ok(Self::Celsius),
            s => Err(format!("Unknown temperature unit {s}. Expecting `ºC`")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RainUnit {
    Mm,
}

impl FromStr for RainUnit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mm" => Ok(Self::Mm),
            s => Err(format!("Unknown rain unit {s}. Expecting `mm`")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindSpeedUnit {
    KmHr,
}

impl FromStr for WindSpeedUnit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "km/hr" => Ok(Self::KmHr),
            s => Err(format!("Unknown wind speed unit {s}. Expecting km/hr")),
        }
    }
}
