import {
  ChangeDetectionStrategy,
  Component,
  inject,
  OnDestroy,
  OnInit,
  signal,
} from "@angular/core";
import { MatRippleModule } from "@angular/material/core";
import { MatIconModule } from "@angular/material/icon";
import { Color, colors, ThemeService } from "../../../services/theme.service";
import { MatSelectModule } from "@angular/material/select";
import {
  ClipboardFilter,
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
import { MatTooltipModule } from "@angular/material/tooltip";
import { platform } from "@tauri-apps/plugin-os";
import {
  tauriHotkeyToBrowserHotkey,
  browserHotkeyToMacSymbols,
  browserHotkeyToLinuxString,
  isValidHotkey,
  browserKeyCodesToTauriHotkey,
} from "./key-map";
import { TitleCasePipe } from "@angular/common";

@Component({
  selector: "app-settings-page",
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    MatIconModule,
    MatButtonModule,
    MatRippleModule,
    MatSelectModule,
    MatFormFieldModule,
    MatCheckboxModule,
    FormsModule,
    MatInputModule,
    MatTooltipModule,
    TitleCasePipe,
  ],
  templateUrl: "./settings-page.component.html",
  styleUrl: "./settings-page.component.scss",
})
export class SettingsPageComponent implements OnInit, OnDestroy {
  readonly themeService = inject(ThemeService);
  readonly settingsService = inject(SettingsService);
  readonly dropperService = inject(DropperService);
  colors: Color[] = colors;
  settingsSubscription: Subscription | undefined;
  settings = signal<Settings | null>(null);
  clipboardFilters = this.settingsService.clipboardFilters;
  clipboardFilterDraft = signal("");
  clipboardFilterError = signal("");
  database = signal("loading...");
  filesPath = signal("loading...");
  promptedFiltersDelete = signal(false);
  promptedDBDelete = signal(false);
  promptedFilesDelete = signal(false);
  recordingStarted = signal(false);
  // this is no needed in signal form
  pressedKeys = new Set<string>();

  ngOnInit() {
    this.settingsSubscription = this.settingsService.settings$.subscribe(
      (settings) => {
        this.settings.set(settings);
      },
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
    this.settingsService.update({ ...settings, clipboardHistorySize: size });
  }

  async changeBookmarkHistorySize(size: number) {
    const settings = this.settings();
    if (!settings) return;
    this.settingsService.update({ ...settings, bookmarkHistorySize: size });
  }

  setClipboardFilterDraft(value: string) {
    this.clipboardFilterDraft.set(value);
    if (this.clipboardFilterError()) {
      this.clipboardFilterError.set("");
    }
  }

  async addClipboardFilter() {
    try {
      await this.settingsService.createClipboardFilter(
        this.clipboardFilterDraft(),
      );
      this.clipboardFilterDraft.set("");
      this.clipboardFilterError.set("");
    } catch (error) {
      this.clipboardFilterError.set(this.getErrorMessage(error));
    }
  }

  async deleteClipboardFilter(filter: ClipboardFilter) {
    await this.settingsService.deleteClipboardFilter(filter.id);
  }

  async clearClipboardFilters() {
    this.promptedFiltersDelete.set(false);
    await this.settingsService.clearClipboardFilters();
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

  private getErrorMessage(error: unknown) {
    if (error instanceof Error) {
      return error.message;
    }

    if (typeof error === "string") {
      return error;
    }

    return "Unable to save regex.";
  }
}
