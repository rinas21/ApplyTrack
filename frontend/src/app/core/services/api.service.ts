import { Injectable } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import { Observable, map } from 'rxjs';
import { ConfigService } from './config.service';

export interface ApiResponse<T> { data: T; }

@Injectable({ providedIn: 'root' })
export class ApiService {
  constructor(private http: HttpClient, private cfg: ConfigService) {}
  private base(path: string) { return `${this.cfg.value.apiBaseUrl}${path}`; }
  get<T>(path: string, params?: Record<string, any>): Observable<T> {
    let hp = new HttpParams();
    if (params) for (const [k,v] of Object.entries(params)) if (v !== undefined && v !== null && v !== '') hp = hp.set(k, String(v));
    return this.http.get<ApiResponse<T> | T>(this.base(path), { params: hp }).pipe(map((r: any) => r?.data ?? r));
  }
  post<T>(path: string, body: any): Observable<T> {
    return this.http.post<ApiResponse<T> | T>(this.base(path), body).pipe(map((r: any) => r?.data ?? r));
  }
  put<T>(path: string, body: any): Observable<T> {
    return this.http.put<ApiResponse<T> | T>(this.base(path), body).pipe(map((r: any) => r?.data ?? r));
  }
  patch<T>(path: string, body: any): Observable<T> {
    return this.http.patch<ApiResponse<T> | T>(this.base(path), body).pipe(map((r: any) => r?.data ?? r));
  }
  delete<T>(path: string): Observable<T> {
    return this.http.delete<ApiResponse<T> | T>(this.base(path)).pipe(map((r: any) => r?.data ?? r));
  }
}
