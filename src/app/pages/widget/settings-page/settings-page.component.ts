import { Component, inject, OnDestroy, OnInit, signal } from "@angular/core";
import { MatRippleModule } from "@angular/material/core";
import { MatIconModule } from "@angular/material/icon";
import { Color, colors, ThemeService } from "../../../services/theme.service";
import { MatSelectModule } from "@angular/material/select";
import {
  ColorPreference,
  LightingPreference,
  Settings,
  SettingsService,
} from "../../../services/settings.service";
import { MatFormFieldModule } from "@angular/material/form-field";
import { FormsModule } from "@angular/forms";
import { Subscription } from "rxjs";
import { MatButtonModule } from "@angular/material/button";
import { DropperService } from "../../../services/dropper.service";
import {
  MatCheckboxChange,
  MatCheckboxModule,
} from "@angular/material/checkbox";
import { MatInputModule } from "@angular/material/input";
import { platform } from "@tauri-apps/plugin-os";
import {
  tauriHotkeyToBrowserHotkey,
  browserHotkeyToMacSymbols,
  browserHotkeyToLinuxString,
  isValidHotkey,
  browserKeyCodesToTauriHotkey,
} from "./key-map";

@Component({
  selector: "app-settings-page",
  imports: [
    MatIconModule,
    MatButtonModule,
    MatRippleModule,
    MatSelectModule,
    MatFormFieldModule,
    MatCheckboxModule,
    FormsModule,
    MatInputModule,
  ],
  templateUrl: "./settings-page.component.html",
  styleUrl: "./settings-page.component.scss",
})
export class SettingsPageComponent implements OnInit, OnDestroy {
  colors: Color[] = colors;
  settingsSubscription: Subscription | undefined;
  settings = signal<Settings | null>(null);
  database = signal("loading...");
  filesPath = signal("loading...");
  promptedDBDelete = signal(false);
  promptedFilesDelete = signal(false);
  recordingStarted = signal(false);
  readonly themeService = inject(ThemeService);
  readonly settingsService = inject(SettingsService);
  readonly dropperService = inject(DropperService);
  // this is no needed in signal form
  pressedKeys = new Set<string>();

  ngOnInit() {
    this.settingsSubscription = this.settingsService.settings$.subscribe(
      (settings) => {
        this.settings.set(settings);
      }
    );
    this.settingsService.getDBPath().then((path) => {
      this.database.set(path);
    });
    this.settingsService.getFilesPath().then((path) => {
      this.filesPath.set(path);
    });
  }

  clearPressedKeys() {
    const settings = this.settings();
    if (!settings) return;
    this.settingsService.update({ ...settings, globalShortcut: null });
  }

  onKeydown(event: KeyboardEvent) {
    if (this.recordingStarted()) {
      this.pressedKeys.add(event.code);

      if (isValidHotkey(this.pressedKeys)) {
        const settings = this.settings();
        if (!settings) return;
        this.settingsService.update({
          ...settings,
          globalShortcut: browserKeyCodesToTauriHotkey(this.pressedKeys),
        });
        (event.target as HTMLElement).blur();
        this.pressedKeys.clear();
      }
    }
  }

  onKeyup(event: KeyboardEvent) {
    if (this.recordingStarted()) {
      this.pressedKeys.delete(event.code);
    }
  }

  async changeColor(color: ColorPreference) {
    const settings = this.settings();
    if (!settings) return;
    this.settingsService.update({ ...settings, color: color });
  }

  async changeLighting(lighting: LightingPreference) {
    const settings = this.settings();
    if (!settings) return;
    this.settingsService.update({ ...settings, lighting: lighting });
  }

  async changeHistorySize(size: number) {
    const settings = this.settings();
    if (!settings) return;
    this.settingsService.update({ ...settings, historySize: size });
  }

  async deleteDB() {
    this.database.set("deleting...");
    this.promptedDBDelete.set(false);

    await this.settingsService.deleteDB();
  }

  async deleteFiles() {
    this.promptedFilesDelete.set(false);

    await this.dropperService.deleteAllFiles();
  }

  changeAutoLaunch(event: MatCheckboxChange) {
    const settings = this.settings();
    if (!settings) return;
    this.settingsService.update({ ...settings, autolaunch: event.checked });
  }

  ngOnDestroy(): void {
    if (this.settingsSubscription) {
      this.settingsSubscription.unsubscribe();
    }
  }

  parseGlobalShortcut(globalShortcut: string) {
    const webKeys = tauriHotkeyToBrowserHotkey(globalShortcut);

    return platform() === "macos"
      ? browserHotkeyToMacSymbols(webKeys)
      : browserHotkeyToLinuxString(webKeys);
  }
}
