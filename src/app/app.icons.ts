import { Injectable } from "@angular/core";
import { MatIconRegistry } from "@angular/material/icon";
import { DomSanitizer } from "@angular/platform-browser";

@Injectable({
  providedIn: "root",
})
export class IconsRegistrar {
  constructor(
    private matIconRegistry: MatIconRegistry,
    private domSanitizer: DomSanitizer
  ) {}

  registerIcons() {
    this.matIconRegistry.addSvgIcon(
      "copy",
      this.domSanitizer.bypassSecurityTrustResourceUrl("/assets/icons/copy.svg")
    );
    this.matIconRegistry.addSvgIcon(
      "delete",
      this.domSanitizer.bypassSecurityTrustResourceUrl(
        "/assets/icons/delete.svg"
      )
    );
    this.matIconRegistry.addSvgIcon(
      "pause",
      this.domSanitizer.bypassSecurityTrustResourceUrl(
        "/assets/icons/pause.svg"
      )
    );
    this.matIconRegistry.addSvgIcon(
      "play",
      this.domSanitizer.bypassSecurityTrustResourceUrl("/assets/icons/play.svg")
    );
    this.matIconRegistry.addSvgIcon(
      "clearAll",
      this.domSanitizer.bypassSecurityTrustResourceUrl(
        "/assets/icons/clearAll.svg"
      )
    );
    this.matIconRegistry.addSvgIcon(
      "hide",
      this.domSanitizer.bypassSecurityTrustResourceUrl("/assets/icons/hide.svg")
    );
    this.matIconRegistry.addSvgIcon(
      "show",
      this.domSanitizer.bypassSecurityTrustResourceUrl("/assets/icons/show.svg")
    );
    this.matIconRegistry.addSvgIcon(
      "expand",
      this.domSanitizer.bypassSecurityTrustResourceUrl(
        "/assets/icons/expand.svg"
      )
    );
    this.matIconRegistry.addSvgIcon(
      "collapse",
      this.domSanitizer.bypassSecurityTrustResourceUrl(
        "/assets/icons/collapse.svg"
      )
    );
    this.matIconRegistry.addSvgIcon(
      "edit",
      this.domSanitizer.bypassSecurityTrustResourceUrl("/assets/icons/edit.svg")
    );
    this.matIconRegistry.addSvgIcon(
      "save",
      this.domSanitizer.bypassSecurityTrustResourceUrl("/assets/icons/save.svg")
    );
    this.matIconRegistry.addSvgIcon(
      "note",
      this.domSanitizer.bypassSecurityTrustResourceUrl("/assets/icons/note.svg")
    );
    this.matIconRegistry.addSvgIcon(
      "settings",
      this.domSanitizer.bypassSecurityTrustResourceUrl(
        "/assets/icons/settings.svg"
      )
    );
    this.matIconRegistry.addSvgIcon(
      "home",
      this.domSanitizer.bypassSecurityTrustResourceUrl("/assets/icons/home.svg")
    );
    this.matIconRegistry.addSvgIcon(
      "new",
      this.domSanitizer.bypassSecurityTrustResourceUrl("/assets/icons/new.svg")
    );
  }
}
