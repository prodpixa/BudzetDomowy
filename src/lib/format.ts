// Grupowanie tysięcy spacją nierozdzielającą: 1234567,89 → 1 234 567,89
function group(n: string): string {
  return n.replace(/^(-?\d+)(\d{3})(,|$)/, (_m, a, b, c) => `${group(a)} ${b}${c}`);
}

/** 123456 gr → "1 234,56 zł" */
export function money(grosze: number, opts: { sign?: boolean; unit?: boolean } = {}): string {
  const { sign = false, unit = true } = opts;
  let s = group((Math.abs(grosze) / 100).toFixed(2).replace(".", ","));
  if (grosze < 0) s = "−" + s;
  else if (sign && grosze > 0) s = "+" + s;
  return unit ? `${s} zł` : s;
}

/** Kwota do pola edycji: 123456 → "1234,56" */
export function moneyInput(grosze: number): string {
  if (!grosze) return "";
  const s = (grosze / 100).toFixed(2).replace(".", ",");
  return s.endsWith(",00") ? s.slice(0, -3) : s;
}

/** Parsuje "1 234,56", "1234.5", "12" → grosze; null jeśli niepoprawne. */
export function parseMoney(input: string): number | null {
  const s = input.replace(/[\s zł]/gi, "").replace(",", ".");
  if (!/^\d+(\.\d{0,2})?$/.test(s)) return null;
  return Math.round(parseFloat(s) * 100);
}

const MONTHS = [
  "Styczeń", "Luty", "Marzec", "Kwiecień", "Maj", "Czerwiec",
  "Lipiec", "Sierpień", "Wrzesień", "Październik", "Listopad", "Grudzień",
];
const MONTHS_GEN = [
  "stycznia", "lutego", "marca", "kwietnia", "maja", "czerwca",
  "lipca", "sierpnia", "września", "października", "listopada", "grudnia",
];
const MONTHS_SHORT = ["sty", "lut", "mar", "kwi", "maj", "cze", "lip", "sie", "wrz", "paź", "lis", "gru"];
const WEEKDAYS = ["niedziela", "poniedziałek", "wtorek", "środa", "czwartek", "piątek", "sobota"];

function pad(n: number): string {
  return String(n).padStart(2, "0");
}

export function currentMonth(): string {
  const d = new Date();
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}`;
}

export function today(): string {
  const d = new Date();
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

export function shiftMonth(month: string, delta: number): string {
  const [y, m] = month.split("-").map(Number);
  const idx = y * 12 + (m - 1) + delta;
  return `${Math.floor(idx / 12)}-${pad((idx % 12) + 1)}`;
}

export function monthLabel(month: string): string {
  const [y, m] = month.split("-").map(Number);
  return `${MONTHS[m - 1]} ${y}`;
}

export function monthShort(month: string): string {
  const [y, m] = month.split("-").map(Number);
  return `${MONTHS_SHORT[m - 1]} ${String(y).slice(2)}`;
}

/** "2026-09-29" → "29 września, wtorek" */
export function dayLabel(date: string): string {
  const [y, m, d] = date.split("-").map(Number);
  const wd = new Date(y, m - 1, d).getDay();
  return `${d} ${MONTHS_GEN[m - 1]}, ${WEEKDAYS[wd]}`;
}

/** Domyślna data nowego wpisu: dziś, jeśli to wybrany miesiąc, inaczej 1. dzień miesiąca. */
export function defaultDate(month: string): string {
  const t = today();
  return t.startsWith(month) ? t : `${month}-01`;
}

export function percent(part: number, whole: number): string {
  if (!whole) return "0%";
  const p = (part / whole) * 100;
  return `${p < 10 ? p.toFixed(1).replace(".", ",") : Math.round(p)}%`;
}

/** Polska odmiana: plural(3, "wpis", "wpisy", "wpisów") → "3 wpisy" */
export function plural(n: number, one: string, few: string, many: string): string {
  const word =
    n === 1
      ? one
      : n % 10 >= 2 && n % 10 <= 4 && (n % 100 < 10 || n % 100 >= 20)
        ? few
        : many;
  return `${n} ${word}`;
}
