pub fn utf8_to_u32(utf8: Vec<u8>) -> anyhow::Result<u32> {
    let string = String::from_utf8(utf8)?;
    let value = string[0..string.len() - 1].trim().parse()?;
    Ok(value)
}
