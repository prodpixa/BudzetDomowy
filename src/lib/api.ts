// Komunikacja z serwerem. Każda operacja to POST /api/<nazwa> z argumentami w JSON.
// Wszystkie kwoty są w groszach (liczby całkowite).

/** Serwer odpowiedział 401 – sesja wygasła lub użytkownik się wylogował. */
export class UnauthorizedError extends Error {}

let onUnauthorized: () => void = () => {};
export function setUnauthorizedHandler(fn: () => void) {
  onUnauthorized = fn;
}

async function request<T>(url: string, init: RequestInit): Promise<T> {
  let res: Response;
  try {
    res = await fetch(url, { credentials: "same-origin", ...init });
  } catch {
    throw new Error("Brak połączenia z serwerem");
  }
  if (res.status === 401 && !url.endsWith("/auth/login")) {
    onUnauthorized();
    throw new UnauthorizedError("Zaloguj się ponownie");
  }
  const body = await res.json().catch(() => null);
  if (!res.ok) throw new Error(body?.error ?? `Błąd serwera (${res.status})`);
  return body as T;
}

function invoke<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
  return request<T>(`/api/${command}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(args),
  });
}

export interface Category {
  id: number | null;
  name: string;
  color: number;
  icon: string;
}

export interface Source {
  id: number | null;
  name: string;
}

export interface Expense {
  id: number | null;
  name: string;
  amount: number;
  date: string;
  categoryId: number | null;
  note: string;
}

export interface Income {
  id: number | null;
  name: string;
  amount: number;
  date: string;
  sourceId: number | null;
  note: string;
}

export type PlannedKind = "expense" | "income";

export interface Planned {
  id: number | null;
  kind: PlannedKind;
  month: string;
  name: string;
  amount: number;
  note: string;
  done: boolean;
}

export type VoucherKind = "topup" | "spend";

export interface Voucher {
  id: number | null;
  kind: VoucherKind;
  place: string;
  amount: number;
  date: string;
  note: string;
}

export interface VoucherSummary {
  balance: number;
  carriedOver: number;
  monthTopups: number;
  monthSpends: number;
}

export interface GroupTotal {
  id: number | null;
  name: string;
  color: number;
  icon: string;
  total: number;
}

export interface PlannedTotals {
  total: number;
  remaining: number;
  count: number;
  doneCount: number;
}

export interface MonthSummary {
  carriedOver: number;
  incomeTotal: number;
  expenseTotal: number;
  balance: number;
  endBalance: number;
  byCategory: GroupTotal[];
  bySource: GroupTotal[];
  plannedExpenses: PlannedTotals;
  plannedIncomes: PlannedTotals;
}

export interface TrendPoint {
  month: string;
  income: number;
  expense: number;
}

export interface CopyResult {
  fromMonth: string | null;
  copied: number;
  skipped: number;
}

export const api = {
  listCategories: () => invoke<Category[]>("list_categories"),
  saveCategory: (item: Category) => invoke<number>("save_category", { item }),
  deleteCategory: (id: number) => invoke<void>("delete_category", { id }),

  listSources: () => invoke<Source[]>("list_sources"),
  saveSource: (item: Source) => invoke<number>("save_source", { item }),
  deleteSource: (id: number) => invoke<void>("delete_source", { id }),

  listExpenses: (month: string) => invoke<Expense[]>("list_expenses", { month }),
  saveExpense: (item: Expense) => invoke<number>("save_expense", { item }),
  deleteExpense: (id: number) => invoke<void>("delete_expense", { id }),

  listIncomes: (month: string) => invoke<Income[]>("list_incomes", { month }),
  saveIncome: (item: Income) => invoke<number>("save_income", { item }),
  deleteIncome: (id: number) => invoke<void>("delete_income", { id }),

  listPlanned: (month: string, kind: PlannedKind) =>
    invoke<Planned[]>("list_planned", { month, kind }),
  savePlanned: (item: Planned) => invoke<number>("save_planned", { item }),
  setPlannedDone: (id: number, done: boolean) => invoke<void>("set_planned_done", { id, done }),
  deletePlanned: (id: number) => invoke<void>("delete_planned", { id }),
  copyPlannedFromPrevious: (month: string, kind: PlannedKind) =>
    invoke<CopyResult>("copy_planned_from_previous", { month, kind }),

  listVouchers: (month: string) => invoke<Voucher[]>("list_vouchers", { month }),
  saveVoucher: (item: Voucher) => invoke<number>("save_voucher", { item }),
  deleteVoucher: (id: number) => invoke<void>("delete_voucher", { id }),
  voucherSummary: (month: string) => invoke<VoucherSummary>("voucher_summary", { month }),

  monthSummary: (month: string) => invoke<MonthSummary>("month_summary", { month }),
  monthTrend: (month: string, count: number) =>
    invoke<TrendPoint[]>("month_trend", { month, count }),

  getOpeningBalance: () => invoke<number>("get_opening_balance"),
  setOpeningBalance: (amount: number) => invoke<void>("set_opening_balance", { amount }),

  // Pliki: przeglądarka pobiera je zwykłym linkiem (ciasteczko sesji idzie automatycznie).
  exportUrl: "/api/export.zip",
  backupUrl: "/api/backup.db",
  restoreDb: (file: File) =>
    request<void>("/api/restore", {
      method: "POST",
      headers: { "Content-Type": "application/octet-stream" },
      body: file,
    }),

  me: () => request<{ username: string }>("/api/auth/me", { method: "GET" }),
  login: (username: string, password: string) =>
    request<{ username: string }>("/api/auth/login", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ username, password }),
    }),
  logout: () => request<void>("/api/auth/logout", { method: "POST" }),
};
