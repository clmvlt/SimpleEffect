//! What whisper-cli's log (stderr) tells while it runs: progress and where it computes.

use se_protocol::TranscriptionBackend;

/// The fraction of `whisper_print_progress_callback: progress =  42%`, if the line is one.
#[must_use]
pub fn progress(line: &str) -> Option<f64> {
    let rest = &line[line.find("progress =")? + "progress =".len()..];
    let percent: f64 = rest.trim().trim_end_matches('%').trim().parse().ok()?;
    Some((percent / 100.0).clamp(0.0, 1.0))
}

/// The backend a `whisper_backend_init_gpu: using CUDA0 backend` line names, if the line is one.
#[must_use]
pub fn backend(line: &str) -> Option<TranscriptionBackend> {
    let rest = &line[line.find("whisper_backend_init_gpu: using ")? + 32..];
    let device = rest.split_whitespace().next()?.to_ascii_lowercase();
    Some(if device.starts_with("cuda") {
        TranscriptionBackend::Cuda
    } else if device.starts_with("mtl") || device.starts_with("metal") {
        TranscriptionBackend::Metal
    } else if device.starts_with("vulkan") {
        TranscriptionBackend::Vulkan
    } else {
        TranscriptionBackend::Cpu
    })
}

/// The backend of a whole log: the GPU it used, else the processor.
#[must_use]
pub fn backend_of(log: &str) -> TranscriptionBackend {
    log.lines()
        .find_map(backend)
        .unwrap_or(TranscriptionBackend::Cpu)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_progress_and_backends() {
        assert_eq!(
            progress("whisper_print_progress_callback: progress =  45%"),
            Some(0.45)
        );
        assert_eq!(progress("whisper_model_load: n_vocab = 51865"), None);
        let log = "load_backend: loaded CUDA backend from C:\\x\\ggml-cuda.dll\n\
                   whisper_backend_init_gpu: found GPU device 0: CUDA0 (type: 1, cnt: 0)\n\
                   whisper_backend_init_gpu: using CUDA0 backend\n";
        assert_eq!(backend_of(log), TranscriptionBackend::Cuda);
        assert_eq!(
            backend("whisper_backend_init_gpu: using MTL0 backend"),
            Some(TranscriptionBackend::Metal)
        );
        assert_eq!(
            backend_of("whisper_backend_init_gpu: no GPU found"),
            TranscriptionBackend::Cpu
        );
    }
}
