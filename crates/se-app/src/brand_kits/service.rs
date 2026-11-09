//! [`BrandKitService`]: the brand kits of the library and the copies projects use (prompt 25).

use std::{path::Path, sync::Arc};

use se_domain::{AssetKind, BrandKitId, BrandLogo, ProjectId, SetBrandKit};
use se_protocol::{CoreEvent, LibraryBrandKit, LibraryLogo};

use super::{
    labels,
    library_kit::{check_library_kit, logo_file_name, new_library_kit, project_kit},
};
use crate::{
    AppError, MediaService, ProjectService,
    new_ids::NewIds,
    ports::{BrandKitLibrary, Clock, EventSink, IdGenerator, MediaProber},
};

/// Folder of a project receiving the copies of its kit's logos.
pub const PROJECT_LOGO_FOLDER: &str = "brand-kit";

/// What the brand kit use cases work with.
#[derive(Clone)]
pub struct BrandKitServiceDeps {
    /// The kits on disk.
    pub library: Arc<dyn BrandKitLibrary>,
    /// Open projects.
    pub projects: Arc<ProjectService>,
    /// Imports of logos as project assets.
    pub media: Arc<MediaService>,
    /// Checks that a logo is an image.
    pub prober: Arc<dyn MediaProber>,
    /// `libraryChanged` events.
    pub events: Arc<dyn EventSink>,
    /// Dates of kits.
    pub clock: Arc<dyn Clock>,
    /// Ids of new kits and logo files.
    pub ids: Arc<dyn IdGenerator>,
}

/// Brand kit use cases. Cheap to clone.
#[derive(Clone)]
pub struct BrandKitService {
    deps: BrandKitServiceDeps,
}

impl BrandKitService {
    /// The service over `deps`.
    #[must_use]
    pub const fn new(deps: BrandKitServiceDeps) -> Self {
        Self { deps }
    }

    /// Every kit, by name (case-insensitive).
    ///
    /// # Errors
    /// The library folder cannot be read.
    pub async fn list(&self) -> Result<Vec<LibraryBrandKit>, AppError> {
        let mut kits = self.deps.library.list().await?;
        kits.sort_by_cached_key(|kit| (kit.name.to_lowercase(), kit.id));
        Ok(kits)
    }

    /// The kit `id`.
    ///
    /// # Errors
    /// [`AppError::BrandKitNotFound`], storage errors.
    pub async fn kit(&self, id: BrandKitId) -> Result<LibraryBrandKit, AppError> {
        self.deps
            .library
            .get(id)
            .await?
            .ok_or(AppError::BrandKitNotFound { kit_id: id })
    }

    /// A new kit called `name` with default values, or a copy of kit `copy_of` (its logos copied
    /// too).
    ///
    /// # Errors
    /// An unusable name, an unknown kit to copy, storage errors.
    pub async fn create(
        &self,
        name: &str,
        copy_of: Option<BrandKitId>,
    ) -> Result<LibraryBrandKit, AppError> {
        let ids = NewIds(self.deps.ids.as_ref());
        let id = ids.brand_kit()?;
        let mut kit = new_library_kit(id, name, ids.caption_style()?, self.now())?;
        if let Some(source) = copy_of {
            let source = self.kit(source).await?;
            let mut logos = Vec::with_capacity(source.logos.len());
            for logo in &source.logos {
                let extension = Path::new(&logo.file)
                    .extension()
                    .and_then(|ext| ext.to_str());
                let file = logo_file_name(self.deps.ids.new_uuid_v7(), extension);
                let original = self.deps.library.logo_path(source.id, &logo.file);
                self.deps.library.import_logo(id, &original, &file).await?;
                logos.push(LibraryLogo {
                    name: logo.name.clone(),
                    file,
                });
            }
            kit = LibraryBrandKit {
                id,
                version: 1,
                updated_at: kit.updated_at,
                name: kit.name,
                logos,
                ..source
            };
        }
        check_library_kit(&kit, &kit.logos)?;
        self.deps.library.save(&kit).await?;
        self.changed();
        Ok(kit)
    }

    /// Saves `kit` as a new version (its logos must be files the kit already holds) and gives
    /// the open projects using it the new values at once (one undoable step « Update brand kit »
    /// each).
    ///
    /// # Errors
    /// Unknown kit, unusable values, storage errors; the update of an open project.
    pub async fn save(&self, kit: &LibraryBrandKit) -> Result<LibraryBrandKit, AppError> {
        let stored = self.kit(kit.id).await?;
        check_library_kit(kit, &stored.logos)?;
        let saved = LibraryBrandKit {
            version: stored.version.saturating_add(1),
            updated_at: self.now(),
            name: kit.name.trim().to_owned(),
            ..kit.clone()
        };
        self.store_and_sync(&saved).await?;
        Ok(saved)
    }

