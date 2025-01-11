use crate::util::sound::sound_from_bytes;
use crate::util::texture::{image_from_bmp, tex_from_bmp};
#[cfg(feature = "dds")]
use crate::util::texture::image_from_dds;
use crate::wld::S3DWld;
use libeq_archive::EqArchive;
use godot::classes::{AudioStreamWav, ImageTexture, RefCounted, Image};
use godot::prelude::*;
use std::fs::File;
use std::path::Path;
use std::ffi::OsStr;

#[derive(GodotClass)]
#[class(init)]
pub struct EQArchive {
    base: Base<RefCounted>,
    archive: Option<EqArchive>,
    /// The file stem of the archive, e.g. "rivervale".  This is used to get the main WLD out of the archive without specifying its name.
    name: String,
}

#[godot_api]
impl EQArchive {
    /// Returns a list of all filenames within the archive.
    #[func]
    pub fn get_filenames(&mut self) -> PackedStringArray {
        self.archive
            .as_ref()
            .expect("The load() method must be called to initialize this class.")
            .iter()
            .map(|(s,_)| GString::from(s))
            .collect()
    }

    /// Returns a Texture2D representation of the given bitmap filename
    #[func]
    pub fn get_texture(&self, filename: GString) -> Option<Gd<ImageTexture>> {
        let data = self._get(filename.to_string().as_str())?;
        tex_from_bmp(data)
            .map_err(|e| {
                godot_error!("Failed to load image from {filename}: {e}");
            })
            .ok()
    }

    /// Returns a Image representation of the given bitmap filename
    #[func]
    pub fn get_image(&self, filename: GString) -> Option<Gd<Image>> {
        let data = self._get(filename.to_string().as_str())?;
        match Path::new(filename.to_string().as_str()).extension().and_then(OsStr::to_str).expect("Filename should have extension") {
            "bmp" => {
                image_from_bmp(data)
                .map_err(|e| {
                    godot_error!("Failed to load image from {filename}: {e}");
                })
                .ok()
            },
            #[cfg(feature = "dds")]
            "dds" => {
                image_from_dds(data)
                .map_err(|e| {
                    godot_error!("Failed to load image from {filename}: {e}");
                })
                .ok()
            }
            _ => {
                godot_error!("Unsupported image format: {filename}");
                None
            }
        }        
    }

    /// Returns a Sound representation of the given audio filename (WAV)
    #[func]
    pub fn get_sound(&self, filename: GString) -> Option<Gd<AudioStreamWav>> {
        let data = self._get(filename.to_string().as_str())?;
        sound_from_bytes(data)
            .map_err(|e| {
                godot_error!("Failed to load audio from {filename}: {e}");
            })
            .ok()
    }

    /// Returns an EQWld object representing a WLD file
    #[func]
    pub fn get_wld(&self, filename: GString) -> Option<Gd<S3DWld>> {
        self._get_wld(filename.to_string().as_str())
    }

    /// Returns the main WLD inside the S3D file.
    /// For Zone S3Ds, this is the WLD containing the zone data.
    /// For ActorDef and Character S3Ds, this is the only WLD in the archive.
    #[func]
    pub fn get_main_wld(&self) -> Option<Gd<S3DWld>> {
        self._get_wld(&format!("{0}.wld", &self.name))
    }

    /// In Zone S3Ds, this will return the lights.wld within the archive.
    #[func]
    pub fn get_lights_wld(&self) -> Option<Gd<S3DWld>> {
        self._get_wld("lights.wld")
    }

    /// In Zone S3Ds, this will return the objects.wld within the archive.
    #[func]
    pub fn get_actorinst_wld(&self) -> Option<Gd<S3DWld>> {
        self._get_wld("objects.wld")
    }

    // FIXME: This should return Variant::nil() if the file does't exist.
    /// Returns a raw bytes representation of the given file
    #[func]
    pub fn get_bytes(&self, filename: GString) -> PackedByteArray {
        let data = self
            ._get(filename.to_string().as_str())
            .or_else(|| Some(vec![]))
            .unwrap();
        PackedByteArray::from(data.as_slice())
    }
}

impl EQArchive {
    /// Initializer to be called by factory
    /// Not possible to initialize in GDScript
    pub fn load(&mut self, filename: &str) {
        godot_print!("Loading archive: {0}", &filename);
        let file = File::open(&filename)
            .map_err(|e| godot_error!("Failed to open archive: {filename}: {e}"))
            .unwrap();

        self.archive = Some(
            EqArchive::read(file)
                .map_err(|e| godot_error!("Failed to parse S3D archive: {filename}: {e:?}"))
                .unwrap(),
        );
        self.name = String::from(Path::new(&filename).file_stem().unwrap().to_str().unwrap());
    }
    /// Attempt to get the given data from the archive.
    /// An error is printed in Godot if the file does not exist.
    fn _get(&self, filename: &str) -> Option<Vec<u8>> {
        self.archive
            .as_ref()
            .expect("The load() method must be called to initialize this class.")
            .iter()
            .find(|(name, _)| name == &filename)
            .or_else(|| { godot_error!("{filename} not found in from archive"); None })
            .and_then(|(_, data)| Some(data.clone()))
    }

    /// Returns an EQWld object representing a WLD file
    fn _get_wld(&self, filename: &str) -> Option<Gd<S3DWld>> {
        let data = self._get(filename)?;
        let mut wld: Gd<S3DWld> = Gd::default();
        wld.bind_mut().load(data);
        Some(wld)
    }
}
// #[godot_api]
// impl RefCountedVirtual for EQArchive {
//     fn init(base: Base<RefCounted>) -> Self {
//         EQArchive {
//             base,
//             archive: None,
//         }
//     }
// }
