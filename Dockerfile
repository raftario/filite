FROM rust:alpine AS build

RUN USER=root cargo new --bin filite
WORKDIR ./filite
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

RUN apk update \
    && apk add --no-cache musl-dev \
    && rm -rf /var/cache/apk/*

RUN cargo build --release

RUN rm target/release/deps/filite*
COPY . ./

RUN cargo build --release


FROM alpine

ARG APP=/usr/src/app

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN addgroup -S $APP_USER \
    && adduser -S -g $APP_USER $APP_USER

RUN apk update \
    && apk add --no-cache ca-certificates tzdata \
    && rm -rf /var/cache/apk/*

COPY --from=build /filite/target/release/filite ${APP}/filite

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./filite"]
