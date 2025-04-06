import { ComponentFixture, TestBed } from '@angular/core/testing';

import { NewNoteComponent } from './new-note.component';

describe('NewNoteComponent', () => {
  let component: NewNoteComponent;
  let fixture: ComponentFixture<NewNoteComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [NewNoteComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(NewNoteComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
