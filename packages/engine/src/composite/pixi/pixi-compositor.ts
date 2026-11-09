// Pixi generates shader and uniform code with `new Function` unless this polyfill is loaded first;
// the renderer's Content-Security-Policy forbids `unsafe-eval`.
import 'pixi.js/unsafe-eval';

import { AlphaFilter, autoDetectRenderer, Container, type Renderer } from 'pixi.js';

import type { FrameGraph, Layer, Size } from '../../evaluate/frame-graph';
import type {
  Compositor,
  CompositorBackground,
  LayerBoxes,
  PictureSet,
  RenderCanvas,
} from '../compositor';
import { outputScale } from '../layout';
import { CaptionNode } from './caption-node';
import type { LayerNode } from './layer-node';
import { PictureNode } from './picture-node';
import { ShapeNode } from './shape-node';
import { TextNode } from './text-node';

/** Options of {@link createPixiCompositor}. */
export interface PixiCompositorOptions {
  readonly canvas: RenderCanvas;
  /** Initial output size in pixels. */
  readonly width: number;
  readonly height: number;
  /** WebGL 2 by default; `webgpu` tries WebGPU first and falls back to WebGL 2. */
  readonly renderer?: 'webgl' | 'webgpu';
  readonly background?: CompositorBackground;
  /**
   * Keep the drawing after compositing so it can be read back later (tests, still exports);
   * slower, off by default.
   */
  readonly preserveDrawingBuffer?: boolean;
}

/** The PixiJS v8 implementation of {@link Compositor}. */
class PixiCompositor implements Compositor {
  private readonly stage = new Container();
  private readonly frameRoot = new Container();
  private nodes = new Map<string, LayerNode>();
  /** One isolated group per cross dissolve on screen, by transition id. */
  private groups = new Map<string, Container>();
  private destroyed = false;

  constructor(private readonly renderer: Renderer) {
    this.stage.addChild(this.frameRoot);
  }

  resize(width: number, height: number): void {
    this.renderer.resize(Math.max(1, Math.round(width)), Math.max(1, Math.round(height)));
  }

  setBackground(background: CompositorBackground): void {
    this.renderer.background.color = 0x000000;
    this.renderer.background.alpha = background === 'black' ? 1 : 0;
  }

  render(graph: FrameGraph, pictures: PictureSet): LayerBoxes {
    const boxes = new Map<string, Size>();
    if (this.destroyed) return boxes;
    const scale = outputScale(graph.frame, this.renderer.screen);
    this.frameRoot.scale.set(scale.x, scale.y);
    const used = new Map<string, LayerNode>();
    const usedGroups = new Map<string, Container>();
    this.frameRoot.removeChildren();
    for (const group of this.groups.values()) group.removeChildren();
    for (const layer of graph.layers) {
      const node = this.nodeFor(layer);
      used.set(layer.itemId, node);
      const box = this.updateNode(node, layer, graph, pictures, scale.x);
      if (box !== null) boxes.set(layer.itemId, box);
      node.root.blendMode = layer.dissolve?.side === 'incoming' ? 'add' : 'inherit';
      this.parentOf(layer, usedGroups).addChild(node.root);
    }
    for (const [itemId, node] of this.nodes) {
      if (used.get(itemId) !== node) node.destroy();
    }
    for (const [transitionId, group] of this.groups) {
      if (!usedGroups.has(transitionId)) destroyGroup(group);
    }
    this.nodes = used;
    this.groups = usedGroups;
    this.renderer.render({ container: this.stage });
    return boxes;
  }

  /**
   * Where a layer's node goes: the frame, or the isolated group of its dissolve (added to the
   * frame where its first layer comes). The group's alpha filter only isolates (alpha 1): the
   * sides are drawn into an empty layer, the incoming one added (`DISSOLVE_COMPOSITING`).
   */
  private parentOf(layer: Layer, usedGroups: Map<string, Container>): Container {
    if (layer.dissolve === undefined) return this.frameRoot;
    const transitionId = layer.dissolve.transitionId;
    let group = usedGroups.get(transitionId);
    if (group === undefined) {
      group =
        this.groups.get(transitionId) ??
        new Container({ filters: [new AlphaFilter({ alpha: 1 })] });
      usedGroups.set(transitionId, group);
      this.frameRoot.addChild(group);
    }
    return group;
  }

  destroy(): void {
    if (this.destroyed) return;
    this.destroyed = true;
    for (const node of this.nodes.values()) node.destroy();
    this.nodes.clear();
    for (const group of this.groups.values()) destroyGroup(group);
    this.groups.clear();
    this.stage.destroy({ children: true });
    this.renderer.destroy();
  }

  /** The node kept for the layer's item, or a new one when its kind changed. */
  private nodeFor(layer: Layer): LayerNode {
    const existing = this.nodes.get(layer.itemId);
    if (existing?.kind === layer.kind) return existing;
    switch (layer.kind) {
      case 'video':
      case 'image':
      case 'template':
        return new PictureNode(layer.kind);
      case 'shape':
        return new ShapeNode();
      case 'text':
        return new TextNode();
      case 'caption':
        return new CaptionNode();
    }
  }

  /** Updates `node` for `layer` (`scale`: output pixels per sequence pixel); returns its box. */
  private updateNode(
    node: LayerNode,
    layer: Layer,
    graph: FrameGraph,
    pictures: PictureSet,
    scale: number,
  ): Size | null {
    if (
      node instanceof PictureNode &&
      (layer.kind === 'video' || layer.kind === 'image' || layer.kind === 'template')
    ) {
      return node.update(layer, pictures.get(layer.itemId), graph.frame, scale);
    }
    if (node instanceof ShapeNode && layer.kind === 'shape') {
      return node.update(layer, graph.frame, scale);
    }
    if (node instanceof TextNode && layer.kind === 'text') {
      return node.update(layer, graph.frame, scale);
    }
    if (node instanceof CaptionNode && layer.kind === 'caption') {
      return node.update(layer, graph.frame, scale);
    }
    return null;
  }
}

/** Frees a dissolve group and its filter (its layer nodes are owned and freed elsewhere). */
function destroyGroup(group: Container): void {
  const filters = [...group.filters];
  group.destroy();
  for (const filter of filters) filter.destroy();
}

/**
 * Creates a PixiJS compositor drawing on `canvas` (an `HTMLCanvasElement`, or an `OffscreenCanvas`
 * in the page or in a Worker). Rejects when no WebGL 2 (or WebGPU) context can be created.
 */
export async function createPixiCompositor(options: PixiCompositorOptions): Promise<Compositor> {
  const renderer = await autoDetectRenderer({
    canvas: options.canvas,
    width: Math.max(1, Math.round(options.width)),
    height: Math.max(1, Math.round(options.height)),
    preference: options.renderer === 'webgpu' ? ['webgpu', 'webgl'] : ['webgl'],
    preferWebGLVersion: 2,
    resolution: 1,
    autoDensity: false,
    antialias: true,
    background: 0x000000,
    backgroundAlpha: options.background === 'transparent' ? 0 : 1,
    preserveDrawingBuffer: options.preserveDrawingBuffer ?? false,
  });
  return new PixiCompositor(renderer);
}
