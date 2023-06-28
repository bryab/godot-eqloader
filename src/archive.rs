use crate::util::sound::sound_from_bytes;
use crate::util::texture::tex_from_bmp;
use crate::wld::S3DWld;
use godot::engine::{AudioStreamWav, ImageTexture, RefCounted};
use godot::prelude::*;
use libeq::archive::EqArchiveReader;
use std::fs::File;
#[derive(GodotClass)]
#[class(base=RefCounted)]
pub struct EQArchive {
    #[base]
    base: Base<RefCounted>,
    archive: Option<EqArchiveReader>,
}

#[godot_api]
impl EQArchive {
    /// Initializer to be called by factory
    /// Not possible to initialize in GDScript
    pub fn load(&mut self, filename: &str) {
        godot_print!("Loading archive: {0}", &filename);
        let file = File::open(&filename)
            .map_err(|e| godot_error!("Failed to open archive: {filename}: {e}"))
            .unwrap();
        self.archive = Some(
            EqArchiveReader::new(file)
                .map_err(|e| godot_error!("Failed to parse S3D archive: {filename}: {e:?}"))
                .unwrap(),
        );
    }
    /// Returns a list of all filenames within the archive.
    #[func]
    pub fn get_filenames(&mut self) -> PackedStringArray {
        self.archive
            .as_ref()
            .expect("The load() method must be called to initialize this class.")
            .filenames
            .iter()
            .map(|s| GodotString::from(s))
            .collect()
    }

    /// Returns a Texture2D representation of the given bitmap filename
    #[func]
    pub fn get_texture(&self, filename: GodotString) -> Option<Gd<ImageTexture>> {
        let data = self.get(filename.to_string().as_str())?;
        tex_from_bmp(data)
            .map_err(|e| {
                godot_error!("Failed to load image from {filename}: {e}");
            })
            .ok()
    }

    /// Returns a Sound representation of the given audio filename (WAV)
    #[func]
    pub fn get_sound(&self, filename: GodotString) -> Option<Gd<AudioStreamWav>> {
        let data = self.get(filename.to_string().as_str())?;
        sound_from_bytes(data)
            .map_err(|e| {
                godot_error!("Failed to load audio from {filename}: {e}");
            })
            .ok()
    }

    /// Returns an EQWld object representing a WLD file
    #[func]
    pub fn get_wld(&self, filename: GodotString) -> Option<Gd<S3DWld>> {
        let data = self.get(filename.to_string().as_str())?;
        let mut wld: Gd<S3DWld> = Gd::new_default();
        wld.bind_mut().load(data);
        Some(wld)
    }

    // FIXME: It appears that I must return an empty array on error (I cannot return None)
    /// Returns a raw bytes representation of the given file
    #[func]
    pub fn get_bytes(&self, filename: GodotString) -> PackedByteArray {
        let data = self
            .get(filename.to_string().as_str())
            .or_else(|| Some(vec![]))
            .unwrap();
        PackedByteArray::from(data.as_slice())
    }

    /// Attempt to get the given data from the archive.
    /// An error is printed in Godot if the file does not exist.
    fn get(&self, filename: &str) -> Option<Vec<u8>> {
        self.archive
            .as_ref()
            .expect("The load() method must be called to initialize this class.")
            .get(filename)
            .map_err(|e| godot_error!("Failed to get {filename} from archive: {e:?}"))
            .ok()
    }
}

#[godot_api]
impl RefCountedVirtual for EQArchive {
    fn init(base: Base<RefCounted>) -> Self {
        EQArchive {
            base,
            archive: None,
        }
    }
}
