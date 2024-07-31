import { Injectable, OnDestroy } from "@angular/core";

export enum ThemePreference {
  system = "system",
  light = "light",
  dark = "dark",
}

@Injectable({
  providedIn: "root",
})
export class ThemeService implements OnDestroy {
  private preference = ThemePreference.system;
  private darkThemeMediaQuery = window.matchMedia(
    "(prefers-color-scheme: dark)"
  );

  constructor() {
    this.darkThemeMediaQuery.addEventListener(
      "change",
      this.themeChangeListener
    );
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
}
