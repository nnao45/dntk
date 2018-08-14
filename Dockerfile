FROM golang:alpine

RUN apk --update add bc wget && rm -rf /var/cache/apk/*

RUN wget https://github.com/nnao45/dntk/releases/download/vvv1.0.11/dntk-linux-amd64-vvv1.0.11.tar.gz

RUN tar xvfz dntk-linux-amd64-vvv1.0.11.tar.gz

CMD ["dntk-linux-amd64/dntk"]
