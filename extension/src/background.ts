chrome.runtime.onInstalled.addListener(() => {
  chrome.storage.local.get(['apiBaseUrl'], (d) => {
    if (!d.apiBaseUrl) chrome.storage.local.set({ apiBaseUrl: 'http://localhost:8080/api/v1' });
  });
});
export {};
