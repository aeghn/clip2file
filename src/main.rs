mod arguments;
use clipboard_rs::{common::RustImage, Clipboard, ClipboardContext, ContentFormat};

use std::{
    fs::{self},
    path::{Path, PathBuf},
};

use arguments::Arguments;
use clap::Parser;

fn timestamp() -> String {
    let date = chrono::Local::now();
    date.format("%y%m-%d-%H%M%S").to_string()
}

fn read_and_save(args: &Arguments) -> anyhow::Result<Vec<PathBuf>> {
    let ctx = ClipboardContext::new().unwrap();
    let mut paths = vec![];
    if let Ok(img) = ctx.get_image() {
        let file_path = build_filepath(args, None::<String>);
        match img.save_to_path(file_path.to_string_lossy().as_ref()) {
            Ok(_) => {
                paths.push(file_path);
            }
            Err(error) => Err(anyhow::anyhow!(error.to_string()))?,
        }
    } else if let Ok(text) = ctx.get_text() {
        let text_str = text.as_str();
        if PathBuf::from(text_str).exists() {
            if let Ok(Some(_)) = imghdr::from_file(text_str) {
                let filepath = build_filepath(args, Some(text_str));
                std::fs::copy(text_str, &filepath)?;
                paths.push(filepath);
            }
        }
    } else if let Ok(files) = ctx.get(&[ContentFormat::Files]) {
        for clip_content in files {
            if let clipboard_rs::ClipboardContent::Files(files) = clip_content {
                for file in files {
                    if let Ok(Some(_)) = imghdr::from_file(&file) {
                        let p = PathBuf::from(file);
                        let filepath = build_filepath(args, Some(p.to_path_buf()));
                        std::fs::copy(p, &filepath)?;
                    }
                }
            }
        }
    }

    Ok(paths)
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
        for filepath in paths {
            if is_img_file(&filepath) {
                println!("{}", diff_wrapper(&args, filepath))
            }
        }
    }
}
