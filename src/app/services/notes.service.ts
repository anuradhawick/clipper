import { Injectable, signal } from "@angular/core";
import { invoke } from "@tauri-apps/api/tauri";
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
export class NotesService {
  notes = signal<NoteItem[]>([]);

  constructor() {
    this.read();
  }

  async read() {
    const notes = await invoke<NoteItem[]>("read_notes", {});
    console.log(notes);
    this.notes.set(notes);
  }

  async create(entry: string) {
    // not saving if trimmed length is zero
    if (entry.trim().length === 0) {
      return;
    }
    // otherwise save without trimming
    const item: NoteItem = { id: uuidv4(), entry };
    const savedItem = await invoke<NoteItem>("create_note", { ...item });
    this.notes.update((notes) => [savedItem, ...notes]);
  }

  async copy(entry: string) {
    await invoke("clipboard_add_entry", { entry: entry });
  }

  async delete(id: string) {
    await invoke("delete_note", { id });
    this.notes.update((notes) => notes.filter((item) => item.id !== id));
  }

  async update(id: string, entry: string) {
    // delete is only spaces
    if (entry.trim().length === 0) {
      await this.delete(id);
      return;
    }
    // update the note with new content, no trimming
    const savedItem = await invoke<NoteItem>("update_note", { id, entry });
    this.notes.update((notes) =>
      notes.map((note) => {
        if (note.id !== id) {
          return note;
        }
        return savedItem;
      })
    );
  }
}
