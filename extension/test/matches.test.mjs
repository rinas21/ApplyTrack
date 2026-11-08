import test from 'node:test';
import assert from 'node:assert/strict';
import { linkedInAdapter } from '../dist/content/adapters/linkedin.js';
import { indeedAdapter } from '../dist/content/adapters/indeed.js';
import { genericAdapter } from '../dist/content/adapters/generic.js';

test('linkedin adapter matches linkedin job URLs', () => {
  assert.equal(linkedInAdapter.matches('https://www.linkedin.com/jobs/view/123'), true);
  assert.equal(linkedInAdapter.matches('https://example.com/jobs'), false);
});

test('indeed adapter matches indeed URLs', () => {
  assert.equal(indeedAdapter.matches('https://www.indeed.com/viewjob?jk=abc'), true);
  assert.equal(indeedAdapter.matches('https://ca.indeed.com/jobs?q=rust'), true);
  assert.equal(indeedAdapter.matches('https://linkedin.com/jobs'), false);
});

test('generic adapter matches any URL as fallback', () => {
  assert.equal(genericAdapter.matches('https://company.example/careers/123'), true);
});
