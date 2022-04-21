mod file_format;

use indicatif::{ProgressBar, ProgressStyle};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use uuid::Uuid;

use file_format::FileFormatFactory;
use rayon::iter::IntoParallelRefIterator;

fn is_music_file(path: &PathBuf) -> bool {
    match path.extension() {
        None => false,
        Some(ext) => match ext.to_str().unwrap_or("") {
            #[cfg(feature = "m4a")]
            "m4a" => true,
            #[cfg(feature = "mp3")]
            "mp3" => true,
            #[cfg(feature = "flac")]
            "flac" => true,
            _ => false,
        },
    }
}

/* TODO: Multithread this? */
fn dir_list(path: &str, vec: &mut Vec<String>) {
    std::fs::read_dir(path)
        .unwrap()
        .into_iter()
        .for_each(|entry| {
            let info = entry.as_ref().unwrap();
            let path = info.path();

            if path.is_dir() {
                let len = vec.len();
                let path = path.to_str().unwrap();
                dir_list(path, vec);
                if vec.len() != len {
                    let path = path.to_string();
                    if !vec.contains(&path) {
                        vec.push(path);
                    }
                }
            } else {
                let path = PathBuf::from(path);
                if is_music_file(&path) {
                    let parent_path = path.parent().unwrap().to_str().unwrap().to_string();
                    if !vec.contains(&parent_path) {
                        vec.push(parent_path);
                    }
                    return;
                }
            }
        });
}

use rayon::prelude::*;

#[derive(Serialize, Debug)]
struct FileEntry {
    folder: String,
    name: String,
    format: String,
    duration: u64,
    uuid: String,
}

#[derive(Serialize, Debug)]
struct FileEntries {
    folders: Vec<String>,
    files: Vec<FileEntry>,
}

fn truncate(input: &String, len: usize) -> String {
	if input.len() <= len {
		return input.clone();
	}
	let ilen = input.len();
	format!("{}....{}", &input[0..len / 2 - 2], &input[ilen - (len / 2 - 2) ..])
}

fn main() {
    let mut folders = vec![];
    dir_list("./", &mut folders);

    for folder in folders.iter_mut() {
        *folder = folder.to_owned() + "/";
    }

    let entries: Mutex<Vec<FileEntry>> = Mutex::new(vec![]);
    let bar = ProgressBar::new(folders.len() as u64);

    bar.set_style(
        ProgressStyle::default_bar()
            .template("{wide_bar} {pos:>7}/{len:7}\n{msg}")
            .progress_chars("##-"),
    );

    folders.par_iter().for_each(|dir| {
        std::fs::read_dir(&dir)
            .unwrap()
            .into_iter()
            .for_each(|entry| {
                let info = entry.as_ref().unwrap();
                let path = info.path();

                if path.is_file() && is_music_file(&path) {
                    bar.set_message(truncate(&path.to_str().unwrap().to_string(), term_size::dimensions().unwrap().0));
                    let file = FileFormatFactory::new_file(path.as_os_str().to_str().unwrap());
                    match file {
                        Some(file) => match file.duration() {
                            Ok(duration) => {
                                let entry = FileEntry {
                                    name: path
                                        .with_extension("")
                                        .components()
                                        .last()
                                        .unwrap()
                                        .as_os_str()
                                        .to_str()
                                        .unwrap()
                                        .to_string(),
                                    format: file.extension().to_string(),
                                    folder: dir.clone(),
                                    duration: duration.as_secs(),
                                    uuid: Uuid::new_v4().to_string().replace("-", ""),
                                };
                                let mut data = entries.lock().unwrap();
                                (*data).push(entry);
                            }
                            Err(_) => {},
                        },
                        None => println!("Invalid format"),
                    };
                }
            });
        bar.inc(1);
    });

    bar.finish_and_clear();
    folders.sort();

    let final_obj = FileEntries {
		folders,
        files: entries.into_inner().unwrap(),
    };

	let mut final_duration: u64 = 0;

	for file in &final_obj.files {
		final_duration += file.duration;
	}

	println!("Indexed {} files", final_obj.files.len());
	println!("Total: {} days, {} hours, {} minutes and {} seconds of music", final_duration / (3600 * 24), (final_duration / 3600) % 24, (final_duration / 60) % 60, final_duration % 60);

    let json = serde_json::to_string(&final_obj).unwrap();

    fs::write("index.json", json).unwrap();
}
