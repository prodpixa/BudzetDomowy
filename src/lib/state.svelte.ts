import { api, type Category, type Expense, type Income, type Source } from "./api";
import { currentMonth } from "./format";

export type View = "dashboard" | "expenses" | "incomes" | "planned" | "vouchers" | "settings";
export type Theme = "system" | "light" | "dark";

interface Toast {
  id: number;
  text: string;
  kind: "ok" | "error" | "info";
}

interface ConfirmRequest {
  title: string;
  text: string;
  confirmLabel: string;
  resolve: (ok: boolean) => void;
}

/** Formularz wydatku/przychodu otwierany globalnie (np. z listy planowanych). */
export type EntryEditor =
  | { kind: "expense"; item: Expense; onSaved?: () => void }
  | { kind: "income"; item: Income; onSaved?: () => void };

function initialTheme(): Theme {
  const t = localStorage.getItem("theme");
  return t === "light" || t === "dark" ? t : "system";
}

class AppState {
  view = $state<View>("dashboard");
  month = $state(currentMonth());
  theme = $state<Theme>(initialTheme());
  categories = $state<Category[]>([]);
  sources = $state<Source[]>([]);
  /** Zwiększane po każdej zmianie danych – widoki przeładowują się w $effect. */
  version = $state(0);
  toasts = $state<Toast[]>([]);
  confirmRequest = $state<ConfirmRequest | null>(null);
  editor = $state<EntryEditor | null>(null);

  categoryById = $derived(new Map(this.categories.map((c) => [c.id, c])));
  sourceById = $derived(new Map(this.sources.map((s) => [s.id, s])));

  async loadDictionaries() {
    const [categories, sources] = await Promise.all([api.listCategories(), api.listSources()]);
    this.categories = categories;
    this.sources = sources;
  }

  changed() {
    this.version++;
  }

  setTheme(theme: Theme) {
    this.theme = theme;
    if (theme === "system") {
      localStorage.removeItem("theme");
      delete document.documentElement.dataset.theme;
    } else {
      localStorage.setItem("theme", theme);
      document.documentElement.dataset.theme = theme;
    }
  }

  toast(text: string, kind: Toast["kind"] = "ok") {
    const id = Date.now() + Math.random();
    this.toasts.push({ id, text, kind });
    setTimeout(() => (this.toasts = this.toasts.filter((t) => t.id !== id)), 3200);
  }

  error(e: unknown) {
    this.toast(typeof e === "string" ? e : String((e as Error)?.message ?? e), "error");
  }

  confirm(title: string, text: string, confirmLabel = "Usuń"): Promise<boolean> {
    return new Promise((resolve) => {
      this.confirmRequest = { title, text, confirmLabel, resolve };
    });
  }

  /** Wykonuje zmianę danych, pokazuje błąd w razie niepowodzenia i odświeża widoki. */
  async mutate<T>(fn: () => Promise<T>, okText?: string): Promise<T | undefined> {
    try {
      const r = await fn();
      this.changed();
      if (okText) this.toast(okText);
      return r;
    } catch (e) {
      this.error(e);
      return undefined;
    }
  }
}

export const app = new AppState();
