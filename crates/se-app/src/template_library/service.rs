//! [`TemplateLibraryService`]: the user's template library use cases (prompt 25).

use std::{collections::BTreeMap, path::Path, sync::Arc};

use se_domain::{
    CaptionStyleId, LibraryLink, LibraryTemplateId, MotionTemplate, Origin, ProjectId,
    ReplaceTemplate, TemplateId,
};
use se_protocol::{
    CoreEvent, InsertLibraryTemplateRequest, InsertedLibraryTemplate, LibraryContent,
    LibraryTemplate, LibraryTemplateInfo, TemplateUpdateReport,
};

use super::{
    archive_import::import_one, info::checked_info, labels, plan::plan_copy, plan::plan_update,
};
use crate::{
    AppError, ProjectService, TemplateCheckService,
    new_ids::NewIds,
    ports::{Clock, EventSink, IdGenerator, TemplateLibrary},
    project_service::{graphic_range, place_graphic},
    template_library::instance_values,
};

/// What the library use cases work with.
#[derive(Clone)]
pub struct TemplateLibraryDeps {
    /// The library on disk.
    pub library: Arc<dyn TemplateLibrary>,
    /// Open projects.
    pub projects: Arc<ProjectService>,
    /// Checks of imported modules by the renderer.
    pub checks: Arc<TemplateCheckService>,
    /// `libraryChanged` events.
    pub events: Arc<dyn EventSink>,
    /// Dates of entries.
    pub clock: Arc<dyn Clock>,
    /// Ids of new entries.
    pub ids: Arc<dyn IdGenerator>,
}

/// Template library use cases. Cheap to clone.
#[derive(Clone)]
pub struct TemplateLibraryService {
    deps: TemplateLibraryDeps,
}

impl TemplateLibraryService {
    /// The service over `deps`.
    #[must_use]
    pub const fn new(deps: TemplateLibraryDeps) -> Self {
        Self { deps }
    }

    /// Every entry, by name (case-insensitive).
    ///
    /// # Errors
    /// The library folder cannot be read.
    pub async fn list(&self) -> Result<Vec<LibraryTemplate>, AppError> {
        let mut entries = self.deps.library.list().await?;
        entries.sort_by_cached_key(|entry| (entry.name.to_lowercase(), entry.id));
        Ok(entries)
    }

    /// The entry `id`.
    ///
    /// # Errors
    /// [`AppError::LibraryTemplateNotFound`], storage errors.
    pub async fn entry(&self, id: LibraryTemplateId) -> Result<LibraryTemplate, AppError> {
        self.deps
            .library
            .get(id)
            .await?
            .ok_or(AppError::LibraryTemplateNotFound { template_id: id })
    }

    /// Saves project template `template_id` in the library: a new version of the entry it was
    /// copied from when it has one (a new version only when its module changed), else a new
    /// entry; the project's copy then links to that version, with the saved name and category
    /// (one undoable step « Save template to library » when that changes it).
    ///
    /// # Errors
    /// Not open, unknown template, unusable information, storage errors.
    pub async fn save_template(
        &self,
        project_id: ProjectId,
        template_id: TemplateId,
        info: &LibraryTemplateInfo,
    ) -> Result<LibraryTemplate, AppError> {
        let info = checked_info(info)?;
        let project = self.deps.projects.project(project_id).await?;
        let template = find_template(&project.templates, template_id)?;
        let linked = match template.library {
            Some(link) => self.deps.library.get(link.template_id).await?,
            None => None,
        };
        let content = LibraryContent::MotionTemplate {
            category: info.category.unwrap_or(template.category),
            source: template.source.clone(),
            params: template.params.clone(),
            default_duration: template.default_duration,
        };
        let entry = match linked {
            Some(previous) => {
                let changed = !same_module(&previous.content, &content);
                LibraryTemplate {
                    version: previous.version + u32::from(changed),
                    ..self.entry_with(previous.id, previous.version, &info, content)
                }
            }
            None => self.entry_with(
                NewIds(self.deps.ids.as_ref()).library_template()?,
                1,
                &info,
                content,
            ),
        };
        self.deps.library.save(&entry).await?;
        self.link_copy(project_id, template, &entry).await?;
        self.changed();
        Ok(entry)
    }

