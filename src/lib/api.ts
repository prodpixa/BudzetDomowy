import { invoke } from "@tauri-apps/api/core";

// Wszystkie kwoty są w groszach (liczby całkowite).

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

  exportCsv: (dir: string) => invoke<string[]>("export_csv", { dir }),
  backupDb: (path: string) => invoke<void>("backup_db", { path }),
  restoreDb: (path: string) => invoke<void>("restore_db", { path }),
};
