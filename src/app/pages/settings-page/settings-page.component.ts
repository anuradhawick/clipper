import { Component, OnDestroy } from "@angular/core";
import { MatRippleModule } from "@angular/material/core";
import { MatIconModule } from "@angular/material/icon";
import { Color, colors, ThemeService } from "../../services/theme.service";
import { MatSelectModule } from "@angular/material/select";
import {
  ColorPreference,
  LightingPreference,
  Settings,
  SettingsService,
} from "../../services/settings.service";
import { MatFormFieldModule } from "@angular/material/form-field";
import { FormsModule } from "@angular/forms";
import { Subscription } from "rxjs";
import { MatButtonModule } from "@angular/material/button";

@Component({
  selector: "app-settings-page",
  imports: [
    MatIconModule,
    MatButtonModule,
    MatRippleModule,
    MatSelectModule,
    MatFormFieldModule,
    FormsModule,
  ],
  templateUrl: "./settings-page.component.html",
  styleUrl: "./settings-page.component.scss",
})
export class SettingsPageComponent implements OnDestroy {
  colors: Color[] = colors;
  settingsSubscription: Subscription;
  settings?: Settings;
  database: string = "loading...";
  promptedDBDelete = false;

  constructor(protected ts: ThemeService, private ss: SettingsService) {
    this.settingsSubscription = this.ss.settings$.subscribe((settings) => {
      this.settings = settings;
    });
    this.ss.getDBPath().then((path) => {
      this.database = path;
    });
  }

  async changeColor(color: ColorPreference) {
    const settings = this.settings;
    if (!settings) return;
    this.ss.update({ ...settings, color: color });
  }

  async changeLighting(lighting: LightingPreference) {
    const settings = this.settings;
    if (!settings) return;
    this.ss.update({ ...settings, lighting: lighting });
  }

  async changeHistorySize(size: number) {
    const settings = this.settings;
    if (!settings) return;
    this.ss.update({ ...settings, historySize: size });
  }

  async deleteDB() {
    this.database = "deleting...";
    this.promptedDBDelete = false;

    await this.ss.deleteDB();
  }

  ngOnDestroy(): void {}
}
