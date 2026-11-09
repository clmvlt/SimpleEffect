/**
 * Motion templates (prompt 24, ADR-0035): the checks of modules AI agents write
 * (`useTemplateCheckRequests`, mounted by the app layout), « Voir le code »
 * (`TemplateCodeDialog`, mounted by the editor, opened with `openTemplateCode`) and the
 * « Modèles du projet » list of the Animations tab (`ProjectTemplates`). The generated
 * inspector section lives in the inspector feature.
 */
export { ProjectTemplates } from './components/ProjectTemplates';
export { TemplateCodeDialog } from './components/TemplateCodeDialog';
export { useTemplateCheckRequests } from './hooks/use-template-check-requests';
export { closeTemplateCode, openTemplateCode } from './store/code-dialog-store';
