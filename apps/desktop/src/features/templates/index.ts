/**
 * Motion templates (prompts 24–25, ADR-0035, ADR-0036): the checks of modules AI agents write
 * (`useTemplateCheckRequests`, mounted by the app layout), « Voir le code »
 * (`TemplateCodeDialog`, mounted by the editor, opened with `openTemplateCode`), the template
 * browser of the Animations tab (`TemplateBrowser`: project templates and the library, drags to
 * the timeline) and the library information dialog (`LibraryInfoDialog`, mounted by the editor;
 * `openSaveCaptionStyle` for the captions feature). The generated inspector section lives in the
 * inspector feature.
 */
export { LibraryInfoDialog } from './components/LibraryInfoDialog';
export { TemplateBrowser } from './components/TemplateBrowser';
export { TemplateCodeDialog } from './components/TemplateCodeDialog';
export { useTemplateCheckRequests } from './hooks/use-template-check-requests';
export { useLibraryTemplates } from './store/library-store';
export { closeTemplateCode, openTemplateCode } from './store/code-dialog-store';
export { openSaveCaptionStyle } from './store/library-dialog-store';
