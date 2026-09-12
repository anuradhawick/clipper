import { Injectable, NgZone, OnDestroy } from "@angular/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { ToastrService } from "ngx-toastr";

interface BackendErrorPayload {
  code: string;
  message: string;
}

@Injectable({
  providedIn: "root",
})
export class BackendErrorService implements OnDestroy {
  private unlistenBackendError: UnlistenFn | undefined;
  private destroyed = false;

  constructor(
    private readonly toastr: ToastrService,
    private readonly ngZone: NgZone,
  ) {
    console.log("BackendErrorService created");
    this.initialize();
  }

  private initialize(): void {
    const receivedDuringStartup = new Set<string>();
    let readingStartup = true;
    let startupErrorKey: string | undefined;
    const key = (payload: BackendErrorPayload) => JSON.stringify(payload);
    listen<BackendErrorPayload>("backend_error", ({ payload }) => {
      const errorKey = key(payload);
      if (errorKey === startupErrorKey) return;
      if (readingStartup) receivedDuringStartup.add(errorKey);
      this.showError(payload);
    })
      .then((unlisten) => {
        if (this.destroyed) {
          unlisten();
          return;
        }
        this.unlistenBackendError = unlisten;
        return invoke<BackendErrorPayload | null>("backend_read_startup_error");
      })
      .then((startupError) => {
        if (startupError && !this.destroyed) {
          startupErrorKey = key(startupError);
          if (!receivedDuringStartup.has(startupErrorKey)) {
            this.showError(startupError);
          }
        }
      })
      .catch((error) => {
        console.warn("Unable to initialize backend error reporting", error);
      })
      .finally(() => {
        readingStartup = false;
        receivedDuringStartup.clear();
      });
  }

  private showError(payload: BackendErrorPayload): void {
    if (this.destroyed) return;
    const title = payload.code || "Backend Error";
    const message = payload.message || "An unexpected backend error occurred.";
    const timeOut = payload.code === "DBERROR" ? 9999999 : 7000;
    console.error("Backend error received:", payload);
    this.ngZone.run(() => {
      this.toastr.error(message, title, {
        timeOut,
        extendedTimeOut: timeOut,
      });
    });
  }

  ngOnDestroy(): void {
    this.destroyed = true;
    if (this.unlistenBackendError) {
      this.unlistenBackendError();
    }
  }
}
