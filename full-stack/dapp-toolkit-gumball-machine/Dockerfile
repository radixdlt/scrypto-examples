FROM node:18.12.0 AS build-stage

WORKDIR /usr/app
COPY client ./client

RUN npm install

RUN npm run build

FROM scratch AS export-stage
COPY --from=build-stage /usr/app/build /

FROM nginx:alpine AS prod
COPY --from=build-stage /usr/app/build /usr/share/nginx/html