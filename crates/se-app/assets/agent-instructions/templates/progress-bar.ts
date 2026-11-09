export const meta = {
  name: 'Progress bar',
  description: 'A bar filling up over the instance, with an optional percentage',
  defaultDuration: 10,
};

export const params = {
  color: { type: 'string', format: 'color', title: 'Couleur', default: '#22c55e' },
  track: { type: 'string', format: 'color', title: 'Fond', default: '#ffffff40' },
  thickness: { type: 'number', title: 'Épaisseur', default: 16, minimum: 4, maximum: 80 },
  position: {
    type: 'string',
    title: 'Position',
    enum: ['top', 'bottom'],
    enumLabels: ['En haut', 'En bas'],
    default: 'bottom',
  },
  showLabel: { type: 'boolean', title: 'Afficher le pourcentage', default: true },
};

export function render(ctx, time, p) {
  const progress = ctx.progress(time, 0, ctx.duration);
  const margin = Math.min(ctx.width, ctx.height) * 0.05;
  const width = ctx.width - margin * 2;
  const top = p.position === 'top' ? margin : ctx.height - margin - p.thickness;
  const radius = p.thickness / 2;
  ctx.globalAlpha = ctx.ease.outCubic(ctx.progress(time, 0, 0.3));
  ctx.fillStyle = p.track;
  ctx.beginPath();
  ctx.roundRect(margin, top, width, p.thickness, radius);
  ctx.fill();
  ctx.fillStyle = p.color;
  ctx.beginPath();
  ctx.roundRect(margin, top, Math.max(p.thickness, width * progress), p.thickness, radius);
  ctx.fill();
  if (!p.showLabel) return;
  const below = p.position === 'top';
  ctx.text(
    `${Math.round(progress * 100)} %`,
    margin + width,
    below ? top + p.thickness + 12 : top - 12,
    {
      size: 36,
      weight: 700,
      color: '#ffffff',
      align: 'right',
      valign: below ? 'top' : 'bottom',
    },
  );
}
