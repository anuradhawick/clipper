import { ComponentFixture, TestBed } from '@angular/core/testing';

import { QrviewerComponent } from './qrviewer.component';

describe('QrviewerComponent', () => {
  let component: QrviewerComponent;
  let fixture: ComponentFixture<QrviewerComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [QrviewerComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(QrviewerComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
