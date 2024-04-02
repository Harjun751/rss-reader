FROM node:lts-alpine as builder

WORKDIR /usr/src/app
ARG backend_url
ENV VITE_IS_DOCKER_COMPOSED $backend_url
COPY . .
RUN npm install
RUN npm run build

FROM nginx 
COPY --from=builder /usr/src/app/dist /usr/share/nginx/html
