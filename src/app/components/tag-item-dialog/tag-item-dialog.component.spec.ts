import { ComponentFixture, TestBed } from "@angular/core/testing";
import { MAT_DIALOG_DATA, MatDialogRef } from "@angular/material/dialog";

import { TagItemDialogComponent } from "./tag-item-dialog.component";
import {
  TAG_COLORS,
  TaggedItemKind,
  TagsService,
} from "../../services/tags.service";

describe("TagItemDialogComponent", () => {
  let component: TagItemDialogComponent;
  let fixture: ComponentFixture<TagItemDialogComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [TagItemDialogComponent],
      providers: [
        {
          provide: MAT_DIALOG_DATA,
          useValue: {
            itemKind: TaggedItemKind.Clipboard,
            itemId: "test-item",
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
            tags: () => [],
            tagColors: TAG_COLORS,
            create: async () => undefined,
            readItemTags: async () => [],
            setItemTags: async () => [],
          },
        },
      ],
    }).compileComponents();

    fixture = TestBed.createComponent(TagItemDialogComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it("should create", () => {
    expect(component).toBeTruthy();
  });
});
