use std::fmt::Write;
use std::{
    ffi::OsStr,
    fs::{self, DirEntry},
    path::{Path, PathBuf},
    sync::{
        mpsc::{self, Sender},
        Mutex,
    },
};

lazy_static::lazy_static! {
    static ref ULT_STRING: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref COUNT: Mutex<usize> = Mutex::new(0);
}

use clap::Parser;
use metaflac::Tag;
use rayon::Scope;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    path: String,
}

fn get_file_info(s: &OsStr, path: &Path) -> Option<(u64, String)> {
    let title = path
        .with_extension("")
        .components()
        .last()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string();
    let length = match s.to_str().unwrap() {
        "mp3" => {
            let metadata = match mp3_metadata::read_from_file(path) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("{}", e);
                    return None;
                }
            };
            metadata.duration.as_secs()
        }
        "flac" => {
            let tag = Tag::read_from_path(path).unwrap();
            let info = tag.get_streaminfo().unwrap();
            info.total_samples / info.sample_rate as u64
        }
        _ => return None,
    };
    Some((length, title))
}

fn scan<'a, U: AsRef<Path>>(
    src: &U,
    tx: Sender<(Result<DirEntry, std::io::Error>, u64)>,
    scope: &Scope<'a>,
) {
    let dir = fs::read_dir(src).unwrap();
    dir.into_iter().for_each(|entry| {
        let info = entry.as_ref().unwrap();
        let path = info.path();

        if path.is_dir() {
            let tx = tx.clone();
            scope.spawn(move |s| scan(&path, tx, s));
        } else {
            if let Some(format) = path.extension() {
                {
                    let mut count = COUNT.lock().unwrap();
                    *count += 1;
                    eprintln!("{}", count);
                }
                let (length, title) = match get_file_info(format, &path) {
                    Some(d) => d,
                    _ => return,
                };

                let mut v = ULT_STRING.lock().unwrap();

                let mut s = String::new();

                write!(s, "  {{\n    \"path\": {:?},\n    \"format\": {:?},\n    \"title\": {:?},\n    \"duration\": {}\n  }}",
                    path.parent().unwrap(),
                    format.to_ascii_lowercase(),
                    title,
                    length).unwrap();
                v.push(s);
            }
            let size = info.metadata().unwrap().len();
            tx.send((entry, size)).unwrap();
        }
    });
}

fn main() {
    let args = Args::parse();
    let source = PathBuf::from(args.path);

    let (tx, _rx) = mpsc::channel();

    rayon::scope(|s| scan(&source, tx, s));
    println!("{{");
    let lock = ULT_STRING.lock().unwrap();
    for (i, string) in lock.iter().enumerate() {
        print!("{}", string);
        if i != lock.iter().len() - 1 {
            print!(",");
        }
        println!();
    }
    println!("}}");
}
