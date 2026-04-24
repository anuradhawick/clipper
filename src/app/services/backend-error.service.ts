import { Injectable, NgZone, OnDestroy } from "@angular/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
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

  constructor(
    private readonly toastr: ToastrService,
    private readonly ngZone: NgZone,
  ) {
    listen<BackendErrorPayload>("backend_error", ({ payload }) => {
      const title = payload.code || "Backend Error";
      const message =
        payload.message || "An unexpected backend error occurred.";
      this.ngZone.run(() => {
        this.toastr.error(message, title);
      });
    })
      .then((unlisten) => {
        this.unlistenBackendError = unlisten;
      })
      .catch((error) => {
        console.warn("Unable to listen for backend_error events", error);
      });
  }

  ngOnDestroy(): void {
    if (this.unlistenBackendError) {
      this.unlistenBackendError();
    }
  }
}
