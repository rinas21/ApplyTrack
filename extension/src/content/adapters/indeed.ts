import { Adapter, ExtractedJob, text } from './types.js';

export const indeedAdapter: Adapter = {
  matches: (url) => /indeed\./.test(url),
  extract(): ExtractedJob | null {
    const title = text('[data-testid="jobsearch-JobInfoHeader-title"], h1.jobsearch-JobInfoHeader-title, h1') || document.title;
    const company = text('[data-testid="inlineHeader-companyName"], .jobsearch-InlineCompanyRating-companyHeader a, .jobsearch-CompanyInfoWithoutHeaderImage div a');
    if (!title && !company) return null;
    const location = text('[data-testid="inlineHeader-companyLocation"], .jobsearch-JobInfoHeader-subtitle div');
    const description = text('#jobDescriptionText');
    return { source: 'indeed', company_name: company, job_title: title, location, job_url: window.location.href, description: description.slice(0, 4000) };
  }
};
