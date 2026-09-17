import { Injectable, OnDestroy, signal } from "@angular/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { v4 as uuidv4 } from "uuid";

export interface NoteItem {
  id: string;
  entry: string;
  created_time?: string;
  updated_time?: string;
}

@Injectable({
  providedIn: "root",
})
export class NotesService implements OnDestroy {
  notes = signal<NoteItem[]>([]);
  private unlistenNotesEvent: UnlistenFn | undefined;

  constructor() {
    console.log("NotesService created");
    this.read();

    listen("notes_updated", async () => {
      await this.read();
    }).then((func) => (this.unlistenNotesEvent = func));
  }

  ngOnDestroy(): void {
    if (this.unlistenNotesEvent) {
      const unlisten = this.unlistenNotesEvent;
      unlisten();
    }
  }

  async read() {
    console.log("Reading all notes");
    const notes = await invoke<NoteItem[]>("notes_read_entries", {});
    this.notes.set(notes);
  }

  async create(entry: string) {
    // not saving if trimmed length is zero
    if (entry.trim().length === 0) {
      return;
    }
    console.log("Creating note", entry);
    // otherwise save without trimming
    const item: NoteItem = { id: uuidv4(), entry };
    const savedItem = await invoke<NoteItem>("notes_create_entry", { ...item });
    this.notes.update((notes) => [savedItem, ...notes]);
  }

  async copy(id: string) {
    console.log("Copying note", id);
    await invoke("notes_clipboard_add_entry", { id });
  }

  async delete(id: string) {
    console.log("Deleting note", id);
    await invoke("notes_delete_one_entry", { id });
    this.notes.update((notes) => notes.filter((item) => item.id !== id));
  }

  async deleteAll() {
    console.log("Deleting all notes");
    await invoke("notes_delete_all_entries", {});
    this.notes.set([]);
  }

  async update(id: string, entry: string) {
    console.log("Updating note", id, entry);
    // delete if only spaces
    if (entry.trim().length === 0) {
      await this.delete(id);
      return;
    }
    // update the note with new content, no trimming
    const savedItem = await invoke<NoteItem>("notes_update_entry", { id, entry });
    this.notes.update((notes) =>
      notes.map((note) => {
        if (note.id !== id) {
          return note;
        }
        return savedItem;
      }),
    );
  }
}
