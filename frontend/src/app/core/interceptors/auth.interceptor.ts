import { HttpErrorResponse, HttpInterceptorFn } from '@angular/common/http';
import { inject } from '@angular/core';
import { Router } from '@angular/router';
import { catchError, throwError } from 'rxjs';
import { AuthService } from '../auth/auth.service';
import { ConfigService } from '../services/config.service';

export const authInterceptor: HttpInterceptorFn = (req, next) => {
  const auth = inject(AuthService);
  const cfg = inject(ConfigService);
  const router = inject(Router);
  const token = auth.getAccessToken();
  let isApi = req.url.startsWith('/api');
  try {
    const apiBase = cfg.value.apiBaseUrl;
    isApi = isApi || req.url.startsWith(apiBase);
    if (token && isApi) {
      req = req.clone({ setHeaders: { Authorization: `Bearer ${token}` } });
    }
  } catch { /* config may not be loaded for static assets */ }
  return next(req).pipe(
    catchError((err: unknown) => {
      if (isApi && err instanceof HttpErrorResponse && err.status === 401) {
        const here = router.url || '/';
        if (!here.startsWith('/login') && !here.startsWith('/callback')) {
          auth.clearSession();
          router.navigate(['/login'], { queryParams: { returnUrl: here } });
        }
      }
      return throwError(() => err);
    })
  );
};
