export type ToastKind = "info" | "success" | "error";

export type Toast = {
  id: number;
  kind: ToastKind;
  message: string;
};

const DEFAULT_DURATION = 4000;
const ERROR_DURATION = 7000;

class ToastStore {
  items = $state<Toast[]>([]);
  private nextId = 1;

  show(message: string, kind: ToastKind = "info", durationMs?: number) {
    const trimmed = message.trim();
    if (!trimmed) return;
    const id = this.nextId++;
    this.items = [...this.items, { id, kind, message: trimmed }];
    const lifetime = durationMs ?? (kind === "error" ? ERROR_DURATION : DEFAULT_DURATION);
    if (lifetime > 0) {
      window.setTimeout(() => this.dismiss(id), lifetime);
    }
  }

  success(message: string, durationMs?: number) {
    this.show(message, "success", durationMs);
  }

  error(message: string, durationMs?: number) {
    this.show(message, "error", durationMs);
  }

  dismiss(id: number) {
    this.items = this.items.filter((toast) => toast.id !== id);
  }
}

export const toastStore = new ToastStore();
