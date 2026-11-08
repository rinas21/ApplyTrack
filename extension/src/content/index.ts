import { linkedInAdapter } from './adapters/linkedin.js';
import { indeedAdapter } from './adapters/indeed.js';
import { genericAdapter } from './adapters/generic.js';

const adapters = [linkedInAdapter, indeedAdapter, genericAdapter];
function extract() {
  const url = window.location.href;
  for (const a of adapters) if (a.matches(url)) { const job = a.extract(); if (job && (job.job_title || job.company_name)) return job; }
  return null;
}
chrome.runtime.onMessage.addListener((msg, _sender, sendResponse) => {
  if (msg?.type === 'APPLYTRACK_EXTRACT') sendResponse({ job: extract() });
  return true;
});
export {};