    /// Saves caption style `style_id` of a project as a new library entry.
    ///
    /// # Errors
    /// Not open, unknown style, unusable information, storage errors.
    pub async fn save_caption_style(
        &self,
        project_id: ProjectId,
        style_id: CaptionStyleId,
        info: &LibraryTemplateInfo,
    ) -> Result<LibraryTemplate, AppError> {
        let info = checked_info(info)?;
        let project = self.deps.projects.project(project_id).await?;
        let style = project
            .caption_styles
            .iter()
            .find(|style| style.id == style_id)
            .ok_or(se_domain::DomainError::CaptionStyleNotFound { style: style_id })?
            .clone();
        let id = NewIds(self.deps.ids.as_ref()).library_template()?;
        let entry = self.entry_with(id, 1, &info, LibraryContent::CaptionStyle { style });
        self.deps.library.save(&entry).await?;
        self.changed();
        Ok(entry)
    }

    /// Renames, describes, tags or refiles entry `id` (content and version unchanged).
    ///
    /// # Errors
    /// Unknown entry, unusable information, storage errors.
    pub async fn update_info(
        &self,
        id: LibraryTemplateId,
        info: &LibraryTemplateInfo,
    ) -> Result<LibraryTemplate, AppError> {
        let info = checked_info(info)?;
        let previous = self.entry(id).await?;
        let content = match previous.content {
            LibraryContent::MotionTemplate {
                category,
                source,
                params,
                default_duration,
            } => LibraryContent::MotionTemplate {
                category: info.category.unwrap_or(category),
                source,
                params,
                default_duration,
            },
            style @ LibraryContent::CaptionStyle { .. } => style,
        };
        let entry = self.entry_with(id, previous.version, &info, content);
        self.deps.library.save(&entry).await?;
        self.changed();
        Ok(entry)
    }

    /// Moves entry `id` to the system trash; projects keep their copies.
    ///
    /// # Errors
    /// Unknown entry, storage errors.
    pub async fn remove(&self, id: LibraryTemplateId) -> Result<(), AppError> {
        self.entry(id).await?;
        self.deps.library.remove(id).await?;
        self.changed();
        Ok(())
    }

    /// Copies a library motion template into a project (reusing a copy of the same version) and
    /// places an instance with the brand kit's values (one undoable step « Insert motion
    /// template »).
    ///
    /// # Errors
    /// Not open, unknown entry, sequence or track, a caption style entry, storage errors.
    pub async fn insert(
        &self,
        request: &InsertLibraryTemplateRequest,
    ) -> Result<InsertedLibraryTemplate, AppError> {
        let entry = self.entry(request.template_id).await?;
        self.deps
            .projects
            .plan_user_edit(request.project_id, labels::INSERT_TEMPLATE, |plan| {
                let template = plan_copy(plan, &entry, Origin::User)?;
                let range = graphic_range(request.start, template.default_duration)?;
                let params = instance_values(plan.project()?, &template, BTreeMap::default());
                let kind = se_domain::ItemKind::MotionTemplate(se_domain::TemplateItem {
                    template_id: template.id,
                    params,
                    transform: se_domain::Transform2D::default(),
                });
                let placed =
                    place_graphic(plan, request.sequence_id, range, request.track_id, kind)?;
                Ok(InsertedLibraryTemplate {
                    template_id: template.id,
                    item_id: placed.item_id,
                    track_id: placed.track_id,
                })
            })
            .await
    }

    /// Replaces project template `template_id` by the newer version of its library entry,
    /// migrating its instances' values (one undoable step « Update motion template »).
    ///
    /// # Errors
    /// Not open, unknown template, a template without library entry (or whose entry is gone),
    /// a copy already up to date, storage errors.
    pub async fn update_project_template(
        &self,
        project_id: ProjectId,
        template_id: TemplateId,
    ) -> Result<TemplateUpdateReport, AppError> {
        let project = self.deps.projects.project(project_id).await?;
        let template = find_template(&project.templates, template_id)?;
        let link = template.library.ok_or_else(|| AppError::InvalidEdit {
            reason: format!(
                "template « {} » does not come from the library: nothing to update",
                template.name
            ),
        })?;
        let entry = self.entry(link.template_id).await?;
        self.deps
            .projects
            .plan_user_edit(project_id, labels::UPDATE_TEMPLATE, |plan| {
                plan_update(plan, template_id, &entry)
            })
            .await
    }

