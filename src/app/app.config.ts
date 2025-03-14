import {
  ApplicationConfig,
  inject,
  provideAppInitializer,
  provideExperimentalZonelessChangeDetection,
} from "@angular/core";
import { provideAnimations } from "@angular/platform-browser/animations";
import { provideRouter } from "@angular/router";

import { routes } from "./app.routes";
import { provideHttpClient } from "@angular/common/http";
import { ClipboardHistoryService } from "./services/clipboard-history.service";
import { ThemeService } from "./services/theme.service";

export const appConfig: ApplicationConfig = {
  providers: [
    provideExperimentalZonelessChangeDetection(),
    provideRouter(routes),
    provideHttpClient(),
    provideAnimations(),
    provideAppInitializer(() => {
      inject(ClipboardHistoryService);
      inject(ThemeService);
    }),
  ],
};
