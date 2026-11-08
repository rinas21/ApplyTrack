import { Adapter, ExtractedJob, text } from './types.js';

export const genericAdapter: Adapter = {
  matches: () => true,
  extract(): ExtractedJob | null {
    const title = text('h1') || document.title;
    const ogSite = (document.querySelector('meta[property="og:site_name"]') as HTMLMetaElement)?.content || '';
    const company = ogSite || location.hostname.replace(/^www\./, '');
    const description = (document.querySelector('meta[name="description"]') as HTMLMetaElement)?.content || text('main, article, body').slice(0, 2000);
    if (!title) return null;
    return { source: 'generic', company_name: company, job_title: title, location: '', job_url: window.location.href, description };
  }
};
