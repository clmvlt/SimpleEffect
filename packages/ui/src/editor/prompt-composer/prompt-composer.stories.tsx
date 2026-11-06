import type { Meta, StoryObj } from '@storybook/react-vite';
import { useState } from 'react';

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../../primitives/select';
import { KeyboardShortcut } from '../keyboard-shortcut';
import { PromptComposer, type PromptComposerProps, type PromptScope } from './prompt-composer';

const SCOPES: PromptScope[] = [
  { id: 'clip-1', label: 'Interview A' },
  { id: 'clip-2', label: 'Plan de coupe 3' },
  { id: 'range', label: '00:12 → 00:45' },
];

function ModelSelect() {
  return (
    <Select defaultValue="default">
      <SelectTrigger size="xs" aria-label="Modèle" className="w-28">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="default">Par défaut</SelectItem>
        <SelectItem value="fast">Rapide</SelectItem>
      </SelectContent>
    </Select>
  );
}

function EffortSelect() {
  return (
    <Select defaultValue="medium">
      <SelectTrigger size="xs" aria-label="Effort" className="w-24">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="low">Faible</SelectItem>
        <SelectItem value="medium">Moyen</SelectItem>
        <SelectItem value="high">Élevé</SelectItem>
      </SelectContent>
    </Select>
  );
}

function ControlledComposer(
  props: Pick<PromptComposerProps, 'busy' | 'disabled'> & { withScopes: boolean },
) {
  const { withScopes, ...rest } = props;
  const [value, setValue] = useState('');
  const [scopes, setScopes] = useState(withScopes ? SCOPES : []);
  const [sent, setSent] = useState<string | null>(null);
  return (
    <div className="flex w-96 flex-col gap-2">
      <PromptComposer
        {...rest}
        label="Demande à l’IA"
        placeholder="Décrivez le montage à faire…"
        submitLabel="Envoyer"
        removeScopeLabel={(scopeLabel) => `Retirer ${scopeLabel}`}
        value={value}
        onValueChange={setValue}
        onSubmit={(text) => {
          setSent(text);
          setValue('');
        }}
        scopes={scopes}
        onRemoveScope={(id) => {
          setScopes(scopes.filter((scope) => scope.id !== id));
        }}
        modelSlot={<ModelSelect />}
        effortSlot={<EffortSelect />}
        submitHint={<KeyboardShortcut keys={['Mod', 'Enter']} keyLabels={{ Enter: 'Entrée' }} />}
      />
      {sent !== null && <p className="text-xs text-text-muted">Envoyé : {sent}</p>}
    </div>
  );
}

const meta = {
  title: 'Editor/PromptComposer',
  component: ControlledComposer,
  args: { withScopes: true, busy: false, disabled: false },
} satisfies Meta<typeof ControlledComposer>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Chips for the scope, model and effort slots; Ctrl/⌘+Enter sends. */
export const Default: Story = {};

export const WithoutScopes: Story = { args: { withScopes: false } };

/** A task is starting: the send button spins and sending is blocked. */
export const Busy: Story = { args: { busy: true } };

export const Disabled: Story = { args: { disabled: true } };
