import type { Meta, StoryObj } from '@storybook/react-vite';
import { Trash2 } from 'lucide-react';

import { Button } from '../button';
import { Icon } from '../icon';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from './alert-dialog';

// Annotated (not `satisfies`): the inferred type would name Radix internals in declarations.
const meta: Meta<typeof AlertDialog> = {
  title: 'Primitives/AlertDialog',
  component: AlertDialog,
};

export default meta;
type Story = StoryObj<typeof AlertDialog>;

/** A destructive confirmation: focus starts on « Annuler », a click outside does nothing. */
export const Destructive: Story = {
  render: (args) => (
    <AlertDialog {...args}>
      <AlertDialogTrigger asChild>
        <Button variant="danger">
          <Icon icon={Trash2} size="sm" />
          Supprimer la séquence
        </Button>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Supprimer « Séquence 1 » ?</AlertDialogTitle>
          <AlertDialogDescription>
            Les clips de la séquence seront retirés. Les médias du projet sont conservés.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Annuler</AlertDialogCancel>
          <AlertDialogAction variant="danger">Supprimer</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  ),
};

/** Shown open, to review the surface. */
export const Open: Story = { ...Destructive, args: { defaultOpen: true } };

/** A non-destructive confirmation uses the primary action. */
export const Confirmation: Story = {
  render: (args) => (
    <AlertDialog {...args}>
      <AlertDialogTrigger asChild>
        <Button>Remplacer le média</Button>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Remplacer le média ?</AlertDialogTitle>
          <AlertDialogDescription>
            Les 4 clips qui l’utilisent pointeront vers le nouveau fichier.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Annuler</AlertDialogCancel>
          <AlertDialogAction>Remplacer</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  ),
};
