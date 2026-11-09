export const meta = {
  name: 'Animated title',
  description: 'A centred title whose words rise in one after the other',
  defaultDuration: 4,
};

export const params = {
  title: { type: 'string', title: 'Titre', default: 'Le grand jour', maxLength: 80 },
  color: { type: 'string', format: 'color', title: 'Couleur', default: '#ffffff' },
  outline: { type: 'string', format: 'color', title: 'Contour', default: '#000000' },
  size: { type: 'number', title: 'Taille', default: 140, minimum: 40, maximum: 300 },
  stagger: {
    type: 'number',
    title: 'Décalage entre les mots (s)',
    default: 0.12,
    minimum: 0,
    maximum: 0.5,
    multipleOf: 0.01,
  },
  font: { type: 'string', format: 'font', title: 'Police', default: 'Inter' },
};

export function render(ctx, time, p) {
  const words = p.title.split(' ').filter((word) => word !== '');
  ctx.setFont({ family: p.font, size: p.size, weight: 900 });
  const space = ctx.measureText(' ').width;
  const widths = words.map((word) => ctx.measureText(word).width);
  const total = widths.reduce((sum, width) => sum + width, 0) + space * (words.length - 1);
  const exit = ctx.ease.inQuad(ctx.progress(time, ctx.duration - 0.4, ctx.duration));
  let x = (ctx.width - total) / 2;
  ctx.textBaseline = 'middle';
  ctx.lineJoin = 'round';
  ctx.lineWidth = p.size * 0.08;
  words.forEach((word, index) => {
    const t = ctx.progress(time, index * p.stagger, index * p.stagger + 0.45);
    const rise = (1 - ctx.ease.outBack(t)) * p.size * 0.6;
    ctx.globalAlpha = ctx.ease.outQuad(t) * (1 - exit);
    ctx.strokeStyle = p.outline;
    ctx.strokeText(word, x, ctx.height / 2 + rise);
    ctx.fillStyle = p.color;
    ctx.fillText(word, x, ctx.height / 2 + rise);
    x += widths[index] + space;
  });
}
