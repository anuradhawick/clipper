import {
  Injectable,
  OnDestroy,
  Renderer2,
  RendererFactory2,
} from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { SettingsService } from "./settings.service";

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

@Injectable({
  providedIn: "root",
})
export class ThemeService implements OnDestroy {
  public selectedColor = ColorPreference.DEFAULT;
  public selectedLighting = LightingPreference.LIGHT;
  public userLightingPreference = LightingPreference.SYSTEM;
  private systemLighting!: LightingPreference;
  private renderer: Renderer2;
  private darkThemeMediaQuery = window.matchMedia(
    "(prefers-color-scheme: dark)"
  );

  constructor(rendererFactory: RendererFactory2, ss: SettingsService) {
    console.log("ThemeService created");
    this.renderer = rendererFactory.createRenderer(null, null);
    this.darkThemeMediaQuery.addEventListener(
      "change",
      this.themeChangeListener.bind(this)
    );
    if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
      console.log("System theme is dark");
      this.systemLighting = LightingPreference.DARK;
    } else {
      console.log("System theme is light");
      this.systemLighting = LightingPreference.LIGHT;
    }
    // get user preference and override if different
    ss.get().then((saved: ThemeSettings) => {
      // is saved pref is system
      if (saved.lighting === LightingPreference.SYSTEM) {
        this.selectedLighting = this.systemLighting;
      } else {
        this.selectedLighting = saved.lighting;
        this.userLightingPreference = saved.lighting;
      }
      this.selectedColor = saved.color;
      const body = document.body;
      this.renderer.addClass(
        body,
        `${this.selectedColor}-${this.selectedLighting}`
      );
    });
  }

  private themeChangeListener(event: MediaQueryListEvent): void {
    this.systemLighting = event.matches
      ? LightingPreference.DARK
      : LightingPreference.LIGHT;

    if (this.userLightingPreference === LightingPreference.SYSTEM) {
      const body = document.body;
      const themeOld = `${this.selectedColor}-${this.selectedLighting}`;
      const themeNew = `${this.selectedColor}-${this.systemLighting}`;
      this.renderer.removeClass(body, themeOld);
      this.renderer.addClass(body, themeNew);
      this.selectedLighting = this.systemLighting;
      console.log("Auto change theme", themeOld, themeNew);
    }
  }

  ngOnDestroy(): void {
    this.darkThemeMediaQuery.removeEventListener(
      "change",
      this.themeChangeListener
    );
  }

  changeColor(color: ColorPreference): ThemeSettings {
    const themeOld = `${this.selectedColor}-${this.selectedLighting}`;
    const themeNew = `${color}-${this.selectedLighting}`;
    const body = document.body;

    this.renderer.removeClass(body, themeOld);
    this.renderer.addClass(body, themeNew);
    this.selectedColor = color;

    console.log("Replace color", themeOld, themeNew);

    return { lighting: this.selectedLighting, color };
  }

  changeLighting(lighting: LightingPreference): ThemeSettings {
    this.userLightingPreference = lighting;
    const body = document.body;
    const themeOld = `${this.selectedColor}-${this.selectedLighting}`;
    let themeNew = `${this.selectedColor}-${lighting}`;

    // if users selects system lighting
    if (lighting === LightingPreference.SYSTEM) {
      themeNew = `${this.selectedColor}-${this.systemLighting}`;
      this.selectedLighting = this.systemLighting;
    } else {
      this.selectedLighting = lighting;
    }

    this.renderer.removeClass(body, themeOld);
    this.renderer.addClass(body, themeNew);

    console.log("Replace lighting", themeOld, themeNew);
    // await invoke("update_settings", {
    //   settings: { lighting, color: this.selectedColor },
    // });
    return { lighting, color: this.selectedColor };
  }
}
