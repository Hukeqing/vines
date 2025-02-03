use crate::common::{Error, FileNode, Res};
use image::ImageError;
use image::ImageFormat;
use std::io::{Cursor, Write};

pub fn get_size(data: &Vec<u8>) -> Res<(u32, u32)> {
    let image = image::load_from_memory(data).map_err(warp_e)?;
    Ok((image.width(), image.height()))
}

pub fn build_thumbnail(data: &Vec<u8>) -> Res<Vec<u8>> {
    let image = image::load_from_memory(data).map_err(warp_e)?;
    let thumbnail = image.thumbnail(300, 300);
    let mut buffer = Vec::new();
    let ref mut writer = Cursor::new(&mut buffer);
    thumbnail.write_to(writer, ImageFormat::Png).map_err(warp_e)?;
    Ok(buffer)
}

pub fn build_thumbnail_from_file(origin_file_node: &mut FileNode, target_file_node: &mut FileNode) -> Res<()> {
    let mut data = Vec::new();
    origin_file_node.read_all(&mut data)?;
    let thumbnail_data = build_thumbnail(&data)?;
    target_file_node.write(&thumbnail_data)?;
    Ok(())
}

fn warp_e(e: ImageError) -> Error {
    Error::ImageLoadError(e.to_string())
}
