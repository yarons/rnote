use crate::config;

use gtk4::{gio, glib, prelude::*, Widget};
use p2d::bounding_volume::AABB;
use std::fs;
use std::path::PathBuf;

pub fn app_config_base_dirpath() -> Option<PathBuf> {
    let mut app_config_dirpath = glib::user_config_dir();
    app_config_dirpath.push(config::APP_NAME);
    let app_config_dir = gio::File::for_path(app_config_dirpath.clone());
    match app_config_dir.make_directory_with_parents(None::<&gio::Cancellable>) {
        Ok(()) => Some(app_config_dirpath),
        Err(e) => match e.kind::<gio::IOErrorEnum>() {
            Some(gio::IOErrorEnum::Exists) => Some(app_config_dirpath),
            _ => {
                log::error!("failed to create app_config_dir, {}", e);
                None
            }
        },
    }
}

#[derive(Debug)]
pub enum FileType {
    Folder,
    RnoteFile,
    XoppFile,
    VectorImageFile,
    BitmapImageFile,
    PdfFile,
    UnknownFile,
}

impl FileType {
    pub fn lookup_file_type(file: &gio::File) -> Self {
        if let Ok(info) = file.query_info(
            "standard::*",
            gio::FileQueryInfoFlags::NONE,
            None::<&gio::Cancellable>,
        ) {
            match info.file_type() {
                gio::FileType::Regular => {
                    if let Some(content_type) = info.content_type() {
                        match content_type.as_str() {
                            "application/rnote" => {
                                return Self::RnoteFile;
                            }
                            "application/x-xopp" => {
                                log::debug!(" is a xopp file ");
                                return Self::XoppFile;
                            }
                            "image/svg+xml" => {
                                return Self::VectorImageFile;
                            }
                            "image/png" | "image/jpeg" => {
                                return Self::BitmapImageFile;
                            }
                            "application/pdf" => {
                                return Self::PdfFile;
                            }
                            _ => {}
                        }
                    }
                }
                gio::FileType::Directory => {
                    return Self::Folder;
                }
                _ => {
                    log::warn!("unkown file type");
                    return Self::UnknownFile;
                }
            }
        } else {
            log::warn!("failed to query FileInfo from file");
        }

        // match on file extensions as fallback
        if let Some(path) = file.path() {
            if let Some(extension_str) = path.extension() {
                match &*extension_str.to_string_lossy() {
                    "rnote" => {
                        return Self::RnoteFile;
                    }
                    "xopp" => {
                        return Self::XoppFile;
                    }
                    _ => {}
                }
            }
        } else {
            log::warn!("no path for file");
        };

        Self::UnknownFile
    }
}

pub fn str_to_file(string: &str, file_path: &str) -> Result<(), anyhow::Error> {
    Ok(fs::write(PathBuf::from(file_path), string)?)
}

/// Translates a AABB to the coordinate space of the dest_widget. None if the widgets don't have a common ancestor
pub fn translate_aabb_to_widget(
    aabb: AABB,
    widget: &impl IsA<Widget>,
    dest_widget: &impl IsA<Widget>,
) -> Option<AABB> {
    let mins = {
        let coords = widget.translate_coordinates(dest_widget, aabb.mins[0], aabb.mins[1])?;
        na::point![coords.0, coords.1]
    };
    let maxs = {
        let coords = widget.translate_coordinates(dest_widget, aabb.maxs[0], aabb.maxs[1])?;
        na::point![coords.0, coords.1]
    };
    Some(AABB::new(mins, maxs))
}

pub fn replace_file_async(bytes: Vec<u8>, file: &gio::File) -> Result<(), anyhow::Error> {
    file.replace_async(
        None,
        false,
        gio::FileCreateFlags::REPLACE_DESTINATION,
        glib::PRIORITY_HIGH_IDLE,
        None::<&gio::Cancellable>,
        move |result| {
            let output_stream = match result {
                Ok(output_stream) => output_stream,
                Err(e) => {
                    log::error!(
                        "replace_async() failed in save_sheet_to_file() with Err {}",
                        e
                    );
                    return;
                }
            };

            if let Err(e) = output_stream.write(&bytes, None::<&gio::Cancellable>) {
                log::error!(
                    "output_stream().write() failed in save_sheet_to_file() with Err {}",
                    e
                );
            };
            if let Err(e) = output_stream.close(None::<&gio::Cancellable>) {
                log::error!(
                    "output_stream().close() failed in save_sheet_to_file() with Err {}",
                    e
                );
            };
        },
    );

    Ok(())
}
