import { APP_INITIALIZER, ApplicationConfig } from "@angular/core";
import { provideAnimations } from "@angular/platform-browser/animations";
import { provideRouter } from "@angular/router";

import { routes } from "./app.routes";
import { provideHttpClient } from "@angular/common/http";
import { ClipboardHistoryService } from "./services/clipboard-history.service";
import { ThemeService } from "./services/theme.service";

export const appConfig: ApplicationConfig = {
  providers: [
    provideRouter(routes),
    provideHttpClient(),
    provideAnimations(),
    ClipboardHistoryService,
    ThemeService,
    {
      provide: APP_INITIALIZER,
      deps: [ThemeService],
      useFactory: (ts: ThemeService) => () => ts,
      multi: true,
    },
    {
      provide: APP_INITIALIZER,
      deps: [ClipboardHistoryService],
      useFactory: (chs: ClipboardHistoryService) => () => chs,
      multi: true,
    },
  ],
};