    /// Copies the image at `path` into kit `id` as a new logo (named after the file) and saves
    /// the kit.
    ///
    /// # Errors
    /// Unknown kit, a file that is not an image, too many logos, storage errors.
    pub async fn add_logo(&self, id: BrandKitId, path: &Path) -> Result<LibraryBrandKit, AppError> {
        let kit = self.kit(id).await?;
        let probed = self
            .deps
            .prober
            .probe(path)
            .await
            .map_err(|error| invalid_logo(path, &error.to_string()))?;
        if probed.kind != AssetKind::Image {
            return Err(invalid_logo(
                path,
                "it is not an image; choose a PNG, JPEG or WebP file",
            ));
        }
        let extension = path.extension().and_then(|ext| ext.to_str());
        let file = logo_file_name(self.deps.ids.new_uuid_v7(), extension);
        self.deps.library.import_logo(id, path, &file).await?;
        let name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map_or_else(|| "Logo".to_owned(), str::to_owned);
        let mut logos = kit.logos.clone();
        logos.push(LibraryLogo { name, file });
        let updated = LibraryBrandKit {
            logos: logos.clone(),
            ..kit.clone()
        };
        check_library_kit(&updated, &logos)?;
        let saved = LibraryBrandKit {
            version: kit.version.saturating_add(1),
            updated_at: self.now(),
            ..updated
        };
        self.store_and_sync(&saved).await?;
        Ok(saved)
    }

    /// Moves kit `id` to the system trash; projects keep their copies.
    ///
    /// # Errors
    /// Unknown kit, storage errors.
    pub async fn remove(&self, id: BrandKitId) -> Result<(), AppError> {
        self.kit(id).await?;
        self.deps.library.remove(id).await?;
        self.changed();
        Ok(())
    }

    /// Gives project `project_id` a copy of kit `kit_id` (its logos copied into the project's
    /// folder and imported as assets), or removes its kit (one undoable step).
    ///
    /// # Errors
    /// Not open, unknown kit, a logo that cannot be imported, storage errors.
    pub async fn set_project_kit(
        &self,
        project_id: ProjectId,
        kit_id: Option<BrandKitId>,
    ) -> Result<(), AppError> {
        match kit_id {
            Some(kit_id) => {
                let kit = self.kit(kit_id).await?;
                self.apply_to_project(project_id, &kit).await
            }
            None => {
                self.deps
                    .projects
                    .plan_user_edit(project_id, labels::REMOVE_BRAND_KIT, |plan| {
                        plan.push(SetBrandKit { brand_kit: None });
                        Ok(())
                    })
                    .await
            }
        }
    }

    /// Adds the kit of project `project_id`, made elsewhere (no library source, or a source this
    /// library lacks), to the library — its logos copied from their assets' files — and links
    /// the project to it.
    ///
    /// # Errors
    /// Not open, a project without kit, a logo file that cannot be read, storage errors.
    pub async fn add_project_kit_to_library(
        &self,
        project_id: ProjectId,
    ) -> Result<LibraryBrandKit, AppError> {
        let project = self.deps.projects.project(project_id).await?;
        let copy = project
            .brand_kit
            .clone()
            .ok_or_else(|| AppError::InvalidEdit {
                reason: "this project has no brand kit to add to the library".to_owned(),
            })?;
        let ids = NewIds(self.deps.ids.as_ref());
        let id = ids.brand_kit()?;
        let mut logos = Vec::with_capacity(copy.logos.len());
        for logo in &copy.logos {
            let asset = project
                .assets
                .iter()
                .find(|asset| asset.id == logo.asset_id)
                .ok_or(AppError::AssetNotFound {
                    project_id,
                    asset_id: logo.asset_id,
                })?;
            let original = Path::new(&asset.original_path);
            let extension = original.extension().and_then(|ext| ext.to_str());
            let file = logo_file_name(self.deps.ids.new_uuid_v7(), extension);
            self.deps.library.import_logo(id, original, &file).await?;
            logos.push(LibraryLogo {
                name: logo.name.clone(),
                file,
            });
        }
        let kit = LibraryBrandKit {
            id,
            version: 1,
            updated_at: self.now(),
            name: copy.name.clone(),
            fonts: copy.fonts.clone(),
            colors: copy.colors,
            logos,
            text_style: copy.text_style.clone(),
            caption_style: copy.caption_style.clone(),
            intro_templates: copy.intro_templates.clone(),
            outro_templates: copy.outro_templates.clone(),
        };
        check_library_kit(&kit, &kit.logos)?;
        self.deps.library.save(&kit).await?;
        self.changed();
        let linked = se_domain::BrandKit {
            source: Some(se_domain::BrandKitSource {
                kit_id: id,
                version: 1,
            }),
            ..copy
        };
        self.deps
            .projects
            .plan_user_edit(project_id, labels::UPDATE_BRAND_KIT, |plan| {
                plan.push(SetBrandKit {
                    brand_kit: Some(linked),
                });
                Ok(())
            })
            .await?;
        Ok(kit)
    }

