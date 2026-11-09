export const meta = {
  name: 'Lower third',
  description: 'A name and a role sliding in beside an accent bar',
  defaultDuration: 5,
};

export const params = {
  name: { type: 'string', title: 'Nom', default: 'Jane Doe', maxLength: 60 },
  role: { type: 'string', title: 'Titre', default: 'Réalisatrice', maxLength: 80 },
  accent: { type: 'string', format: 'color', title: 'Accent', default: '#ffcc00' },
  textColor: { type: 'string', format: 'color', title: 'Texte', default: '#ffffff' },
  font: { type: 'string', format: 'font', title: 'Police', default: 'Inter' },
  slide: { type: 'number', title: 'Glissement', default: 80, minimum: 0, maximum: 400 },
};

export function render(ctx, time, p) {
  const enter = ctx.ease.outCubic(ctx.progress(time, 0, 0.6));
  const exit = ctx.ease.inCubic(ctx.progress(time, ctx.duration - 0.5, ctx.duration));
  const shown = enter * (1 - exit);
  if (shown <= 0) return;
  const margin = Math.min(ctx.width, ctx.height) * 0.08;
  const left = margin - p.slide * (1 - shown);
  const top = ctx.height - margin - 190;
  ctx.globalAlpha = shown;
  const bar = ctx.ease.outBack(ctx.progress(time, 0.1, 0.7));
  ctx.fillStyle = p.accent;
  ctx.fillRect(left, top, 12, 170 * bar);
  ctx.text(p.name, left + 36, top, {
    family: p.font,
    size: 72,
    weight: 800,
    color: p.textColor,
  });
  ctx.text(p.role, left + 36, top + 96, {
    family: p.font,
    size: 44,
    weight: 500,
    color: ctx.withAlpha(p.textColor, 0.85),
  });
}
