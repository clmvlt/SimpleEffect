//! Services of the [`super::MediaTestbed`] built over the projects and media services: speech
//! (smart cuts, captions), libraries (templates, brand kits), reframing and exports.

use std::sync::Arc;

use super::{
    FakeExportOutputs, FakeFrameSampler, FakeMedia, FakeRendererLink, FakeSubjectDetector,
    FakeSubtitleFiles, InMemoryAppLibrary, InMemoryBrandKitLibrary, InMemoryTemplateLibrary,
    media_testbed::Runtime,
};
use crate::{
    BrandKitService, BrandKitServiceDeps, CaptionService, CaptionServiceDeps, ExportRegistry,
    ExportService, ExportServiceDeps, JobQueue, MediaService, ProjectService, ReframeService,
    ReframeServiceDeps, SmartCutService, SmartCutServiceDeps, TemplateCheckService,
    TemplateLibraryDeps, TemplateLibraryService, TranscriptionService,
};

/// The export folder proposed by the testbed's export service.
pub const TESTBED_EXPORTS_DIR: &str = if cfg!(windows) {
    r"C:\exports"
} else {
    "/exports"
};

/// Exports over fake outputs, checked by the fake prober, drawn by the fake renderer.
pub(super) fn export_service(
    projects: &Arc<ProjectService>,
    registry: Arc<ExportRegistry>,
    renderer: &Arc<FakeRendererLink>,
    files: &Arc<FakeMedia>,
    library: &Arc<InMemoryAppLibrary>,
    runtime: &Runtime,
) -> (ExportService, FakeExportOutputs) {
    let outputs = FakeExportOutputs::default();
    let exports = ExportService::new(ExportServiceDeps {
        registry,
        projects: projects.clone(),
        outputs: Arc::new(outputs.clone()),
        prober: files.clone(),
        renderer: renderer.clone(),
        library: library.clone(),
        events: runtime.events.clone(),
        clock: runtime.clock.clone(),
        ids: runtime.ids.clone(),
        default_directory: TESTBED_EXPORTS_DIR.into(),
    });
    (exports, outputs)
}

/// Automatic reframing over fake sampling and detection (prompt 26).
pub(super) fn reframing_service(
    projects: &Arc<ProjectService>,
    media: &Arc<MediaService>,
    jobs: JobQueue,
    runtime: &Runtime,
) -> (ReframeService, FakeFrameSampler, FakeSubjectDetector) {
    let (sampler, detector) = (FakeFrameSampler::default(), FakeSubjectDetector::default());
    let reframing = ReframeService::new(ReframeServiceDeps {
        projects: projects.clone(),
        media: media.clone(),
        jobs,
        sampler: Arc::new(sampler.clone()),
        detector: Arc::new(detector.clone()),
        events: runtime.events.clone(),
        clock: runtime.clock.clone(),
    });
    (reframing, sampler, detector)
}

/// The template library and brand kit services over in-memory libraries (prompt 25).
pub(super) fn library_services(
    projects: &Arc<ProjectService>,
    media: &Arc<MediaService>,
    files: &Arc<FakeMedia>,
    checks: &Arc<TemplateCheckService>,
    runtime: &Runtime,
) -> (
    InMemoryTemplateLibrary,
    TemplateLibraryService,
    InMemoryBrandKitLibrary,
    BrandKitService,
) {
    let template_library = InMemoryTemplateLibrary::default();
    let library_templates = TemplateLibraryService::new(TemplateLibraryDeps {
        library: Arc::new(template_library.clone()),
        projects: projects.clone(),
        checks: checks.clone(),
        events: runtime.events.clone(),
        clock: runtime.clock.clone(),
        ids: runtime.ids.clone(),
    });
    let brand_kit_library = InMemoryBrandKitLibrary::writing_to(files.clone());
    let brand_kits = BrandKitService::new(BrandKitServiceDeps {
        library: Arc::new(brand_kit_library.clone()),
        projects: projects.clone(),
        media: media.clone(),
        prober: files.clone(),
        events: runtime.events.clone(),
        clock: runtime.clock.clone(),
        ids: runtime.ids.clone(),
    });
    (
        template_library,
        library_templates,
        brand_kit_library,
        brand_kits,
    )
}

/// Smart cuts and captions (over in-memory SRT files).
pub(super) fn speech_services(
    projects: &Arc<ProjectService>,
    media: &Arc<MediaService>,
    transcription: &TranscriptionService,
    library: &Arc<InMemoryAppLibrary>,
) -> (SmartCutService, CaptionService, FakeSubtitleFiles) {
    let smart_cuts = SmartCutService::new(SmartCutServiceDeps {
        projects: projects.clone(),
        media: media.clone(),
        transcription: transcription.clone(),
        library: library.clone(),
    });
    let subtitles = FakeSubtitleFiles::default();
    let captions = CaptionService::new(CaptionServiceDeps {
        projects: projects.clone(),
        transcription: transcription.clone(),
        subtitles: Arc::new(subtitles.clone()),
    });
    (smart_cuts, captions, subtitles)
}
