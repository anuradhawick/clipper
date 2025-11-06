import { TestBed } from '@angular/core/testing';

import { WindowActionsService } from './window-actions.service';

describe('WindowActionsService', () => {
  let service: WindowActionsService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(WindowActionsService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
