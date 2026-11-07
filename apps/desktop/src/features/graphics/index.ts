/**
 * Graphics feature (prompt 15): creation tools (text, shapes, images at the playhead on the
 * highest free video track), the « Animations » presets panel, and the graphics preferences.
 */
export { AnimationsPanel } from './components/AnimationsPanel';
export { GraphicsTools, SHAPE_KINDS } from './components/GraphicsTools';
export { graphicsApi, type GraphicsApi } from './api';
export { useCreateGraphic, type GraphicCreator } from './hooks/use-create-graphic';
