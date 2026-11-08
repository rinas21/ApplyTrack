import { captureJob, recentJobs, checkDuplicate } from './lib/api.js';

const $ = (id: string) => document.getElementById(id) as HTMLInputElement;
const status = document.getElementById('status')!;
const result = document.getElementById('result')!;
const recent = document.getElementById('recent')!;

async function extractFromTab() {
  status.textContent = 'Detecting job on current tab...';
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  if (!tab?.id || !tab.url) { status.textContent = 'No active tab.'; return; }
  if (!/^https?:/.test(tab.url)) { status.textContent = 'Open a job page to capture details.'; return; }
  try {
    const resp = await chrome.tabs.sendMessage(tab.id, { type: 'APPLYTRACK_EXTRACT' });
    const job = resp?.job;
    if (job) {
      $('company').value = job.company_name || '';
      $('title').value = job.job_title || '';
      $('location').value = job.location || '';
      $('url').value = job.job_url || tab.url;
      $('description').value = (job.description || '').slice(0, 4000);
      status.textContent = `Detected from ${job.source}. Review before saving.`;
      const dup = await checkDuplicate($('url').value).catch(() => null);
      if (dup) status.textContent += ' This URL may already be saved.';
    } else {
      $('url').value = tab.url;
      status.textContent = 'Could not confidently extract fields. Fill them in and save.';
    }
  } catch {
    $('url').value = tab.url;
    status.textContent = 'Content extraction unavailable on this page. Enter details manually.';
  }
}

async function loadRecent() {
  try {
    const items = await recentJobs();
    if (!items?.length) { recent.textContent = 'No recent jobs (connect extension in Settings).'; return; }
    recent.innerHTML = items.map((j: any) => `<div style="padding:4px 0;border-bottom:1px solid #26314f"><strong>${esc(j.company_name)}</strong><div class="muted">${esc(j.title)}</div></div>`).join('');
  } catch (e: any) { recent.textContent = e.message || 'Could not load recent jobs. Check API/token in Settings.'; }
}
function esc(s: string) { return String(s||'').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]!)); }

document.getElementById('refresh')!.addEventListener('click', () => { extractFromTab(); loadRecent(); });
document.getElementById('save')!.addEventListener('click', async () => {
  result.textContent = 'Saving...';
  try {
    const r = await captureJob({
      company_name: $('company').value.trim(),
      job_title: $('title').value.trim(),
      location: $('location').value.trim(),
      job_url: $('url').value.trim(),
      description: $('description').value.trim(),
      source: /linkedin/.test($('url').value) ? 'linkedin' : /indeed/.test($('url').value) ? 'indeed' : 'extension',
      mark_applied: $('applied').checked,
    });
    result.textContent = r.duplicate ? 'Already recorded (duplicate detected).' : `Saved.${r.application_id ? ' Application recorded (confirmed).' : ''}`;
    loadRecent();
  } catch (e: any) { result.textContent = e.message || 'Save failed.'; }
});

extractFromTab(); loadRecent();
export {};
