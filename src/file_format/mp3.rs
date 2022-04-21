use super::FileFormat;
use std::{path::Path, time::Duration};

pub struct Mp3Format {
    path: String,
}

impl Mp3Format {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl FileFormat for Mp3Format {
    fn extension(&self) -> &'static str {
        "mp3"
    }

    fn duration(&self) -> Result<Duration, String> {
        let path = Path::new(&self.path);
        match mp3_duration::from_path(path) {
            Ok(duration) => return Ok(duration),
            Err(e) => return Err(e.to_string()),
        };
    }
}
