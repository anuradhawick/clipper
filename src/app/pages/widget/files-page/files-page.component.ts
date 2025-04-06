import { Component, inject } from "@angular/core";
import { DropperService } from "../../../services/dropper.service";
import { FileIconComponent } from "./file-icon/file-icon.component";

@Component({
  selector: "app-files-page",
  imports: [FileIconComponent],
  templateUrl: "./files-page.component.html",
  styleUrl: "./files-page.component.scss",
})
export class FilesPageComponent {
  readonly dropperService = inject(DropperService);
}