    /// Writes entry `id` as a `.setemplate` file at `path`.
    ///
    /// # Errors
    /// Unknown entry, storage errors.
    pub async fn export(&self, id: LibraryTemplateId, path: &Path) -> Result<(), AppError> {
        let entry = self.entry(id).await?;
        self.deps.library.write_archive(&entry, path).await?;
        Ok(())
    }

    /// Imports `.setemplate` files: each module is checked by the window of `project_id` like
    /// those agents write; a newer version of an entry replaces it, an older or equal one
    /// changes nothing.
    ///
    /// # Errors
    /// Not open (each file's own failure is in its result).
    pub async fn import(
        &self,
        project_id: ProjectId,
        paths: &[String],
    ) -> Result<Vec<se_protocol::LibraryImportResult>, AppError> {
        let project = self.deps.projects.project(project_id).await?;
        let format = project.settings.default_format;
        let mut results = Vec::with_capacity(paths.len());
        let mut changed = false;
        for path in paths {
            let outcome = import_one(
                self.deps.library.as_ref(),
                &self.deps.checks,
                project_id,
                &format,
                Path::new(path),
            )
            .await;
            changed |= matches!(
                outcome,
                se_protocol::LibraryImportOutcome::Imported { .. }
                    | se_protocol::LibraryImportOutcome::Updated { .. }
            );
            results.push(se_protocol::LibraryImportResult {
                path: path.clone(),
                outcome,
            });
        }
        if changed {
            self.changed();
        }
        Ok(results)
    }

    /// Links the project's copy to `entry` with its name, description and category (no change,
    /// no transaction).
    async fn link_copy(
        &self,
        project_id: ProjectId,
        template: &MotionTemplate,
        entry: &LibraryTemplate,
    ) -> Result<(), AppError> {
        let LibraryContent::MotionTemplate { category, .. } = entry.content else {
            return Ok(());
        };
        let linked = MotionTemplate {
            name: entry.name.clone(),
            description: entry.description.clone(),
            category,
            library: Some(LibraryLink {
                template_id: entry.id,
                version: entry.version,
            }),
            ..template.clone()
        };
        if linked == *template {
            return Ok(());
        }
        self.deps
            .projects
            .plan_user_edit(project_id, labels::SAVE_TO_LIBRARY, |plan| {
                plan.push(ReplaceTemplate { template: linked });
                Ok(())
            })
            .await
    }

    fn entry_with(
        &self,
        id: LibraryTemplateId,
        version: u32,
        info: &LibraryTemplateInfo,
        content: LibraryContent,
    ) -> LibraryTemplate {
        LibraryTemplate {
            id,
            version,
            name: info.name.clone(),
            description: info.description.clone(),
            tags: info.tags.clone(),
            updated_at: u64::try_from(self.deps.clock.now_unix_millis()).unwrap_or(0),
            content,
        }
    }

    fn changed(&self) {
        self.deps.events.publish(CoreEvent::LibraryChanged {
            templates: true,
            brand_kits: false,
        });
    }
}

/// Whether two contents hold the same module (source, parameters, length); the category does
/// not count.
fn same_module(left: &LibraryContent, right: &LibraryContent) -> bool {
    match (left, right) {
        (
            LibraryContent::MotionTemplate {
                source,
                params,
                default_duration,
                ..
            },
            LibraryContent::MotionTemplate {
                source: other_source,
                params: other_params,
                default_duration: other_duration,
                ..
            },
        ) => source == other_source && params == other_params && default_duration == other_duration,
        _ => false,
    }
}

fn find_template(
    templates: &[MotionTemplate],
    id: TemplateId,
) -> Result<&MotionTemplate, AppError> {
    templates
        .iter()
        .find(|template| template.id == id)
        .ok_or(AppError::Domain(se_domain::DomainError::TemplateNotFound {
            template: id,
        }))
}
