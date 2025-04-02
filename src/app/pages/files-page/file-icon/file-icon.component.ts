import { Component, inject, input, viewChild } from "@angular/core";
import {
  DropperService,
  FileEntry,
  FileType,
} from "../../../services/dropper.service";
import { MatIconModule } from "@angular/material/icon";
import { MatRippleModule } from "@angular/material/core";
import { MatMenuModule, MatMenuTrigger } from "@angular/material/menu";
import { openPath } from "@tauri-apps/plugin-opener";
import { MatDialog } from "@angular/material/dialog";
import { ActionConfirmationDialogComponent } from "../../../components/action-confirmation-dialog/action-confirmation-dialog.component";

export enum FileFormat {
  Zip = "Zip",
  Image = "Image",
  Video = "Video",
  Audio = "Audio",
  Text = "Text",
  Pdf = "Pdf",
  Unknown = "Unknown",
}

@Component({
  selector: "app-file-icon",
  imports: [MatIconModule, MatRippleModule, MatMenuModule],
  templateUrl: "./file-icon.component.html",
  styleUrl: "./file-icon.component.scss",
})
export class FileIconComponent {
  item = input.required<FileEntry>();
  menu = viewChild.required<MatMenuTrigger>(MatMenuTrigger);
  contextMenuPosition = { x: "0px", y: "0px" };
  FileType = FileType;
  FileFormat = FileFormat;
  openPath = openPath;
  readonly dialog = inject(MatDialog);
  readonly dropperService = inject(DropperService);

  deleteFile(): void {
    const dialogRef = this.dialog.open(ActionConfirmationDialogComponent, {
      data: {
        title: `Delete ${this.item().fileType === FileType.File ? "File" : "Folder"}`,
        message: `Are you sure you want to delete "${this.item().file}"?`,
      },
    });
    dialogRef.afterClosed().subscribe((result) => {
      if (result) {
        this.dropperService.deleteFile(this.item().file);
      }
    });
  }

  onRightClick(event: MouseEvent): void {
    this.contextMenuPosition.x = event.clientX + "px";
    this.contextMenuPosition.y = event.clientY + "px";
    event.preventDefault();
    this.menu().openMenu();
  }

  getFileFormat(file: string): FileFormat {
    const ext = file.split(".").pop()?.toLowerCase();
    switch (ext) {
      case "zip":
      case "gz":
      case "bz2":
        return FileFormat.Zip;
      case "jpg":
      case "jpeg":
      case "png":
      case "gif":
        return FileFormat.Image;
      case "mp4":
      case "avi":
        return FileFormat.Video;
      case "mp3":
      case "wav":
        return FileFormat.Audio;
      case "txt":
        return FileFormat.Text;
      case "pdf":
        return FileFormat.Pdf;
      default:
        return FileFormat.Unknown;
    }
  }
}
