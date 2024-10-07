import { Injectable } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";

@Injectable({
  providedIn: "root",
})
export class WindowActionsService {
  constructor() {
    console.log("WindowActionsService created");
  }

  hideWindow() {
    invoke("hide_window", {});
  }
}
