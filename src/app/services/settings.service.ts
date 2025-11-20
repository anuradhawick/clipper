import { Injectable, OnDestroy } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { BehaviorSubject, Observable } from "rxjs";

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
  historySize: number;
}

export interface GeneralSettings {
  autolaunch: boolean;
  globalShortcut: string | null;
}

export interface Settings extends ThemeSettings, HistorySize, GeneralSettings {}

@Injectable({
  providedIn: "root",
})
export class SettingsService implements OnDestroy {
  private settingsSubject: BehaviorSubject<Settings>;
  public settings$: Observable<Settings>;
  private unlistenSettingsChanged: UnlistenFn | undefined;

  constructor() {
    this.settingsSubject = new BehaviorSubject<Settings>({
      color: ColorPreference.DEFAULT,
      lighting: LightingPreference.SYSTEM,
      historySize: 100,
      autolaunch: false,
      globalShortcut: null,
    });
    this.settings$ = this.settingsSubject.asObservable();
    this.loadInitialSettings();
    this.listenForSettingsChanges();
  }

  private async loadInitialSettings() {
    const settings = await this.get();
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
