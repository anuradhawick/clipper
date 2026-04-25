import { ComponentFixture, TestBed } from "@angular/core/testing";
import { MAT_DIALOG_DATA, MatDialogRef } from "@angular/material/dialog";

import { TagsService } from "../../../../services/tags.service";
import { TagEditDialogComponent } from "./tag-edit-dialog.component";

describe("TagEditDialogComponent", () => {
  let component: TagEditDialogComponent;
  let fixture: ComponentFixture<TagEditDialogComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [TagEditDialogComponent],
      providers: [
        {
          provide: MAT_DIALOG_DATA,
          useValue: {
            tag: {
              id: "tag-1",
              tag: "Reference",
              kind: "cyan",
              timestamp: "2026-04-25T00:00:00Z",
            },
          },
        },
        {
          provide: MatDialogRef,
          useValue: {
            close: () => undefined,
          },
        },
        {
          provide: TagsService,
          useValue: {
            update: async () => undefined,
          },
        },
      ],
    }).compileComponents();

    fixture = TestBed.createComponent(TagEditDialogComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it("should create", () => {
    expect(component).toBeTruthy();
  });
});