    /// Writes `kit`, then updates every open project using it.
    async fn store_and_sync(&self, kit: &LibraryBrandKit) -> Result<(), AppError> {
        self.deps.library.save(kit).await?;
        self.changed();
        for project_id in self.deps.projects.open_project_ids().await {
            let project = self.deps.projects.project(project_id).await?;
            let uses_it = project
                .brand_kit
                .as_ref()
                .and_then(|copy| copy.source)
                .is_some_and(|source| source.kit_id == kit.id);
            if uses_it {
                self.apply_to_project(project_id, kit).await?;
            }
        }
        Ok(())
    }

    /// Copies `kit`'s logos into the project and imports them, then sets the project's copy of
    /// the kit (« Choose brand kit », or « Update brand kit » when it already used this kit).
    async fn apply_to_project(
        &self,
        project_id: ProjectId,
        kit: &LibraryBrandKit,
    ) -> Result<(), AppError> {
        let folder = self.deps.projects.project_folder(project_id).await?;
        let mut logos = Vec::with_capacity(kit.logos.len());
        for logo in &kit.logos {
            let file_name = Path::new(&logo.file)
                .file_name()
                .map(ToOwned::to_owned)
                .unwrap_or_default();
            let destination = folder
                .join(PROJECT_LOGO_FOLDER)
                .join(kit.id.to_string())
                .join(file_name);
            self.deps
                .library
                .copy_logo(kit.id, &logo.file, &destination)
                .await?;
            let asset_id = self.import_logo(project_id, &destination).await?;
            logos.push(BrandLogo {
                name: logo.name.clone(),
                asset_id,
            });
        }
        let copy = project_kit(kit, logos);
        let project = self.deps.projects.project(project_id).await?;
        let same_source = project
            .brand_kit
            .as_ref()
            .and_then(|current| current.source)
            .is_some_and(|source| source.kit_id == kit.id);
        let label = if same_source {
            labels::UPDATE_BRAND_KIT
        } else {
            labels::CHOOSE_BRAND_KIT
        };
        if project.brand_kit.as_ref() == Some(&copy) {
            return Ok(());
        }
        self.deps
            .projects
            .plan_user_edit(project_id, label, |plan| {
                plan.push(SetBrandKit {
                    brand_kit: Some(copy),
                });
                Ok(())
            })
            .await
    }

    /// The asset of the image at `path` in the project: imported now, or the one already holding
    /// its content.
    async fn import_logo(
        &self,
        project_id: ProjectId,
        path: &Path,
    ) -> Result<se_domain::AssetId, AppError> {
        let report = self
            .deps
            .media
            .import_files(project_id, &[path.to_path_buf()])
            .await?;
        if let Some(asset) = report.imported.first() {
            return Ok(*asset);
        }
        if let Some(duplicate) = report.duplicates.first() {
            return Ok(duplicate.asset_id);
        }
        let reason = report.failures.first().map_or_else(
            || "it could not be imported".to_owned(),
            |failure| failure.message.clone(),
        );
        Err(invalid_logo(path, &reason))
    }

    fn now(&self) -> u64 {
        u64::try_from(self.deps.clock.now_unix_millis()).unwrap_or(0)
    }

    fn changed(&self) {
        self.deps.events.publish(CoreEvent::LibraryChanged {
            templates: false,
            brand_kits: true,
        });
    }
}

fn invalid_logo(path: &Path, reason: &str) -> AppError {
    AppError::InvalidLibraryFile {
        path: path.display().to_string(),
        reason: format!("this logo cannot be used: {reason}"),
    }
}
