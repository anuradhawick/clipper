import { ComponentFixture, TestBed } from "@angular/core/testing";

import { ActionConfirmationDialogComponent } from "./action-confirmation-dialog.component";

describe("ActionConfirmationDialogComponent", () => {
  let component: ActionConfirmationDialogComponent;
  let fixture: ComponentFixture<ActionConfirmationDialogComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ActionConfirmationDialogComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(ActionConfirmationDialogComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it("should create", () => {
    expect(component).toBeTruthy();
  });
});
