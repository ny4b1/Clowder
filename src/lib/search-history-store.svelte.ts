const STORAGE_KEY = "clowder.search-history";
const MAX_ITEMS = 20;

function loadInitial(): string[] {
  if (typeof localStorage === "undefined") return [];
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((v): v is string => typeof v === "string").slice(0, MAX_ITEMS);
  } catch {
    return [];
  }
}

class SearchHistoryStore {
  items = $state<string[]>(loadInitial());

  push(query: string) {
    const trimmed = query.trim();
    if (!trimmed) return;
    const filtered = this.items.filter((v) => v !== trimmed);
    this.items = [trimmed, ...filtered].slice(0, MAX_ITEMS);
    this.persist();
  }

  private persist() {
    if (typeof localStorage === "undefined") return;
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(this.items));
    } catch {
      // storage full or disabled
    }
  }
}

export const searchHistoryStore = new SearchHistoryStore();
