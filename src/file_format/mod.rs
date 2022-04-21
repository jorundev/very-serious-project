use std::{path::PathBuf, time::Duration};

#[cfg(feature = "flac")]
pub mod flac;
#[cfg(feature = "flac")]
use self::flac::FlacFormat;

#[cfg(feature = "mp3")]
pub mod mp3;
#[cfg(feature = "mp3")]
use self::mp3::Mp3Format;

#[cfg(feature = "m4a")]
pub mod m4a;
#[cfg(feature = "m4a")]
use self::m4a::M4aFormat;

use std::str::FromStr;

pub trait FileFormat {
    fn extension(&self) -> &'static str;
    fn duration(&self) -> Result<Duration, String>;
}

pub struct FileFormatFactory {}

impl FileFormatFactory {
    pub fn new_file(path: &str) -> Option<Box<dyn FileFormat>> {
        let path = PathBuf::from_str(path).unwrap();
        if let Some(ext) = path.extension() {
            if let Some(ext) = ext.to_str() {
                match ext {
                    #[cfg(feature = "mp3")]
                    "mp3" => {
                        return Some(Box::new(Mp3Format::new(path.to_str().unwrap().to_string())))
                    }
                    #[cfg(feature = "flac")]
                    "flac" => {
                        return Some(Box::new(FlacFormat::new(
                            path.to_str().unwrap().to_string(),
                        )))
                    }
                    #[cfg(feature = "m4a")]
                    "m4a" => {
                        return Some(Box::new(M4aFormat::new(path.to_str().unwrap().to_string())))
                    }
                    _ => return None,
                }
            }
        };
        None
    }
}
