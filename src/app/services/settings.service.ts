import { Injectable, OnDestroy, signal } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { BehaviorSubject, Observable } from "rxjs";
import { v4 as uuidv4 } from "uuid";

export enum LightingPreference {
  SYSTEM = "system",
  LIGHT = "light",
  DARK = "dark",
}

export enum ColorPreference {
  DEFAULT = "default",
  AZURE = "azure",
  YELLOW = "yellow",
  CYAN = "cyan",
}

export interface ThemeSettings {
  color: ColorPreference;
  lighting: LightingPreference;
}

export interface HistorySize {
  clipboardHistorySize: number;
  bookmarkHistorySize: number;
}

export interface GeneralSettings {
  autolaunch: boolean;
  globalShortcut: string | null;
}

export interface ClipboardFilter {
  id: string;
  filter_regex: string;
  created_date?: string;
}

export interface Settings extends ThemeSettings, HistorySize, GeneralSettings {}

@Injectable({
  providedIn: "root",
})
export class SettingsService implements OnDestroy {
  clipboardFilters = signal<ClipboardFilter[]>([]);
  private settingsSubject: BehaviorSubject<Settings>;
  public settings$: Observable<Settings>;
  private unlistenSettingsChanged: UnlistenFn | undefined;

  constructor() {
    this.settingsSubject = new BehaviorSubject<Settings>({
      color: ColorPreference.DEFAULT,
      lighting: LightingPreference.SYSTEM,
      clipboardHistorySize: 100,
      bookmarkHistorySize: 100,
      autolaunch: false,
      globalShortcut: null,
    });
    this.settings$ = this.settingsSubject.asObservable();
    this.loadInitialSettings();
    this.loadInitialClipboardFilters();
    this.listenForSettingsChanges();
  }

  private async loadInitialSettings() {
    const settings = await this.get();
    console.log("Initial settings loaded:", settings);
    this.settingsSubject.next(settings);
  }

  private async listenForSettingsChanges() {
    // Listen for settings_changed events from other windows
    listen("settings_changed", (event: { payload: Settings }) => {
      console.log("Settings changed event received:", event.payload);
      this.settingsSubject.next(event.payload);
    }).then((func) => (this.unlistenSettingsChanged = func));
  }

  ngOnDestroy(): void {
    if (this.unlistenSettingsChanged) {
      this.unlistenSettingsChanged();
    }
  }

  async update(settings: Settings) {
    await invoke("settings_update", {
      settings: settings,
    });
    // Note: Settings will be updated via the settings_changed event listener
    // This avoids duplicate updates and ensures all windows update consistently
  }

  async get(): Promise<Settings> {
    return await invoke<Settings>("settings_read", {});
  }

  private async loadInitialClipboardFilters() {
    const filters = await this.readClipboardFilters();
    this.clipboardFilters.set(filters);
  }

  async readClipboardFilters(): Promise<ClipboardFilter[]> {
    return await invoke<ClipboardFilter[]>("filters_read_entries", {});
  }

  async createClipboardFilter(filterRegex: string): Promise<ClipboardFilter> {
    const normalizedRegex = this.normalizeRegex(filterRegex);
    const savedFilter = await invoke<ClipboardFilter>("filters_create_entry", {
      id: uuidv4(),
      filterRegex: normalizedRegex,
    });

    this.clipboardFilters.update((filters) => [savedFilter, ...filters]);
    return savedFilter;
  }

  async updateClipboardFilter(
    id: string,
    filterRegex: string,
  ): Promise<ClipboardFilter> {
    const normalizedRegex = this.normalizeRegex(filterRegex);
    const updatedFilter = await invoke<ClipboardFilter>(
      "filters_update_entry",
      {
        id,
        filterRegex: normalizedRegex,
      },
    );

    this.clipboardFilters.update((filters) =>
      filters.map((filter) => (filter.id === id ? updatedFilter : filter)),
    );
    return updatedFilter;
  }

  async deleteClipboardFilter(id: string) {
    await invoke("filters_delete_one", { id });
    this.clipboardFilters.update((filters) =>
      filters.filter((filter) => filter.id !== id),
    );
  }

  async clearClipboardFilters() {
    await invoke("filters_delete_all", {});
    this.clipboardFilters.set([]);
  }

  private normalizeRegex(filterRegex: string): string {
    const normalizedRegex = filterRegex.trim();

    if (normalizedRegex.length === 0) {
      throw new Error("Regex cannot be empty.");
    }

    try {
      new RegExp(normalizedRegex);
    } catch {
      throw new Error("Enter a valid regular expression.");
    }

    return normalizedRegex;
  }

  async deleteDB() {
    await invoke("db_delete_dbfile", {});
  }

  async getDBPath(): Promise<string> {
    return invoke<string>("db_get_dbfile_path", {});
  }

  async getFilesPath(): Promise<string> {
    return invoke<string>("files_get_storage_path", {});
  }
}
