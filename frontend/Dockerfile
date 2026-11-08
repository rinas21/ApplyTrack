# syntax=docker/dockerfile:1
FROM node:22-alpine AS build
WORKDIR /app
COPY package*.json ./
RUN npm ci || npm install
COPY . .
ARG API_BASE_URL=http://localhost:8080/api/v1
ARG KEYCLOAK_URL=http://localhost:8180
ARG KEYCLOAK_REALM=applytrack
ARG KEYCLOAK_CLIENT_ID=applytrack-web
RUN printf '{"apiBaseUrl":"%s","keycloakUrl":"%s","keycloakRealm":"%s","keycloakClientId":"%s"}' \
  "$API_BASE_URL" "$KEYCLOAK_URL" "$KEYCLOAK_REALM" "$KEYCLOAK_CLIENT_ID" > public/config.json
RUN npm run build -- --configuration production

FROM nginx:1.27-alpine
COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /app/dist/applytrack-frontend/browser /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
