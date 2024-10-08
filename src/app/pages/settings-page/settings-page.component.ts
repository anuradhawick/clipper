import { Component } from "@angular/core";
import { MatRippleModule } from "@angular/material/core";
import { MatIconModule } from "@angular/material/icon";
import {
  ColorPreference,
  LightingPreference,
  ThemeService,
} from "../../services/theme.service";
import { invoke } from "@tauri-apps/api/core";
import { SettingsService } from "../../services/settings.service";

interface ColorDuo {
  from: string;
  to: string;
}

interface Color {
  name: ColorPreference;
  light: ColorDuo;
  dark: ColorDuo;
}

@Component({
  selector: "app-settings-page",
  standalone: true,
  imports: [MatIconModule, MatRippleModule],
  templateUrl: "./settings-page.component.html",
  styleUrl: "./settings-page.component.scss",
})
export class SettingsPageComponent {
  colors: Color[] = [
    {
      name: ColorPreference.DEFAULT,
      light: { from: "from-gray-400", to: "to-gray-500" },
      dark: { from: "from-gray-700", to: "to-gray-800" },
    },
    {
      name: ColorPreference.AZURE,
      light: { from: "from-blue-400", to: "to-blue-500" },
      dark: { from: "from-blue-700", to: "to-blue-800" },
    },
    {
      name: ColorPreference.YELLOW,
      light: { from: "from-yellow-400", to: "to-yellow-500" },
      dark: { from: "from-yellow-700", to: "to-yellow-800" },
    },
    {
      name: ColorPreference.CYAN,
      light: { from: "from-cyan-400", to: "to-cyan-500" },
      dark: { from: "from-cyan-700", to: "to-cyan-800" },
    },
  ];
  selectedLighting = "system";

  constructor(protected ts: ThemeService, private ss: SettingsService) {}

  async changeColor(theme: Color) {
    const newTheme = this.ts.changeColor(theme.name);
    const settings = await this.ss.get();
    this.ss.update({ ...settings, ...newTheme });
  }

  async changeLighting(lighting: LightingPreference) {
    const newTheme = this.ts.changeLighting(lighting);
    const settings = await this.ss.get();
    this.ss.update({ ...settings, ...newTheme });
  }
}
