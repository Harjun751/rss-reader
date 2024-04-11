FROM node:lts-alpine as builder

WORKDIR /usr/src/app
ARG backend_url
ENV VITE_IS_DOCKER_COMPOSED $backend_url
COPY . .
RUN npm install
RUN npm run build

FROM nginx 
ARG NGINX_SERVER_NAME
ENV NGINX_SERVER_NAME $NGINX_SERVER_NAME
COPY nginx-files/nginx-selfsigned.crt /etc/ssl/certs/nginx-selfsigned.crt
COPY nginx-files/nginx-selfsigned.key /etc/ssl/private/nginx-selfsigned.key
COPY nginx-files/ssl-params.conf /etc/nginx/ssl/ssl-params.conf
COPY nginx-files/nginx.conf /etc/nginx/templates/default.conf.template
COPY --from=builder /usr/src/app/dist /usr/share/nginx/html
