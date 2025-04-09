import { Routes } from "@angular/router";
import { ClipboardItemsPageComponent } from "./pages/widget/clipboard-items/clipboard-items-page.component";
import { SettingsPageComponent } from "./pages/widget/settings-page/settings-page.component";
import { NotesPageComponent } from "./pages/widget/notes-page/notes-page.component";
import { NewNoteComponent } from "./pages/widget/notes-page/new-note/new-note.component";
import { FilesPageComponent } from "./pages/widget/files-page/files-page.component";
import { WidgetComponent } from "./pages/widget/widget.component";
import { ManagerComponent } from "./pages/manager/manager.component";
import { QrviewerComponent } from "./pages/qrviewer/qrviewer.component";

export const routes: Routes = [
  {
    path: "",
    redirectTo: "clipper/clipboard",
    pathMatch: "full",
  },
  {
    path: "clipper",
    component: WidgetComponent,
    children: [
      {
        path: "clipboard",
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
      {
        path: "files",
        component: FilesPageComponent,
      },
    ],
  },
  {
    path: "manager",
    component: ManagerComponent,
  },
  {
    path: "qrviewer",
    component: QrviewerComponent,
  },
];
