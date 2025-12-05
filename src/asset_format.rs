#![allow(unused_imports)]

use std::sync::LazyLock;

use bevy::asset::{
    AssetLoader, LoadContext,
    io::{Reader, VecReader},
};
use bevy::image::{
    CompressedImageFormats, ImageFormatSetting, ImageLoader, ImageLoaderError, ImageLoaderSettings,
};
use bevy::prelude::*;

use chacha20poly1305::{
    ChaCha8Poly1305, Key,
    aead::{Aead, AeadCore, KeyInit},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub(super) fn plugin(app: &mut App) {
    app.init_asset_loader::<EncryptedImageLoader>();
}

static KEY: LazyLock<[u8; 32]> = LazyLock::new(|| {
    let mut hasher = Sha256::new();
    hasher.update(env!("CARGO_PKG_NAME").as_bytes());
    hasher.update(
        "All paid assets are listed in src/menus/credits.rs, \
along with a URL where you can purchase them for yourself."
            .as_bytes(),
    );
    hasher.finalize().into()
});

#[derive(Error, Debug)]
pub enum AssetFormatError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("ChaCha error: {}", .0)]
    ChaChaError(chacha20poly1305::Error),
    #[error(transparent)]
    ImageLoader(#[from] ImageLoaderError),
    #[error(transparent)]
    DeserializeError(#[from] flexbuffers::DeserializationError),
    #[error(transparent)]
    SerializeError(#[from] flexbuffers::SerializationError),
}

impl From<chacha20poly1305::Error> for AssetFormatError {
    fn from(error: chacha20poly1305::Error) -> Self {
        AssetFormatError::ChaChaError(error)
    }
}

impl From<flexbuffers::ReaderError> for AssetFormatError {
    fn from(error: flexbuffers::ReaderError) -> Self {
        let error: flexbuffers::DeserializationError = error.into();
        error.into()
    }
}

#[derive(Reflect, Serialize, Deserialize, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct EncryptedAsset {
    #[serde(with = "serde_bytes")]
    nonce: [u8; 12],
    #[serde(with = "serde_bytes")]
    ciphertext: Vec<u8>,
}

impl EncryptedAsset {
    #[cfg(feature = "dev_native")]
    pub fn encrypt(key: Key, plaintext: &[u8]) -> Result<EncryptedAsset, AssetFormatError> {
        let cipher = ChaCha8Poly1305::new(&key);
        let nonce =
            ChaCha8Poly1305::generate_nonce().map_err(|e| std::io::Error::other(e.to_string()))?;
        let ciphertext = cipher.encrypt(&nonce, plaintext)?;
        let nonce = *nonce.as_ref();
        Ok(EncryptedAsset { ciphertext, nonce })
    }

    pub fn decrypt(&self, key: Key) -> Result<Vec<u8>, AssetFormatError> {
        let cipher = ChaCha8Poly1305::new(&key);
        let nonce = (&(self.nonce)).into();
        let decrypted = cipher.decrypt(nonce, self.ciphertext.as_ref())?;
        Ok(decrypted)
    }
}

#[derive(Default)]
pub struct EncryptedImageLoader;

impl AssetLoader for EncryptedImageLoader {
    type Asset = Image;
    type Settings = ImageLoaderSettings;
    type Error = AssetFormatError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &ImageLoaderSettings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Image, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let flex_reader = flexbuffers::Reader::get_root(bytes.as_slice())?;
        let encrypted = EncryptedAsset::deserialize(flex_reader)?;
        let decrypted = encrypted.decrypt((*KEY).into())?;
        let mut vec_reader = VecReader::new(decrypted);
        let image_loader = ImageLoader::new(CompressedImageFormats::NONE);
        let settings = ImageLoaderSettings {
            format: ImageFormatSetting::Guess,
            ..(settings.clone())
        };
        let asset = image_loader
            .load(&mut vec_reader, &settings, load_context)
            .await?;
        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &["epng"]
    }
}

/*
#[derive(Default)]
pub struct EncryptedAudioLoader;
*/

#[cfg(feature = "dev_native")]
pub fn encrypt_raw_assets(_: On<Pointer<Click>>) {
    if let Err(e) = asset_encryption::encrypt_assets() {
        error!("Failed to encrypt raw assets: {}", e);
    } else {
        info!("Successfully encrypted raw assets");
    }
}

#[cfg(not(feature = "dev_native"))]
pub fn encrypt_raw_assets(_: On<Pointer<Click>>) {
    ()
}

#[cfg(feature = "dev_native")]
mod asset_encryption {
    use super::*;
    use std::fs;
    use std::path::Path;

    pub fn encrypt_assets() -> Result<(), AssetFormatError> {
        let raw_assets_dir = Path::new("raw_assets");
        let assets_dir = Path::new("assets");
        process_directory(raw_assets_dir, assets_dir, raw_assets_dir)
    }

    fn process_directory(
        current_dir: &Path,
        assets_base: &Path,
        raw_assets_base: &Path,
    ) -> Result<(), AssetFormatError> {
        for entry in fs::read_dir(current_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively process subdirectories
                process_directory(&path, assets_base, raw_assets_base)?;
            } else if path.is_file() {
                encrypt_file(&path, assets_base, raw_assets_base)?;
            }
        }

        Ok(())
    }

    fn encrypt_file(
        source_path: &Path,
        assets_base: &Path,
        raw_assets_base: &Path,
    ) -> Result<(), AssetFormatError> {
        let relative_path = source_path
            .strip_prefix(raw_assets_base)
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        let mut dest_path = assets_base.join(relative_path);
        if let Some(ext) = dest_path.extension() {
            let new_ext = format!("e{}", ext.to_string_lossy());
            dest_path.set_extension(new_ext);
        }

        if dest_path.exists() {
            info!("Skipping {} (already exists)", dest_path.display());
            return Ok(());
        }

        // Create parent directories if needed
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let plaintext = fs::read(source_path)?;
        let encrypted = EncryptedAsset::encrypt((*KEY).into(), &plaintext)?;
        let mut serializer = flexbuffers::FlexbufferSerializer::new();
        encrypted.serialize(&mut serializer)?;
        let bytes = serializer.view();
        fs::write(&dest_path, bytes)?;

        info!(
            "Encrypted {} -> {}",
            source_path.display(),
            dest_path.display()
        );

        Ok(())
    }
}
