import { Adapter, ExtractedJob, text } from './types.js';

export const linkedInAdapter: Adapter = {
  matches: (url) => /linkedin\.com\/jobs\//.test(url),
  extract(): ExtractedJob | null {
    const title = text('.job-details-jobs-unified-top-card__job-title, .top-card-layout__title, h1.t-24') || document.title.replace(/\s*\|.*$/, '');
    const company = text('.job-details-jobs-unified-top-card__company-name a, .job-details-jobs-unified-top-card__company-name, .topcard__org-name-link, .topcard__flavor a');
    if (!title && !company) return null;
    const location = text('.job-details-jobs-unified-top-card__primary-description-container .tvm__text, .topcard__flavor--bullet, .job-details-jobs-unified-top-card__bullet');
    const description = text('#job-details, .jobs-description__content, .show-more-less-html__markup');
    return { source: 'linkedin', company_name: company, job_title: title, location, job_url: locationHref(), description: description.slice(0, 4000) };
  }
};
function locationHref() { return window.location.href.split('?')[0]; }
