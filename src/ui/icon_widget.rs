use crate::util::icons::find_forced_svg_path;
use gtk::prelude::*;
use std::path::Path;

const DEFAULT_HIDPI_MULTIPLIER: i32 = 2;

pub fn build_icon_image(icon_name: &str, pixel_size: i32) -> gtk::Image {
    if let Some(svg_path) = find_forced_svg_path(icon_name) {
        if let Some(image) = build_from_svg_path(&svg_path, pixel_size, DEFAULT_HIDPI_MULTIPLIER) {
            return image;
        }
    }

    let image = gtk::Image::from_icon_name(icon_name);
    image.set_pixel_size(pixel_size);
    image.set_use_fallback(true);
    image
}

pub fn build_icon_image_for_widget_scale(
    icon_name: &str,
    pixel_size: i32,
    scale_factor: i32,
) -> gtk::Image {
    let effective_scale = scale_factor.max(1);
    if let Some(svg_path) = find_forced_svg_path(icon_name) {
        if let Some(image) = build_from_svg_path(&svg_path, pixel_size, effective_scale) {
            return image;
        }
    }

    let image = gtk::Image::from_icon_name(icon_name);
    image.set_pixel_size(pixel_size);
    image.set_use_fallback(true);
    image
}

pub fn build_from_svg_path(path: &Path, pixel_size: i32, scale_factor: i32) -> Option<gtk::Image> {
    let render_size = pixel_size.saturating_mul(scale_factor.max(1));
    let pixbuf =
        gdk_pixbuf::Pixbuf::from_file_at_scale(path, render_size, render_size, true).ok()?;
    let texture = gdk::Texture::for_pixbuf(&pixbuf);
    let image = gtk::Image::from_paintable(Some(&texture));
    image.set_pixel_size(pixel_size);
    Some(image)
}
