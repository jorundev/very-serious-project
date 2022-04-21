use super::FileFormat;
use std::time::Duration;

pub struct M4aFormat {
    path: String,
}

impl M4aFormat {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl FileFormat for M4aFormat {
    fn extension(&self) -> &'static str {
        "m4a"
    }

    fn duration(&self) -> Result<Duration, String> {
        let tag = match mp4ameta::Tag::read_from_path(&self.path) {
            Ok(tag) => tag,
            Err(err) => return Err(err.to_string()),
        };
        match tag.audio_info().duration {
            Some(duration) => return Ok(duration),
            None => return Err("No Duration Data".to_string()),
        }
    }
}
