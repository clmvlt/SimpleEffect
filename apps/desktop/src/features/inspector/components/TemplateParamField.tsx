import type { TemplateParam } from '@se/protocol';
import {
  Input,
  KeyframeButton,
  PropertyRow,
  ScrubInput,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
  Textarea,
} from '@se/ui';
import { useState, type ReactNode } from 'react';
import { useTranslation } from 'react-i18next';

import { BrandColorField } from '../../../shared/brand-kit';
import { useProjectSelector } from '../../../shared/project-store';
import type { ParamChange, TemplateParamsEditor } from '../hooks/use-template-params';
import { keepAlpha, opaquePart } from '../services/inspector-values';
import { colorAt, numberAt, withColorAt, withNumberAt } from '../services/template-params';
import { FontFamilySelect } from '../../../shared/fonts';

/** Value of the asset menu meaning « no image ». */
const NO_ASSET = '__none__';

/**
 * The control a template parameter's kind gets (prompt 24): text → input or text area, number →
 * scrub field with keyframes, colour → colour field with keyframes, flag → switch, option →
 * select, font → font menu, image → menu of the project's images.
 */
export function TemplateParamField({
  editor,
  param,
}: {
  readonly editor: TemplateParamsEditor;
  readonly param: TemplateParam;
}) {
  const id = `inspector-param-${param.name}`;
  const value = editor.value(param);
  const { spec } = param;
  if (spec.kind === 'number' || spec.kind === 'color') {
    return (
      <ParamRow editor={editor} param={param} id={id} animated>
        {spec.kind === 'number' ? (
          <NumberField editor={editor} param={param} id={id} />
        ) : (
          <ColorParamField editor={editor} param={param} id={id} />
        )}
      </ParamRow>
    );
  }
  return (
    <ParamRow editor={editor} param={param} id={id}>
      {spec.kind === 'string' && <TextField editor={editor} param={param} id={id} />}
      {spec.kind === 'boolean' && value.kind === 'boolean' && (
        <Switch
          id={id}
          checked={value.value}
          onCheckedChange={(checked) => {
            editor.commit(param, () => ({ kind: 'boolean', value: checked }));
          }}
        />
      )}
      {spec.kind === 'enum' && value.kind === 'enum' && (
        <Select
          value={value.value}
          onValueChange={(option) => {
            editor.commit(param, () => ({ kind: 'enum', value: option }));
          }}
        >
          <SelectTrigger id={id} size="xs" className="w-full">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {spec.options.map((option) => (
              <SelectItem key={option.value} value={option.value}>
                {option.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      )}
      {spec.kind === 'font' && value.kind === 'font' && (
        <FontFamilySelect
          id={id}
          value={value.value}
          onChange={(family) => {
            editor.commit(param, () => ({ kind: 'font', value: family }));
          }}
        />
      )}
      {spec.kind === 'asset' && value.kind === 'asset' && (
        <AssetField editor={editor} param={param} id={id} selected={value.value} />
      )}
    </ParamRow>
  );
}

function ParamRow({
  editor,
  param,
  id,
  animated = false,
  children,
}: {
  readonly editor: TemplateParamsEditor;
  readonly param: TemplateParam;
  readonly id: string;
  readonly animated?: boolean;
  readonly children: ReactNode;
}) {
  const { t } = useTranslation();
  const keyframe = animated ? (
    <KeyframeButton
      state={editor.keyframe(param)}
      onToggle={() => {
        editor.toggleKeyframe(param);
      }}
      onPrevious={editor.goToKeyframe(param, -1)}
      onNext={editor.goToKeyframe(param, 1)}
      label={t('editor.inspector.keyframes.toggle', { property: param.label })}
      previousLabel={t('editor.inspector.keyframes.previous')}
      nextLabel={t('editor.inspector.keyframes.next')}
    />
  ) : undefined;
  return (
    <div title={param.description ?? undefined}>
      <PropertyRow label={param.label} htmlFor={id} keyframe={keyframe}>
        {children}
      </PropertyRow>
    </div>
  );
}

function TextField({ editor, param, id }: FieldProps) {
  const [draft, setDraft] = useState<string | null>(null);
  const value = editor.value(param);
  if (value.kind !== 'text' || param.spec.kind !== 'string') return null;
  const shown = draft ?? value.value;
  const change = (text: string) => {
    setDraft(text);
    editor.preview(param, () => ({ kind: 'text', value: text }));
  };
  const commit = () => {
    if (draft !== null && draft !== value.value) {
      editor.commit(param, () => ({ kind: 'text', value: draft }));
    }
    setDraft(null);
  };
  const props = {
    id,
    value: shown,
    maxLength: param.spec.maxLength,
    onBlur: commit,
  };
  return param.spec.multiline ? (
    <Textarea
      {...props}
      rows={2}
      onChange={(event) => {
        change(event.target.value);
      }}
    />
  ) : (
    <Input
      {...props}
      size="xs"
      onChange={(event) => {
        change(event.target.value);
      }}
      onKeyDown={(event) => {
        if (event.key === 'Enter') commit();
      }}
    />
  );
}

interface FieldProps {
  readonly editor: TemplateParamsEditor;
  readonly param: TemplateParam;
  readonly id: string;
}

function NumberField({ editor, param, id }: FieldProps) {
  const [draft, setDraft] = useState<number | null>(null);
  const number = numberAt(editor.value(param), editor.local);
  if (number === null || param.spec.kind !== 'number') return null;
  const { minimum, maximum, step } = param.spec;
  const change =
    (next: number): ParamChange =>
    (current, local, tolerance) =>
      withNumberAt(current, local, next, tolerance);
  return (
    <ScrubInput
      id={id}
      size="xs"
      className="w-full"
      value={draft ?? number}
      step={step ?? 1}
      min={minimum ?? undefined}
      max={maximum ?? undefined}
      onChange={(next) => {
        setDraft(next);
        editor.preview(param, change(next));
      }}
      onCommit={(next) => {
        setDraft(null);
        editor.commit(param, change(next));
      }}
    />
  );
}

function ColorParamField({ editor, param, id }: FieldProps) {
  const color = colorAt(editor.value(param), editor.local);
  if (color === null) return null;
  const change =
    (hex: string): ParamChange =>
    (current, local, tolerance) =>
      withColorAt(current, local, keepAlpha(hex, color), tolerance);
  return (
    <BrandColorField
      id={id}
      label={param.label}
      value={opaquePart(color)}
      onChange={(hex) => {
        editor.preview(param, change(hex));
      }}
      onCommit={(hex) => {
        editor.commit(param, change(hex));
      }}
    />
  );
}

function AssetField({
  editor,
  param,
  id,
  selected,
}: FieldProps & { readonly selected: string | null }) {
  const { t } = useTranslation();
  const assets = useProjectSelector((state) => state.mirror?.assets);
  const images = Object.values(assets ?? {}).filter((asset) => asset.kind === 'image');
  return (
    <Select
      value={selected ?? NO_ASSET}
      onValueChange={(choice) => {
        editor.commit(param, () => ({
          kind: 'asset',
          value: choice === NO_ASSET ? null : choice,
        }));
      }}
    >
      <SelectTrigger id={id} size="xs" className="w-full">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value={NO_ASSET}>{t('editor.templates.inspector.noImage')}</SelectItem>
        {images.map((asset) => (
          <SelectItem key={asset.id} value={asset.id}>
            {fileName(asset.originalPath)}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}

/** The last part of a path, whatever its separators. */
function fileName(path: string): string {
  return path.split(/[\\/]/).at(-1) ?? path;
}
