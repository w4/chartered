FROM node AS builder
ARG CHARTERED_WEB_URL
RUN ["/bin/bash", "-c", ": ${CHARTERED_WEB_URL:?CHARTERED_WEB_URL must be set to the public URL that chartered-web can be reached by passing --build-arg to docker build.}"]
WORKDIR /app
COPY . /app
RUN npm install -D && npm run build
RUN echo '{"type": "module"}' > /app/build/package.json
ENTRYPOINT [ "/bin/sh" ]

FROM node
WORKDIR /app
COPY --from=builder /app/build .
EXPOSE 3000
CMD [ "node", "/app/index.js" ]