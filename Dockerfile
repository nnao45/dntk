FROM golang:alpine

RUN apk --update add bc wget && rm -rf /var/cache/apk/*

RUN wget https://github.com/nnao45/dntk/releases/download/v1.0.9/dntk-linux-amd64-v1.0.9.tar.gz

RUN tar xvfz dntk-linux-amd64-v1.0.9.tar.gz

CMD ["dntk-linux-amd64/dntk"]
