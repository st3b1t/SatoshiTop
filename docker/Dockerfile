FROM node:18.15.0-alpine

RUN apk --no-cache add procps
ENV LANG=en_US.utf8 \
    TERM=xterm-256color \
    RPCCONNECT=172.17.0.1

COPY lib lib
COPY bin bin
COPY test test
COPY package.json .
COPY package-lock.json .

RUN npm install --production

ENTRYPOINT ["./bin/satop"]
