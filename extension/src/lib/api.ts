export async function getConfig() {
  const d = await chrome.storage.local.get(['apiBaseUrl','accessToken']);
  return { apiBaseUrl: d.apiBaseUrl || 'http://localhost:8080/api/v1', accessToken: d.accessToken || '' };
}

export async function apiFetch(path: string, init: RequestInit = {}) {
  const cfg = await getConfig();
  const headers = new Headers(init.headers || {});
  headers.set('Content-Type', 'application/json');
  if (cfg.accessToken) headers.set('Authorization', `Bearer ${cfg.accessToken}`);
  const res = await fetch(`${cfg.apiBaseUrl}${path}`, { ...init, headers });
  const text = await res.text();
  let json: any = null; try { json = text ? JSON.parse(text) : null; } catch {}
  if (!res.ok) throw new Error(json?.message || json?.error || `API error ${res.status}`);
  return json?.data ?? json;
}

export async function captureJob(payload: any) {
  return apiFetch('/extension/capture', { method: 'POST', body: JSON.stringify(payload) });
}
export async function recentJobs() {
  return apiFetch('/extension/recent');
}
export async function checkDuplicate(jobUrl: string) {
  const jobs = await apiFetch('/jobs');
  return (jobs || []).find((j: any) => j.job_url && jobUrl && j.job_url === jobUrl);
}
