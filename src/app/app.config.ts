import {
  ApplicationConfig,
  inject,
  provideAppInitializer,
} from "@angular/core";
import { provideRouter } from "@angular/router";
import { routes } from "./app.routes";
import { provideHttpClient } from "@angular/common/http";
import { ClipboardHistoryService } from "./services/clipboard-history.service";
import { ThemeService } from "./services/theme.service";
import { DropperService } from "./services/dropper.service";
import {
  provideClientHydration,
  withEventReplay,
} from "@angular/platform-browser";
import { BookmarksService } from "./services/bookmarks.service";
import { provideAnimationsAsync } from "@angular/platform-browser/animations/async";
import { provideToastr } from "ngx-toastr";
import { BackendErrorService } from "./services/backend-error.service";

export const appConfig: ApplicationConfig = {
  providers: [
    provideRouter(routes),
    provideHttpClient(),
    provideAppInitializer(() => {
      inject(ClipboardHistoryService);
      inject(BookmarksService);
      inject(ThemeService);
      inject(DropperService);
      inject(BackendErrorService);
    }),
    provideAnimationsAsync(),
    provideToastr({
      closeButton: true,
      progressBar: true,
      positionClass: "toast-bottom-right",
      preventDuplicates: true,
      timeOut: 7000,
    }),
    provideClientHydration(withEventReplay()),
  ],
};
