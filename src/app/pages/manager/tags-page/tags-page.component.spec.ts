import { ComponentFixture, TestBed } from "@angular/core/testing";
import { MatDialog } from "@angular/material/dialog";

import { TagsPageComponent } from "./tags-page.component";
import { TAG_COLORS, TagsService } from "../../../services/tags.service";

describe("TagsPageComponent", () => {
  let component: TagsPageComponent;
  let fixture: ComponentFixture<TagsPageComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [TagsPageComponent],
      providers: [
        {
          provide: MatDialog,
          useValue: {
            open: () => ({
              afterClosed: () => ({
                subscribe: () => undefined,
              }),
            }),
          },
        },
        {
          provide: TagsService,
          useValue: {
            tags: () => [],
            tagColors: TAG_COLORS,
            create: async () => undefined,
            update: async () => undefined,
            delete: async () => undefined,
          },
        },
      ],
    }).compileComponents();

    fixture = TestBed.createComponent(TagsPageComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it("should create", () => {
    expect(component).toBeTruthy();
  });
});
