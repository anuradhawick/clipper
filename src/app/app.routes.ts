import { Routes } from "@angular/router";
import { ClipboardItemsPageComponent } from "./pages/clipboard-items/clipboard-items-page.component";
import { SettingsPageComponent } from "./pages/settings-page/settings-page.component";
import { NotesPageComponent } from "./pages/notes-page/notes-page.component";
import { NewNoteComponent } from "./pages/notes-page/new-note/new-note.component";

export const routes: Routes = [
  {
    path: "",
    component: ClipboardItemsPageComponent,
  },
  {
    path: "settings",
    component: SettingsPageComponent,
  },
  {
    path: "notes",
    component: NotesPageComponent,
    children: [
      {
        path: "new",
        component: NewNoteComponent,
      },
    ],
  },
];
