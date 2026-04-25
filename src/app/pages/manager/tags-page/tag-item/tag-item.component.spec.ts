import { ComponentFixture, TestBed } from "@angular/core/testing";

import { TagEntry } from "../../../../services/tags.service";
import { TagItemComponent } from "./tag-item.component";

describe("TagItemComponent", () => {
  let component: TagItemComponent;
  let fixture: ComponentFixture<TagItemComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [TagItemComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(TagItemComponent);
    fixture.componentRef.setInput("tag", {
      id: "tag-1",
      tag: "Reference",
      kind: "cyan",
      timestamp: "2026-04-25T00:00:00Z",
    } satisfies TagEntry);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it("should create", () => {
    expect(component).toBeTruthy();
  });
});
