FROM golang:alpine

RUN apk --update add bc wget && rm -rf /var/cache/apk/*

RUN wget https://github.com/nnao45/dntk/releases/download/vvvv1.0.12/dntk-linux-amd64-v1.0.12.tar.gz

RUN tar xvfz dntk-linux-amd64-vvvv1.0.12.tar.gz

CMD ["dntk-linux-amd64/dntk"]
