//! The transcription models SimpleEffect offers to download, the voice detection model, and which
//! model suits a machine. Pure data and decisions.
//!
//! Models come from the official ggml repositories of whisper.cpp on Hugging Face, pinned to a
//! revision so the SHA-256 never changes under us (ADR-0031). To add or update one: take `size`
//! and `lfs.oid` (the SHA-256) from `https://huggingface.co/api/models/<repo>/tree/<revision>`.

/// A downloadable file: where it comes from and what it must be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CatalogueFile {
    /// File name in the models folder (and in the repository).
    pub file_name: &'static str,
    /// Size in bytes.
    pub size: u64,
    /// SHA-256, lowercase hex.
    pub sha256: &'static str,
    /// Pinned `https://` address.
    pub url: &'static str,
}

/// A transcription model of the catalogue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CatalogueModel {
    /// Stable id (`base`), used in preferences and transcripts.
    pub id: &'static str,
    /// The file.
    pub file: CatalogueFile,
    /// whisper.cpp's alignment heads preset for better word timings (`--dtw`).
    pub alignment: &'static str,
}

/// The models offered, smallest first.
pub const MODELS: [CatalogueModel; 5] = [
    CatalogueModel {
        id: "tiny",
        file: CatalogueFile {
            file_name: "ggml-tiny.bin",
            size: 77_691_713,
            sha256: "be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21",
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-tiny.bin",
        },
        alignment: "tiny",
    },
    CatalogueModel {
        id: "base",
        file: CatalogueFile {
            file_name: "ggml-base.bin",
            size: 147_951_465,
            sha256: "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-base.bin",
        },
        alignment: "base",
    },
    CatalogueModel {
        id: "small",
        file: CatalogueFile {
            file_name: "ggml-small.bin",
            size: 487_601_967,
            sha256: "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b",
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-small.bin",
        },
        alignment: "small",
    },
    CatalogueModel {
        id: "medium",
        file: CatalogueFile {
            file_name: "ggml-medium.bin",
            size: 1_533_763_059,
            sha256: "6c14d5adee5f86394037b4e4e8b59f1673b6cee10e3cf0b11bbdbee79c156208",
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-medium.bin",
        },
        alignment: "medium",
    },
    CatalogueModel {
        id: "large-v3-turbo",
        file: CatalogueFile {
            file_name: "ggml-large-v3-turbo.bin",
            size: 1_624_555_275,
            sha256: "1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69",
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-large-v3-turbo.bin",
        },
        alignment: "large.v3.turbo",
    },
];

/// The voice detection model (Silero VAD for whisper.cpp).
pub const VOICE_MODEL: CatalogueFile = CatalogueFile {
    file_name: "ggml-silero-v6.2.0.bin",
    size: 885_098,
    sha256: "2aa269b785eeb53a82983a20501ddf7c1d9c48e33ab63a41391ac6c9f7fb6987",
    url: "https://huggingface.co/ggml-org/whisper-vad/resolve/9ffd54a1e1ee413ddf265af9913beaf518d1639b/ggml-silero-v6.2.0.bin",
};

/// The catalogue model with this id.
#[must_use]
pub fn model(id: &str) -> Option<&'static CatalogueModel> {
    MODELS.iter().find(|model| model.id == id)
}

/// The catalogue model with this SHA-256 (an imported file that is in fact one of ours).
#[must_use]
pub fn model_with_sha256(sha256: &str) -> Option<&'static CatalogueModel> {
    MODELS.iter().find(|model| model.file.sha256 == sha256)
}

/// Whether `file_name` belongs to the catalogue (a model or the voice model).
#[must_use]
pub fn is_catalogue_file(file_name: &str) -> bool {
    file_name == VOICE_MODEL.file_name
        || MODELS.iter().any(|model| model.file.file_name == file_name)
}

/// What decides which model suits a machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hardware {
    /// Whether a GPU whisper.cpp can use is present (now, or once the GPU pack is installed).
    pub gpu: bool,
    /// Logical processors.
    pub cpu_threads: usize,
}

/// The model to suggest: `large-v3-turbo` with a GPU (about 40× real time on a mid-range GPU),
/// `small` on a processor with 8 threads or more, `base` otherwise.
#[must_use]
pub fn recommended_model(hardware: Hardware) -> &'static str {
    if hardware.gpu {
        "large-v3-turbo"
    } else if hardware.cpu_threads >= 8 {
        "small"
    } else {
        "base"
    }
}

/// Catalogue models to try, best first, when the user chose none: the recommended one, then the
/// others from the most to the least accurate on a GPU, or from the fastest useful on a processor.
#[must_use]
pub fn preference_order(hardware: Hardware) -> [&'static str; 5] {
    if hardware.gpu {
        ["large-v3-turbo", "medium", "small", "base", "tiny"]
    } else if hardware.cpu_threads >= 8 {
        ["small", "base", "medium", "large-v3-turbo", "tiny"]
    } else {
        ["base", "small", "tiny", "medium", "large-v3-turbo"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODEL_REPOSITORY: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/";

    #[test]
    fn every_model_has_a_pinned_https_address_and_a_sha256() {
        for model in &MODELS {
            assert!(
                model.file.url.starts_with(MODEL_REPOSITORY)
                    && model.file.url.ends_with(model.file.file_name),
                "{}",
                model.id
            );
            assert_eq!(model.file.sha256.len(), 64);
            assert!(
                model
                    .file
                    .sha256
                    .chars()
                    .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
            );
        }
        assert!(VOICE_MODEL.url.starts_with("https://"));
        assert!(is_catalogue_file("ggml-base.bin") && is_catalogue_file(VOICE_MODEL.file_name));
        assert!(!is_catalogue_file("my-french.bin"));
    }

    #[test]
    fn recommends_turbo_with_a_gpu_and_small_models_on_processors() {
        let gpu = Hardware {
            gpu: true,
            cpu_threads: 4,
        };
        let strong = Hardware {
            gpu: false,
            cpu_threads: 16,
        };
        let weak = Hardware {
            gpu: false,
            cpu_threads: 4,
        };
        assert_eq!(recommended_model(gpu), "large-v3-turbo");
        assert_eq!(recommended_model(strong), "small");
        assert_eq!(recommended_model(weak), "base");
        for hardware in [gpu, strong, weak] {
            assert_eq!(preference_order(hardware)[0], recommended_model(hardware));
            assert!(
                preference_order(hardware)
                    .iter()
                    .all(|id| model(id).is_some())
            );
        }
        assert_eq!(
            model_with_sha256(MODELS[1].file.sha256).map(|m| m.id),
            Some("base")
        );
    }
}
