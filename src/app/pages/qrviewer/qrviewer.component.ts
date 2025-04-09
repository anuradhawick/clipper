import { Component, inject, signal } from "@angular/core";
import { QRCodeComponent } from "angularx-qrcode";
import { ActivatedRoute } from "@angular/router";
import { openUrl } from "@tauri-apps/plugin-opener";

@Component({
  selector: "app-qrviewer",
  imports: [QRCodeComponent],
  templateUrl: "./qrviewer.component.html",
  styleUrl: "./qrviewer.component.scss",
})
export class QrviewerComponent {
  route = inject(ActivatedRoute);
  qrCodeValue = signal("");
  openUrl = openUrl;

  ngOnInit() {
    this.route.queryParams.subscribe((params) => {
      const url = params["url"];
      if (url) {
        this.qrCodeValue.set(url);
      }
    });
  }
}
