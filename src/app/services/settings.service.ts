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

export interface Settings extends ThemeSettings, HistorySize {}

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
    });
    this.settings$ = this.settingsSubject.asObservable();
    this.loadInitialSettings();
  }

  private async loadInitialSettings() {
    const settings = await this.get();
    this.settingsSubject.next(settings);
  }

  async update(settings: Settings) {
    await invoke("update_settings", {
      settings: settings,
    });
    this.settingsSubject.next(settings);
  }

  async get(): Promise<Settings> {
    return await invoke<Settings>("read_settings", {});
  }

  async deleteDB() {
    await invoke("delete_db", {});
  }

  async getDBPath(): Promise<string> {
    return invoke<string>("get_db_path", {});
  }

  async getFilesPath(): Promise<string> {
    return invoke<string>("get_files_path", {});
  }
}
