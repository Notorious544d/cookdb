const FRACTIONS = [
  [1 / 8, '⅛'], [1 / 4, '¼'], [1 / 3, '⅓'], [3 / 8, '⅜'],
  [1 / 2, '½'], [5 / 8, '⅝'], [2 / 3, '⅔'], [3 / 4, '¾'], [7 / 8, '⅞'],
];

const numberValue = (n) =>
  n.type === 'regular' ? n.value : n.value.whole + n.value.num / n.value.den + n.value.err;

function fmt(x) {
  const whole = Math.floor(x + 1e-9);
  const frac = x - whole;
  if (frac < 0.02) return String(whole);
  if (frac > 0.98) return String(whole + 1);
  const hit = FRACTIONS.find(([v]) => Math.abs(v - frac) < 0.02);
  if (hit) return (whole ? String(whole) : '') + hit[1];
  return String(Math.round(x * 100) / 100);
}

export function quantityText(q, factor = 1) {
  if (!q) return '';
  const f = q.scalable ? factor : 1;
  const v = q.value;
  let amount;
  if (v.type === 'number') amount = fmt(numberValue(v.value) * f);
  else if (v.type === 'range') amount = `${fmt(numberValue(v.value.start) * f)}–${fmt(numberValue(v.value.end) * f)}`;
  else amount = String(v.value); // text values can't be scaled
  return q.unit ? `${amount} ${q.unit}` : amount;
}