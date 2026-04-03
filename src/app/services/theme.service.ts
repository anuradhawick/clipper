import {
  Injectable,
  OnDestroy,
  Renderer2,
  RendererFactory2,
} from "@angular/core";
import {
  ColorPreference,
  LightingPreference,
  SettingsService,
  ThemeSettings,
} from "./settings.service";
import { Subscription } from "rxjs";
import { Event, EventType, Router } from "@angular/router";
import { Location } from "@angular/common";

export interface ColorDuo {
  from: string;
  to: string;
}

export interface Color {
  name: ColorPreference;
  light: ColorDuo;
  dark: ColorDuo;
}

export const colors: Color[] = [
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
    "(prefers-color-scheme: dark)",
  );
  private settingsSubscription: Subscription;
  private routerSubscription: Subscription;

  constructor(
    rendererFactory: RendererFactory2,
    location: Location,
    ss: SettingsService,
    router: Router,
  ) {
    console.log("ThemeService created");
    this.renderer = rendererFactory.createRenderer(null, null);
    this.darkThemeMediaQuery.addEventListener(
      "change",
      this.themeChangeListener.bind(this),
    );
    if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
      console.log("System theme is dark");
      this.systemLighting = LightingPreference.DARK;
    } else {
      console.log("System theme is light");
      this.systemLighting = LightingPreference.LIGHT;
    }
    // get user preference and override if different
    this.settingsSubscription = ss.settings$.subscribe(
      (saved: ThemeSettings) => {
        console.log("Theme changed", saved);
        this.changeColor(saved.color);
        this.changeLighting(saved.lighting);
      },
    );

    this.routerSubscription = router.events.subscribe((event: Event) => {
      switch (event.type) {
        case EventType.NavigationEnd:
          const body = document.body;
          const url = location.path();

          if (url.startsWith("/clipper")) {
            !body.classList.contains("rounded-lg") &&
              this.renderer.addClass(body, "rounded-lg");
          } else {
            body.classList.contains("rounded-lg") &&
              this.renderer.removeClass(body, "rounded-lg");
          }
          break;
      }
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
      this.themeChangeListener,
    );
    this.settingsSubscription.unsubscribe();
    this.routerSubscription.unsubscribe();
  }

  private changeColor(color: ColorPreference) {
    const themeOld = `${this.selectedColor}-${this.selectedLighting}`;
    const themeNew = `${color}-${this.selectedLighting}`;
    const body = document.body;

    this.renderer.removeClass(body, themeOld);
    this.renderer.addClass(body, themeNew);
    this.selectedColor = color;

    console.log("Replace color", themeOld, themeNew);
  }

  private changeLighting(lighting: LightingPreference) {
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
  }
}
