FROM node:lts-alpine as builder

WORKDIR /usr/src/app
COPY . .
ENV VITE_IS_DOCKER_COMPOSED=true
RUN npm run build

FROM nginx
ENV VITE_IS_DOCKER_COMPOSED=true 
COPY --from=builder /usr/src/app/dist /usr/share/nginx/html
