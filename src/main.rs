use std::{fs::{self, File}, io::Write, path::{Path, PathBuf}};
use argh::FromArgs;
use color_eyre::eyre::{ContextCompat, OptionExt, Result};
use evbunpack_core::{pe::{PeVariant, RestorePeContext}, vfs::Unpacker};
use fmmap::{MmapFile, MmapFileExt, MmapFileMut, MmapFileMutExt};

#[derive(Debug, FromArgs)]
#[argh(help_triggers("-h", "--help"))]
/// Evbunpacker
struct Args {
    /// do not extract virtual filesystem
    #[argh(switch)]
    no_vfs: bool,
    /// do not restore PE
    #[argh(switch)]
    no_pe: bool,
    /// manually specify a PE variant (10_70, 9_70, 7_80)
    #[argh(option, from_str_fn(parse_variant))]
    variant: Option<PeVariant>,
    /// executable to unpack from
    #[argh(positional)]
    file: PathBuf,
    /// folder to write unpacked contents
    #[argh(positional)]
    output: Option<PathBuf>
}

fn parse_variant(val: &str) -> Result<PeVariant, String> {
    Ok(match val {
        "10_70" | "1070" => PeVariant::V10_70,
        "9_70" | "970" => PeVariant::V9_70,
        "7_80" | "780" => PeVariant::V7_80,
        _ => return Err("Invalid PE variant".to_string())
    })
}

fn unpack_vfs(data: &[u8], output: &Path) -> Result<()> {

    let unpacker = Unpacker::new(data)?;
    for (path, node) in unpacker.files() {
        let path = output.join(path);
        if node.is_folder {
            fs::create_dir_all(path)?;
            continue
        }

        let mut file = File::create(path)?;
        file.write_all(unpacker.get_file_data(node)?.unwrap().as_ref())?
    }

    Ok(())
}

fn restore_pe(data: &mut [u8], variant: Option<PeVariant>, output: &Path) -> Result<()> {
    let context = if let Some(variant) = variant { RestorePeContext::new(data).with_variant(variant)? } else { RestorePeContext::new(data).with_variant_auto().ok_or_eyre("Failed to detect the correct PE variant. Try specifying it manually")? };

    let end = context.restore_pe()?;
    let mut file = File::create(output)?;
    file.write_all(&data[..end])?;

    Ok(())
}

enum Mmap {
    Readonly(MmapFile),
    Writable(MmapFileMut)
}

impl Mmap {
    fn as_slice(&self) -> &[u8] {
        match self {
            Self::Readonly(file) => file.as_slice(),
            Self::Writable(file) => file.as_slice()
        }
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        match self {
            Self::Writable(file) => file.as_mut_slice(),
            _ => unreachable!("This method should not be called when mmap is readonly")
        }
    }
}

fn main() -> Result<()> {
    let Args { no_pe, no_vfs, output, file, variant } = argh::from_env();
    let output = output.unwrap_or_else(|| PathBuf::from("unpacked"));
    let mut mmap = if no_pe { Mmap::Readonly(MmapFile::open(&file)?) } else { Mmap::Writable(MmapFileMut::open_cow(&file)?) };

    if !no_vfs {
        unpack_vfs(mmap.as_slice(), &output)?
    }

    if !no_pe {
        let file_stem = file.file_stem().context("Failed to get file stem")?.to_str().context("Failed to convert file stem to str")?;
        let output_file_name = format!("{}_unpacked.exe", file_stem);
        let output = output.join(&output_file_name);
        restore_pe(mmap.as_mut_slice(), variant, &output)?;
    }

    Ok(())
}
