import { Injectable, signal } from "@angular/core";
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
export class NotesService {
  notes = signal<NoteItem[]>([]);

  constructor() {
    console.log("NotesService created");
    this.read();
  }

  async read() {
    console.log("Reading all notes");
    const notes = await invoke<NoteItem[]>("read_notes", {});
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
    const savedItem = await invoke<NoteItem>("create_note", { ...item });
    this.notes.update((notes) => [savedItem, ...notes]);
  }

  async copy(id: string) {
    console.log("Copying note", id);
    await invoke("clipboard_add_note", { id });
  }

  async delete(id: string) {
    console.log("Deleting note", id);
    await invoke("delete_note", { id });
    this.notes.update((notes) => notes.filter((item) => item.id !== id));
  }

  async deleteAll() {
    console.log("Deleting all notes");
    await invoke("delete_all_notes", {});
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
