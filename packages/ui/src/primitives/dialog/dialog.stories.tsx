import type { Meta, StoryObj } from '@storybook/react-vite';

import { Button } from '../button';
import { Field } from '../field';
import { Input } from '../input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../select';
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from './dialog';

// Annotated (not `satisfies`): the inferred type would name Radix internals in declarations.
const meta: Meta<typeof Dialog> = {
  title: 'Primitives/Dialog',
  component: Dialog,
};

export default meta;
type Story = StoryObj<typeof Dialog>;

/** A small creation form: Tab stays inside, Escape or « Annuler » closes. */
export const Default: Story = {
  render: (args) => (
    <Dialog {...args}>
      <DialogTrigger asChild>
        <Button variant="primary">Nouvelle séquence</Button>
      </DialogTrigger>
      <DialogContent closeLabel="Fermer">
        <DialogHeader>
          <DialogTitle>Nouvelle séquence</DialogTitle>
          <DialogDescription>Choisis un nom et un format d’image.</DialogDescription>
        </DialogHeader>
        <div className="flex flex-col gap-3">
          <Field label="Nom">
            <Input defaultValue="Séquence 1" />
          </Field>
          <Select defaultValue="16:9">
            <Field label="Format">
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
            </Field>
            <SelectContent>
              <SelectItem value="16:9">16:9 — 1920 × 1080</SelectItem>
              <SelectItem value="9:16">9:16 — 1080 × 1920</SelectItem>
              <SelectItem value="1:1">1:1 — 1080 × 1080</SelectItem>
            </SelectContent>
          </Select>
        </div>
        <DialogFooter>
          <DialogClose asChild>
            <Button variant="ghost">Annuler</Button>
          </DialogClose>
          <Button variant="primary">Créer</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  ),
};

/** Shown open, to review the surface and the backdrop. */
export const Open: Story = { ...Default, args: { defaultOpen: true } };

/** The three widths: `sm`, `md` (default) and `lg`. */
export const Sizes: Story = {
  render: () => (
    <div className="flex gap-2">
      {(['sm', 'md', 'lg', 'xl'] as const).map((size) => (
        <Dialog key={size}>
          <DialogTrigger asChild>
            <Button>{size}</Button>
          </DialogTrigger>
          <DialogContent size={size} closeLabel="Fermer">
            <DialogHeader>
              <DialogTitle>Taille {size}</DialogTitle>
              <DialogDescription>Largeur maximale de la fenêtre.</DialogDescription>
            </DialogHeader>
          </DialogContent>
        </Dialog>
      ))}
    </div>
  ),
};
