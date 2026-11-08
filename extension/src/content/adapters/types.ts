export interface ExtractedJob {
  source: string;
  company_name: string;
  job_title: string;
  location: string;
  job_url: string;
  description: string;
}
export interface Adapter {
  matches(url: string): boolean;
  extract(): ExtractedJob | null;
}
export function text(sel: string): string {
  return (document.querySelector(sel)?.textContent || '').trim().replace(/\s+/g, ' ');
}
