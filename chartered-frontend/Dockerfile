FROM node AS builder
ARG BASE_URL
RUN ["/bin/bash", "-c", ": ${BASE_URL:?BASE_URL must be set to the public URL that chartered-web can be reached by passing --build-arg to docker build.}"]
WORKDIR /app
COPY . /app
RUN yarn install && yarn build
ENTRYPOINT [ "/bin/sh" ]

FROM joseluisq/static-web-server
ENV SERVER_LOG_LEVEL=info SERVER_ERROR_PAGE_404=./index.html SERVER_ROOT=. SERVER_SECURITY_HEADERS=true
WORKDIR /app
COPY --from=builder /app/dist .
