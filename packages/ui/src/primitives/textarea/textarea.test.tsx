import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { Textarea } from './textarea';

describe('Textarea', () => {
  it('is a multi-line text field named by its label', async () => {
    const user = userEvent.setup();
    render(
      <>
        <label htmlFor="prompt">Consigne</label>
        <Textarea id="prompt" />
      </>,
    );
    const textarea = screen.getByRole('textbox', { name: 'Consigne' });
    await user.type(textarea, 'Coupe les silences{Enter}puis ajoute des sous-titres');
    expect(textarea).toHaveValue('Coupe les silences\npuis ajoute des sous-titres');
  });

  it('exposes the invalid state', () => {
    render(<Textarea aria-label="Consigne" aria-invalid />);
    expect(screen.getByRole('textbox', { name: 'Consigne' })).toBeInvalid();
  });
});
