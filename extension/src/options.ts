const api = document.getElementById('apiBaseUrl') as HTMLInputElement;
const token = document.getElementById('accessToken') as HTMLInputElement;
const msg = document.getElementById('msg')!;
chrome.storage.local.get(['apiBaseUrl','accessToken']).then(d => { api.value = d.apiBaseUrl || 'http://localhost:8080/api/v1'; token.value = d.accessToken || ''; });
document.getElementById('save')!.addEventListener('click', async () => {
  await chrome.storage.local.set({ apiBaseUrl: api.value.trim(), accessToken: token.value.trim() });
  msg.textContent = 'Saved.';
  setTimeout(()=>msg.textContent='', 1500);
});
export {};
