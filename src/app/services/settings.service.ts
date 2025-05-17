import { Injectable } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
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
export class SettingsService {
  private settingsSubject: BehaviorSubject<Settings>;
  public settings$: Observable<Settings>;

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
  }

  private async loadInitialSettings() {
    const settings = await this.get();
    this.settingsSubject.next(settings);
  }

  async update(settings: Settings) {
    await invoke("settings_update", {
      settings: settings,
    });
    this.settingsSubject.next(settings);
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
