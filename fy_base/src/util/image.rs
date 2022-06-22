use bytes::Bytes;
use image::io::Reader as ImageReader;

use std::io::Cursor;

pub fn check_bmp_magic(buf: &[u8]) -> bool {
    buf.len() >= 2 && buf[0] == 0x42 && buf[1] == 0x4d
}

pub fn escape_bmp_to_jpg(content: &[u8]) -> image::ImageResult<Bytes> {
    // 如果不是bmp，则跳过
    if !check_bmp_magic(content) {
        let buf = Bytes::from(Vec::from(content));
        return Ok(buf);
    }

    let img = ImageReader::new(Cursor::new(content))
        .with_guessed_format()?
        .decode()?;
    let mut buf = Vec::new();
    img.write_to(
        &mut Cursor::new(&mut buf),
        image::ImageOutputFormat::Jpeg(85),
    )?;
    Ok(Bytes::from(buf))
}

pub fn escape_bmp(content: Bytes) -> std::result::Result<Bytes, String> {
    if !check_bmp_magic(&content) {
        return Ok(content);
    }
    let img = match ImageReader::new(Cursor::new(content)).with_guessed_format() {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("{:?}", e));
        }
    };

    let img = match img.decode() {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("{:?}", e));
        }
    };

    let mut buf = Vec::new();
    match img.write_to(
        &mut Cursor::new(&mut buf),
        image::ImageOutputFormat::Jpeg(85),
    ) {
        Ok(_) => Ok(Bytes::from(buf)),
        Err(e) => {
            return Err(format!("{:?}", e));
        }
    }
}
