mod arguments;
use clipboard_rs::{common::RustImage, Clipboard, ClipboardContext, ContentFormat};

use std::{
    fs::{self},
    path::{Path, PathBuf},
    process::exit,
};

use arguments::Arguments;
use clap::Parser;

enum ClipType {
    ImageOrFile(Vec<PathBuf>),
    Other,
}

fn timestamp() -> String {
    let date = chrono::Local::now();
    date.format("%y%m-%d-%H%M%S").to_string()
}

fn read_and_save(args: &Arguments) -> anyhow::Result<ClipType> {
    let ctx = ClipboardContext::new().unwrap();
    let mut paths = vec![];
    let mut img_or_file = false;
    let only_image = args.only_img.unwrap_or(false);
    fn should_save(line: &str, only_image: bool) -> bool {
        !only_image || (only_image && imghdr::from_file(&line).is_ok_and(|e| e.is_some()))
    }

    let mut handle_text = || -> anyhow::Result<()> {
        if let Ok(text) = ctx.get_text() {
            if args.parse_text.unwrap_or(false) {
                for line in text.split("\n") {
                    if PathBuf::from(line).exists() {
                        let line = line.trim();

                        if should_save(line, only_image) {
                            img_or_file = true;
                            let filepath = build_filepath(args, Some(line));
                            std::fs::copy(line, &filepath)?;
                            paths.push(filepath);
                        }
                    }
                }
            }
        }
        Ok(())
    };

    if let Ok(img) = ctx.get_image() {
        let file_path = build_filepath(args, None::<String>);
        match img.save_to_path(file_path.to_string_lossy().as_ref()) {
            Ok(_) => {
                img_or_file = true;
                paths.push(file_path);
            }
            Err(error) => Err(anyhow::anyhow!(error.to_string()))?,
        }
    } else if let Ok(files) = ctx.get(&[ContentFormat::Files]) {
        if files.is_empty() {
            handle_text()?;
        } else {
            img_or_file = true;
            for clip_content in files {
                if let clipboard_rs::ClipboardContent::Files(files) = clip_content {
                    for file in files {
                        if should_save(&file, only_image) {
                            let p = PathBuf::from(&file);
                            let filepath = build_filepath(args, Some(p.to_path_buf()));
                            std::fs::copy(p, &filepath)?;
                        }
                    }
                }
            }
        }
    } else {
        handle_text()?;
    }

    if img_or_file {
        Ok(ClipType::ImageOrFile(paths))
    } else {
        Ok(ClipType::Other)
    }
}

fn diff_wrapper(args: &Arguments, filepath: impl AsRef<Path>) -> String {
    let path = if let Some(p) = args.base_dir.clone() {
        if let Some(diff) = pathdiff::diff_paths(&filepath, p) {
            diff
        } else {
            filepath.as_ref().to_path_buf()
        }
    } else {
        filepath.as_ref().to_path_buf()
    };

    let path = path.as_os_str().to_string_lossy().to_owned();
    #[cfg(target_os = "windows")]
    let path = path.replace("\\", "/");

    path
}

fn is_img_file(filepath: impl AsRef<Path>) -> bool {
    imghdr::from_file(filepath.as_ref()).map_or(false, |e| e.is_some())
}

fn build_filepath(args: &Arguments, original_filepath: Option<impl AsRef<Path>>) -> PathBuf {
    let dir_path = PathBuf::from(&args.dir);

    let filename = if let Some(mut filename) = args.name.clone() {
        if args.timestamp.unwrap_or(true) {
            filename.insert(0, '-');
            filename.insert_str(0, &timestamp());
        }
        filename
    } else if let Some(ori_name) = original_filepath
        .as_ref()
        .and_then(|p| p.as_ref().file_name())
        .map(|s| s.to_string_lossy())
    {
        let mut filename = ori_name.to_string();
        if args.timestamp.unwrap_or(true) {
            filename.insert(0, '-');
            filename.insert_str(0, timestamp().as_str());
        }
        filename
    } else {
        let mut filename = timestamp();
        filename.push_str("-clipboard.png");
        filename
    };

    dir_path.join(filename)
}

fn main() {
    let args: Arguments = Arguments::parse();

    let dir_path = PathBuf::from(args.dir.clone());

    if !dir_path.exists() {
        fs::create_dir_all(dir_path).unwrap();
    }
    if let Ok(paths) = read_and_save(&args) {
        match paths {
            ClipType::ImageOrFile(paths) => {
                for filepath in paths {
                    if is_img_file(&filepath) {
                        println!("{}", diff_wrapper(&args, filepath))
                    }
                }
            }
            ClipType::Other => exit(59),
        }
    } else {
        exit(59);
    }
}
