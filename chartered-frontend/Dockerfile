FROM node AS builder
ARG VITE_CHARTERED_WEB_URL
ARG VITE_CHARTERED_SSH_URL
RUN ["/bin/bash", "-c", ": ${VITE_CHARTERED_WEB_URL:?VITE_CHARTERED_WEB_URL must be set to the public URL that chartered-web can be reached by passing --build-arg to docker build.}"]
RUN ["/bin/bash", "-c", ": ${VITE_CHARTERED_SSH_URL:?VITE_CHARTERED_SSH_URL must be set to the SSH URL that chartered-git can be reached by passing --build-arg to docker build.}"]
WORKDIR /app
COPY . /app
RUN npm install -D && npm run build
RUN echo '{"type": "module"}' > /app/build/package.json
ENTRYPOINT [ "/bin/sh" ]

FROM joseluisq/static-web-server
ENV SERVER_LOG_LEVEL=info SERVER_ERROR_PAGE_404=./index.html SERVER_ROOT=. SERVER_SECURITY_HEADERS=true
WORKDIR /app
COPY --from=builder /app/build .
