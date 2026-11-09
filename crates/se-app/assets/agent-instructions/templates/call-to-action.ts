export const meta = {
  name: 'Call to action',
  description: 'A pill-shaped button that pops in, pulses gently and pops out',
  defaultDuration: 3,
};

export const params = {
  label: { type: 'string', title: 'Texte', default: 'Abonne-toi', maxLength: 40 },
  color: { type: 'string', format: 'color', title: 'Couleur', default: '#ef4444' },
  textColor: { type: 'string', format: 'color', title: 'Texte', default: '#ffffff' },
  icon: { type: 'string', format: 'asset', title: 'Icône' },
  height: {
    type: 'number',
    title: 'Hauteur dans l’image (0–1)',
    default: 0.72,
    minimum: 0,
    maximum: 1,
    multipleOf: 0.01,
  },
};

export function render(ctx, time, p) {
  const pop = ctx.ease.outBack(ctx.progress(time, 0, 0.4));
  const exit = ctx.ease.inBack(ctx.progress(time, ctx.duration - 0.35, ctx.duration));
  const pulse = 1 + 0.04 * Math.sin(Math.max(0, time - 0.4) * Math.PI * 3);
  const scale = pop * (1 - exit) * pulse;
  if (scale <= 0) return;
  ctx.setFont({ size: 64, weight: 800 });
  const textWidth = ctx.measureText(p.label).width;
  const icon = p.icon === null ? 0 : 72;
  const pill = { width: textWidth + 96 + (icon > 0 ? icon + 24 : 0), height: 120 };
  ctx.translate(ctx.width / 2, ctx.height * p.height);
  ctx.scale(scale);
  ctx.shadowColor = 'rgba(0, 0, 0, 0.35)';
  ctx.shadowBlur = 24;
  ctx.shadowOffsetY = 8;
  ctx.fillStyle = p.color;
  ctx.beginPath();
  ctx.roundRect(-pill.width / 2, -pill.height / 2, pill.width, pill.height, pill.height / 2);
  ctx.fill();
  ctx.shadowColor = 'transparent';
  let x = -pill.width / 2 + 48;
  if (p.icon !== null) {
    const fit = Math.min(icon / p.icon.width, icon / p.icon.height);
    const w = p.icon.width * fit;
    const h = p.icon.height * fit;
    ctx.drawImage(p.icon, x + (icon - w) / 2, -h / 2, w, h);
    x += icon + 24;
  }
  ctx.fillStyle = p.textColor;
  ctx.textBaseline = 'middle';
  ctx.fillText(p.label, x, 4);
}
