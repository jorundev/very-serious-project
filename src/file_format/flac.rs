use metaflac::Tag;

use super::FileFormat;
use std::time::Duration;

pub struct FlacFormat {
    path: String,
}

impl FlacFormat {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl FileFormat for FlacFormat {
    fn extension(&self) -> &'static str {
        "flac"
    }

    fn duration(&self) -> Result<Duration, String> {
        let tag = match Tag::read_from_path(&self.path) {
            Ok(tag) => tag,
            Err(err) => match err.kind {
                metaflac::ErrorKind::Io(ioerr) => return Err(ioerr.to_string()),
                metaflac::ErrorKind::StringDecoding(utf8error) => return Err(utf8error.to_string()),
                _ => return Err(err.to_string()),
            },
        };
        let info = match tag.get_streaminfo() {
            Some(info) => info,
            None => return Err("No stream information".to_string()),
        };
        let seconds = info.total_samples / info.sample_rate as u64;
        Ok(Duration::from_secs(seconds))
    }
}
