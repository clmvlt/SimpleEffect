import type { Meta, StoryObj } from '@storybook/react-vite';

import { Field } from '../field';
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectSeparator,
  SelectTrigger,
  SelectValue,
} from './index';

// Annotated rather than `satisfies`: the inferred type would name Radix-internal prop types,
// which declaration emit cannot reference.
const meta: Meta<typeof Select> = {
  title: 'Primitives/Select',
  component: Select,
  args: { defaultValue: '25' },
  render: (args) => (
    <Select {...args}>
      <SelectTrigger aria-label="Cadence" className="w-40">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="23.976">23,976 i/s</SelectItem>
        <SelectItem value="24">24 i/s</SelectItem>
        <SelectItem value="25">25 i/s</SelectItem>
        <SelectItem value="29.97">29,97 i/s</SelectItem>
        <SelectItem value="30">30 i/s</SelectItem>
        <SelectItem value="50">50 i/s</SelectItem>
        <SelectItem value="60">60 i/s</SelectItem>
      </SelectContent>
    </Select>
  ),
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** Open by default, to review the list styles. */
export const Open: Story = { args: { defaultOpen: true } };

export const Placeholder: Story = {
  args: { defaultValue: undefined },
  render: (args) => (
    <Select {...args}>
      <SelectTrigger aria-label="Format" className="w-48">
        <SelectValue placeholder="Choisir un format" />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="16:9">16:9 — Paysage</SelectItem>
        <SelectItem value="9:16">9:16 — Vertical</SelectItem>
      </SelectContent>
    </Select>
  ),
};

/** Groups with labels and separators, and a disabled option. */
export const Groups: Story = {
  args: { defaultValue: 'h264' },
  render: (args) => (
    <Select {...args}>
      <SelectTrigger aria-label="Codec" className="w-48">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectGroup>
          <SelectLabel>Diffusion</SelectLabel>
          <SelectItem value="h264">H.264</SelectItem>
          <SelectItem value="h265">H.265 (HEVC)</SelectItem>
          <SelectItem value="av1" disabled>
            AV1
          </SelectItem>
        </SelectGroup>
        <SelectSeparator />
        <SelectGroup>
          <SelectLabel>Montage</SelectLabel>
          <SelectItem value="prores">ProRes 422</SelectItem>
          <SelectItem value="dnxhr">DNxHR HQ</SelectItem>
        </SelectGroup>
      </SelectContent>
    </Select>
  ),
};

/** The three trigger heights: 24, 28 and 32 px. */
export const Sizes: Story = {
  render: (args) => (
    <div className="flex flex-col gap-2">
      {(['xs', 'sm', 'md'] as const).map((size) => (
        <Select key={size} {...args}>
          <SelectTrigger aria-label={`Cadence (${size})`} size={size} className="w-40">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="24">24 i/s</SelectItem>
            <SelectItem value="25">25 i/s</SelectItem>
          </SelectContent>
        </Select>
      ))}
    </div>
  ),
};

/** Inside a `Field`: wrap the trigger, which is the focusable control. */
export const WithField: Story = {
  render: (args) => (
    <Select {...args}>
      <Field label="Cadence" description="Images par seconde de la séquence." className="w-56">
        <SelectTrigger>
          <SelectValue />
        </SelectTrigger>
      </Field>
      <SelectContent>
        <SelectItem value="24">24 i/s</SelectItem>
        <SelectItem value="25">25 i/s</SelectItem>
        <SelectItem value="30">30 i/s</SelectItem>
      </SelectContent>
    </Select>
  ),
};

export const Invalid: Story = {
  render: (args) => (
    <Select {...args}>
      <Field label="Cadence" error="Cadence incompatible avec le média source." className="w-56">
        <SelectTrigger>
          <SelectValue />
        </SelectTrigger>
      </Field>
      <SelectContent>
        <SelectItem value="25">25 i/s</SelectItem>
      </SelectContent>
    </Select>
  ),
};

export const Disabled: Story = { args: { disabled: true } };
