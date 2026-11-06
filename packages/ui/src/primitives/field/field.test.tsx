import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { Input } from '../input';
import { Field } from './field';

describe('Field', () => {
  it('names the control with its label', () => {
    render(
      <Field label="Nom de la séquence">
        <Input />
      </Field>,
    );
    const input = screen.getByRole('textbox', { name: 'Nom de la séquence' });
    expect(input).toBeValid();
    expect(input).not.toHaveAttribute('aria-describedby');
  });

  it('describes the control with the description and the error, and marks it invalid', () => {
    render(
      <Field label="Durée" description="Format hh:mm:ss:ff" error="Timecode invalide">
        <Input />
      </Field>,
    );
    const input = screen.getByRole('textbox', { name: 'Durée' });
    expect(input).toHaveAccessibleDescription('Format hh:mm:ss:ff Timecode invalide');
    expect(input).toBeInvalid();
  });

  it('uses the given id for the control', () => {
    render(
      <Field label="Nom" id="sequence-name">
        <Input />
      </Field>,
    );
    expect(screen.getByRole('textbox', { name: 'Nom' })).toHaveAttribute('id', 'sequence-name');
  });

  it('puts the control before the label in horizontal orientation', () => {
    render(
      <Field label="Verrouiller" orientation="horizontal">
        <input type="checkbox" />
      </Field>,
    );
    const checkbox = screen.getByRole('checkbox', { name: 'Verrouiller' });
    const label = screen.getByText('Verrouiller');
    expect(checkbox.compareDocumentPosition(label) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  });
});
