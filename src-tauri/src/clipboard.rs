use std::{borrow::Cow, path::Path};

use arboard::{Clipboard, ImageData};
use sha2::{Digest, Sha256};

use crate::{
  error::{AppError, AppResult},
  models::{ClipboardEntry, ClipboardPayload, ImagePayload}
};

pub fn read_payload() -> AppResult<Option<ClipboardPayload>> {
  let mut clipboard = Clipboard::new().map_err(|err| AppError::Clipboard(err.to_string()))?;

  match clipboard.get_image() {
    Ok(image) => {
      return Ok(Some(ClipboardPayload::Image(ImagePayload {
        width: image.width,
        height: image.height,
        bytes: image.bytes.into_owned()
      })));
    }
    Err(err) => {
      log::debug!("arboard image read skipped: {err}");
    }
  }

  #[cfg(target_os = "macos")]
  if let Some(image) = read_payload_from_macos_pasteboard()? {
    return Ok(Some(ClipboardPayload::Image(image)));
  }

  if let Ok(text) = clipboard.get_text() {
    if !text.trim().is_empty() {
      return Ok(Some(ClipboardPayload::Text(text)));
    }
  }

  Ok(None)
}

#[cfg(target_os = "macos")]
fn read_payload_from_macos_pasteboard() -> AppResult<Option<ImagePayload>> {
  use image::ImageFormat;
  use objc::{class, msg_send, runtime::Object, sel, sel_impl};
  use std::slice;

  unsafe {
    let pasteboard: *mut Object = msg_send![class!(NSPasteboard), generalPasteboard];
    if pasteboard.is_null() {
      return Ok(None);
    }

    let png_type = nsstring_from_str("public.png");
    let tiff_type = nsstring_from_str("public.tiff");

    let png_data: *mut Object = msg_send![pasteboard, dataForType: png_type];
    let tiff_data: *mut Object = msg_send![pasteboard, dataForType: tiff_type];

    release_nsobject(png_type);
    release_nsobject(tiff_type);

    let (raw, format) = if !png_data.is_null() {
      (png_data, ImageFormat::Png)
    } else if !tiff_data.is_null() {
      (tiff_data, ImageFormat::Tiff)
    } else {
      return Ok(None);
    };

    let length: usize = msg_send![raw, length];
    if length == 0 {
      return Ok(None);
    }

    let bytes_ptr: *const u8 = msg_send![raw, bytes];
    if bytes_ptr.is_null() {
      return Ok(None);
    }

    let bytes = slice::from_raw_parts(bytes_ptr, length);
    let decoded = image::load_from_memory_with_format(bytes, format).map_err(|err| {
      AppError::Clipboard(format!("failed to decode macOS pasteboard image: {err}"))
    })?;
    let rgba = decoded.to_rgba8();
    let (width, height) = rgba.dimensions();

    Ok(Some(ImagePayload {
      width: width as usize,
      height: height as usize,
      bytes: rgba.into_raw()
    }))
  }
}

#[cfg(target_os = "macos")]
const NS_UTF8_ENCODING: usize = 4;

#[cfg(target_os = "macos")]
unsafe fn nsstring_from_str(value: &str) -> *mut objc::runtime::Object {
  use std::ffi::c_void;

  use objc::{class, msg_send, runtime::Object, sel, sel_impl};

  let ns_string: *mut Object = msg_send![class!(NSString), alloc];
  msg_send![
    ns_string,
    initWithBytes: value.as_ptr() as *const c_void
    length: value.len()
    encoding: NS_UTF8_ENCODING
  ]
}

#[cfg(target_os = "macos")]
unsafe fn release_nsobject(object: *mut objc::runtime::Object) {
  use objc::{msg_send, sel, sel_impl};

  if !object.is_null() {
    let _: () = msg_send![object, release];
  }
}

pub fn write_entry_to_clipboard(entry: &ClipboardEntry) -> AppResult<()> {
  let mut clipboard = Clipboard::new().map_err(|err| AppError::Clipboard(err.to_string()))?;

  match entry.kind {
    crate::models::EntryKind::Text => {
      let text = entry
        .text_content
        .as_ref()
        .or(entry.text_preview.as_ref())
        .ok_or_else(|| AppError::Clipboard("missing text content".to_string()))?;

      clipboard
        .set_text(text.to_owned())
        .map_err(|err| AppError::Clipboard(err.to_string()))?;
    }
    crate::models::EntryKind::Image => {
      let image_path = entry
        .image_path
        .as_ref()
        .ok_or_else(|| AppError::Clipboard("missing image path".to_string()))?;

      let image = image::open(Path::new(image_path))
        .map_err(|err| AppError::Clipboard(format!("failed to open image: {err}")))?
        .to_rgba8();

      let (width, height) = image.dimensions();
      let bytes = image.into_raw();
      clipboard
        .set_image(ImageData {
          width: width as usize,
          height: height as usize,
          bytes: Cow::Owned(bytes)
        })
        .map_err(|err| AppError::Clipboard(err.to_string()))?;
    }
  }

  Ok(())
}

pub fn payload_hash(payload: &ClipboardPayload) -> String {
  let mut hasher = Sha256::new();

  match payload {
    ClipboardPayload::Text(text) => {
      hasher.update(b"text:");
      hasher.update(text.as_bytes());
    }
    ClipboardPayload::Image(image) => {
      hasher.update(b"image:");
      hasher.update(image.width.to_le_bytes());
      hasher.update(image.height.to_le_bytes());
      hasher.update(&image.bytes);
    }
  }

  hex::encode(hasher.finalize())
}
