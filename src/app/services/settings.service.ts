import { Injectable } from "@angular/core";
import { ThemeSettings } from "./theme.service";
import { invoke } from "@tauri-apps/api/core";

export interface Settings extends ThemeSettings {}

@Injectable({
  providedIn: "root",
})
export class SettingsService {
  async update(settings: Settings) {
    await invoke("update_settings", {
      settings: settings,
    });
  }

  async get(): Promise<Settings> {
    return await invoke<Settings>("read_settings", {});
  }
}
