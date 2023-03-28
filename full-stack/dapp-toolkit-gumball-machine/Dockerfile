FROM node:18.12.0 AS build-stage

WORKDIR /usr/app
COPY client/package.json client/package-lock.json ./
RUN npm install

COPY client ./
RUN npm run build

FROM scratch AS export-stage
COPY --from=build-stage /usr/app/dist /

FROM nginx:alpine AS prod
COPY --from=build-stage /usr/app/dist /usr/share/nginx/html