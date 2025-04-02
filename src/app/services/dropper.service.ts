import { Injectable, OnDestroy, signal } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export enum DragEventType {
  Started = "Started",
  Dropped = "Dropped",
  Ended = "Ended",
}

export interface DragEvent {
  eventType: DragEventType;
  paths: string[];
}

export enum FileType {
  Directory = "Directory",
  File = "File",
}

export interface FileEntry {
  file: string;
  clipperPath: string;
  fileType: FileType;
}

@Injectable({
  providedIn: "root",
})
export class DropperService implements OnDestroy {
  private unlistenDragDrop: any;
  private unlistenFiles: any;
  public inProgess = signal(false);
  public files = signal<FileEntry[]>([]);

  constructor() {
    console.log("DropperService created");

    listen("dragdrop", (event: { payload: DragEvent }) => {
      switch (event.payload.eventType) {
        case DragEventType.Started:
          this.inProgess.set(true);
          break;
        case DragEventType.Ended:
          this.inProgess.set(false);
          break;
        case DragEventType.Dropped:
          this.inProgess.set(false);
          break;
      }
    }).then((func) => (this.unlistenDragDrop = func));

    listen("files_added", (event: { payload: FileEntry[] }) => {
      this.files.update((files) => {
        const newFiles = event.payload;
        const existingFilesWithoutNewFiles = files.filter(
          (file) => !newFiles.some((newFile) => newFile.file === file.file)
        );
        return [...existingFilesWithoutNewFiles, ...newFiles];
      });
    }).then((func) => (this.unlistenFiles = func));

    invoke<FileEntry[]>("get_files").then((files) => {
      this.files.set(files);
    });
  }

  ngOnDestroy() {
    console.log("DropperService destroyed");
    this.unlistenDragDrop();
    this.unlistenFiles();
  }

  deleteFile(file: string): void {
    invoke("delete_file", { file }).then(() => {
      this.files.update((files) => {
        return files.filter((f) => f.file !== file);
      });
    });
  }
}
