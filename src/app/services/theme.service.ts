import {
  Injectable,
  OnDestroy,
  Renderer2,
  RendererFactory2,
} from "@angular/core";

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

  constructor(rendererFactory: RendererFactory2) {
    this.renderer = rendererFactory.createRenderer(null, null);
    this.darkThemeMediaQuery.addEventListener(
      "change",
      this.themeChangeListener
    );
    if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
      console.log("System theme is dark");
      this.selectedLighting = LightingPreference.DARK;
      this.systemLighting = LightingPreference.DARK;
    } else {
      console.log("System theme is light");
      this.selectedLighting = LightingPreference.LIGHT;
      this.systemLighting = LightingPreference.LIGHT;
    }
    const body = document.body;
    this.renderer.addClass(
      body,
      `${this.selectedColor}-${this.selectedLighting}`
    );
    // get user preference and override if different
  }

  private themeChangeListener(event: MediaQueryListEvent): void {
    console.log("System theme changed:", event.matches ? "Dark" : "Light");
  }

  ngOnDestroy(): void {
    this.darkThemeMediaQuery.removeEventListener(
      "change",
      this.themeChangeListener
    );
  }

  changeColor(color: ColorPreference) {
    const themeOld = `${this.selectedColor}-${this.selectedLighting}`;
    const themeNew = `${color}-${this.selectedLighting}`;
    const body = document.body;
    console.log(themeOld, themeNew);
    this.renderer.removeClass(body, themeOld);
    this.renderer.addClass(body, themeNew);
    this.selectedColor = color;
  }

  changeLighting(lighting: LightingPreference) {
    this.userLightingPreference = lighting;
    const body = document.body;
    let themeNew = `${this.selectedColor}-${lighting}`;
    let themeOld = `${this.selectedColor}-${this.selectedLighting}`;

    if (this.selectedLighting == LightingPreference.SYSTEM) {
      themeOld = `${this.selectedColor}-${this.systemLighting}`;
    }

    if (lighting == LightingPreference.SYSTEM) {
      themeNew = `${this.selectedColor}-${this.systemLighting}`;
    }

    console.log("Replacement", themeOld, themeNew);
    this.renderer.removeClass(body, themeOld);
    this.renderer.addClass(body, themeNew);
    this.selectedLighting = lighting;
  }
}
