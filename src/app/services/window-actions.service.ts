import { Injectable } from "@angular/core";
import { invoke } from "@tauri-apps/api/tauri";

@Injectable({
  providedIn: "root",
})
export class WindowActionsService {
  constructor() {}

  hideWindow() {
    invoke("hide_window", {});
  }
}
