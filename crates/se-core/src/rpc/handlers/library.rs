//! The app's library (prompt 25, ADR-0036): template library entries and brand kits. Paths must
//! be absolute.

use se_domain::{BrandKitId, CaptionStyleId, LibraryTemplateId, ProjectId, TemplateId};
use se_protocol::{
    CommandResult, InsertLibraryTemplateRequest, LibraryBrandKit, LibraryTemplateInfo,
    ProtocolError,
};

use super::{HandlerContext, absolute_path};

/// `listLibraryTemplates`.
///
/// # Errors
/// The library folder cannot be read.
pub async fn list_templates(context: &HandlerContext) -> Result<CommandResult, ProtocolError> {
    let templates = context.library_templates.list().await?;
    Ok(CommandResult::ListLibraryTemplates { templates })
}

/// `saveTemplateToLibrary`.
///
/// # Errors
/// Not open, unknown template, unusable information, storage errors.
pub async fn save_template(
    context: &HandlerContext,
    project_id: ProjectId,
    template_id: TemplateId,
    info: &LibraryTemplateInfo,
) -> Result<CommandResult, ProtocolError> {
    let template = context
        .library_templates
        .save_template(project_id, template_id, info)
        .await?;
    Ok(CommandResult::SaveTemplateToLibrary { template })
}

/// `saveCaptionStyleToLibrary`.
///
/// # Errors
/// Not open, unknown style, unusable information, storage errors.
pub async fn save_caption_style(
    context: &HandlerContext,
    project_id: ProjectId,
    style_id: CaptionStyleId,
    info: &LibraryTemplateInfo,
) -> Result<CommandResult, ProtocolError> {
    let template = context
        .library_templates
        .save_caption_style(project_id, style_id, info)
        .await?;
    Ok(CommandResult::SaveCaptionStyleToLibrary { template })
}

/// `updateLibraryTemplateInfo`.
///
/// # Errors
/// Unknown entry, unusable information, storage errors.
pub async fn update_info(
    context: &HandlerContext,
    template_id: LibraryTemplateId,
    info: &LibraryTemplateInfo,
) -> Result<CommandResult, ProtocolError> {
    let template = context
        .library_templates
        .update_info(template_id, info)
        .await?;
    Ok(CommandResult::UpdateLibraryTemplateInfo { template })
}

/// `removeLibraryTemplate`.
///
/// # Errors
/// Unknown entry, storage errors.
pub async fn remove_template(
    context: &HandlerContext,
    template_id: LibraryTemplateId,
) -> Result<CommandResult, ProtocolError> {
    context.library_templates.remove(template_id).await?;
    Ok(CommandResult::RemoveLibraryTemplate)
}

/// `insertLibraryTemplate`: one undoable step.
///
/// # Errors
/// Not open, unknown entry, sequence or track, a caption style entry, storage errors.
pub async fn insert_template(
    context: &HandlerContext,
    request: &InsertLibraryTemplateRequest,
) -> Result<CommandResult, ProtocolError> {
    let inserted = context.library_templates.insert(request).await?;
    Ok(CommandResult::InsertLibraryTemplate(inserted))
}

/// `updateTemplateFromLibrary`: one undoable step.
///
/// # Errors
/// Not open, unknown template or entry, a copy already up to date, storage errors.
pub async fn update_project_template(
    context: &HandlerContext,
    project_id: ProjectId,
    template_id: TemplateId,
) -> Result<CommandResult, ProtocolError> {
    let report = context
        .library_templates
        .update_project_template(project_id, template_id)
        .await?;
    Ok(CommandResult::UpdateTemplateFromLibrary(report))
}

/// `exportLibraryTemplate`.
///
/// # Errors
/// A relative path, unknown entry, storage errors.
pub async fn export_template(
    context: &HandlerContext,
    template_id: LibraryTemplateId,
    path: &str,
) -> Result<CommandResult, ProtocolError> {
    let path = absolute_path(path)?;
    context.library_templates.export(template_id, path).await?;
    Ok(CommandResult::ExportLibraryTemplate)
}

/// `importLibraryTemplates`: each file's outcome, checked by the project's window.
///
/// # Errors
/// A relative path, not open.
pub async fn import_templates(
    context: &HandlerContext,
    project_id: ProjectId,
    paths: &[String],
) -> Result<CommandResult, ProtocolError> {
    for path in paths {
        absolute_path(path)?;
    }
    let results = context.library_templates.import(project_id, paths).await?;
    Ok(CommandResult::ImportLibraryTemplates { results })
}

/// `listBrandKits`.
///
/// # Errors
/// The library folder cannot be read.
pub async fn list_brand_kits(context: &HandlerContext) -> Result<CommandResult, ProtocolError> {
    let kits = context.brand_kits.list().await?;
    Ok(CommandResult::ListBrandKits { kits })
}

/// `createBrandKit`.
///
/// # Errors
/// Unusable name, unknown kit to copy, storage errors.
pub async fn create_brand_kit(
    context: &HandlerContext,
    name: &str,
    copy_of: Option<BrandKitId>,
) -> Result<CommandResult, ProtocolError> {
    let kit = context.brand_kits.create(name, copy_of).await?;
    Ok(CommandResult::CreateBrandKit { kit })
}

/// `saveBrandKit`: open projects using it follow.
///
/// # Errors
/// Unknown kit, unusable values, storage errors.
pub async fn save_brand_kit(
    context: &HandlerContext,
    kit: &LibraryBrandKit,
) -> Result<CommandResult, ProtocolError> {
    let kit = context.brand_kits.save(kit).await?;
    Ok(CommandResult::SaveBrandKit { kit })
}

/// `removeBrandKit`.
///
/// # Errors
/// Unknown kit, storage errors.
pub async fn remove_brand_kit(
    context: &HandlerContext,
    kit_id: BrandKitId,
) -> Result<CommandResult, ProtocolError> {
    context.brand_kits.remove(kit_id).await?;
    Ok(CommandResult::RemoveBrandKit)
}

/// `addBrandKitLogo`.
///
/// # Errors
/// A relative path, unknown kit, a file that is not an image, storage errors.
pub async fn add_brand_kit_logo(
    context: &HandlerContext,
    kit_id: BrandKitId,
    path: &str,
) -> Result<CommandResult, ProtocolError> {
    let path = absolute_path(path)?;
    let kit = context.brand_kits.add_logo(kit_id, path).await?;
    Ok(CommandResult::AddBrandKitLogo { kit })
}

/// `setProjectBrandKit`: one undoable step (plus the import of new logos).
///
/// # Errors
/// Not open, unknown kit, a logo that cannot be imported, storage errors.
pub async fn set_project_brand_kit(
    context: &HandlerContext,
    project_id: ProjectId,
    kit_id: Option<BrandKitId>,
) -> Result<CommandResult, ProtocolError> {
    context
        .brand_kits
        .set_project_kit(project_id, kit_id)
        .await?;
    Ok(CommandResult::SetProjectBrandKit)
}

/// `addProjectBrandKitToLibrary`.
///
/// # Errors
/// Not open, no kit, unreadable logo files, storage errors.
pub async fn add_project_brand_kit_to_library(
    context: &HandlerContext,
    project_id: ProjectId,
) -> Result<CommandResult, ProtocolError> {
    let kit = context
        .brand_kits
        .add_project_kit_to_library(project_id)
        .await?;
    Ok(CommandResult::AddProjectBrandKitToLibrary { kit })
}
