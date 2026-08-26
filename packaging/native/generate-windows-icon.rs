use std::{env, fs, io::Write, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = env::args_os().skip(1);
    let input = PathBuf::from(arguments.next().ok_or("indica el PNG 256x256")?);
    let output = PathBuf::from(arguments.next().ok_or("indica el ICO de salida")?);
    let png = fs::read(input)?;
    let size = u32::try_from(png.len())?;
    let mut icon = fs::File::create(output)?;
    icon.write_all(&[0, 0, 1, 0, 1, 0])?;
    icon.write_all(&[0, 0, 0, 0])?;
    icon.write_all(&1_u16.to_le_bytes())?;
    icon.write_all(&32_u16.to_le_bytes())?;
    icon.write_all(&size.to_le_bytes())?;
    icon.write_all(&22_u32.to_le_bytes())?;
    icon.write_all(&png)?;
    Ok(())
}
