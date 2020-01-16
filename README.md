# filite

A simple, light and standalone pastebin, URL shortener and file-sharing service that hosts **fi**les, redirects **li**nks and stores **te**xts.

[![GitHub Actions](https://github.com/raftario/filite/workflows/Build/badge.svg)](https://github.com/raftario/filite/actions?workflowID=Build)
[![Crates.io](https://img.shields.io/crates/v/filite.svg)](https://crates.io/crates/filite)

## Features

### What it is

* Easy to use. Installation and set-up take less than a minute and a built-in web UI is provided.
* Standalone. No external dependencies required, everything that is needed is packed into the binary.
* Light and fast. The Rust web framework [Actix](https://actix.rs) is used under the hood, providing great speed with a minimal footprint.

### What it is not

* A tracking tool. No stats are stored to increase speed and reduce resource usage, if this is what you are looking for this tool is not for you.

## Installation

1. Get the binary either from the [releases page](https://github.com/raftario/filite/releases) or [using Cargo](https://crates.io/crates/filite)
2. Run it a first time and follow the instructions
3. Edit your config file, check the [dedicated section](#config) for details
4. Run the binary again and you're good to go, just browse to http://localhost:8080 (don't forget to replace `8080` with the port specified in your config)
5. Optionally, set up a [reverse proxy](#reverse-proxy)

## Usage

When asked for a login, use whatever username you want and the password you provided during setup. Usage is pretty straightforward using the web UI, but here are some tips.

* On successful submission, the URL will be copied to clipboard. If copying to clipboard fails, it will be displayed as an alert.
* Press space in the URL input to generate a random one
* If the entered URL is already in use, the input will have a yellow outline

Details for programmatic usage are provided in [the dedicated section](#programmatic-usage).

## Planned features

* Decent test suite
* TLS support
* Simple admin page
* systemd service generation

## Config

The config follows the following format. Most of the time, the defaults are reasonable.

```toml
# Port to listen on
port = 8080
# SQLite database URL
database_url = "database.db"
# Database connection pool size
pool_size = 4
# Path to the directory where files will be stored, relative or absolute
files_dir = "files"

# Highlight.js configuration
[highlight]
# Theme to use
theme = "github"
# Additional languages to import
languages = ["rust"]
```

## Client tools

### ShareX

> To get the value of `AUTORIZATION` convert `<USERNAME>:<PASSWORD>` to base64.
> Don't forget to replace `http://example.com` with your own address.

#### File

```json
{
  "Version": "13.0.1",
  "Name": "filite (file)",
  "DestinationType": "ImageUploader, FileUploader",
  "RequestMethod": "POST",
  "RequestURL": "http://example.com/f",
  "Headers": {
    "Authorization": "Basic <AUTORIZATION>"
  },
  "Body": "MultipartFormData",
  "FileFormName": "file",
  "URL": "http://example.com/f/$response$"
}
```

#### Link

```json
{
  "Version": "13.0.1",
  "Name": "filite (link)",
  "DestinationType": "URLShortener",
  "RequestMethod": "POST",
  "RequestURL": "http://example.com/l",
  "Headers": {
    "Authorization": "Basic <AUTORIZATION>"
  },
  "Body": "JSON",
  "Data": "{\"forward\":\"$input$\"}",
  "URL": "http://example.com/l/$response$"
}
```

#### Text

> You can remove the prompt and always enable syntax highlighting by replacing `$prompt:Highlight|false$` with `true` or `false`.

```json
{
  "Version": "13.0.1",
  "Name": "filite (text)",
  "DestinationType": "TextUploader",
  "RequestMethod": "POST",
  "RequestURL": "http://example.com/t",
  "Headers": {
    "Authorization": "Basic <AUTORIZATION>"
  },
  "Body": "JSON",
  "Data": "{\"contents\":\"$input$\",\"highlight\":$prompt:Highlight|false$}",
  "URL": "http://example.com/t/$response$"
}
```

## Reverse proxy

### NGINX

> Don't forget to replace `8080` with the port specified in your config and `example.com` with your own domain.

```nginx
server {
    listen 80;
    listen [::]:80;

    server_name example.com;

    location / {
        proxy_pass http://localhost:8080;
        
        location /f {
            client_max_body_size 10M;
        }
    }
}
```

## Programmatic usage

### Posting new elements

Send a PUT request with a JSON body following the following schemes. Don't forget to set the `Content-Type` header to `application/json` and the `Authorization` header to a valid value (username isn't important).

#### File

`PUT /f/id`

```json
{
    "base64": "Base64-encoded file",
    "filename": "Filename"
}
```

#### Link

`PUT /l/id`

```json
{
    "forward": "URL to forward to"
}
```

#### Text

`PUT /t/id`

```json
{
    "contents": "Text contents"
}
```

### Getting existing elements

The response will be a JSON array following the following schemes

#### Files

`GET /f`

```json
[
    {
        "id": "ID (URL) as an integer",
        "filepath": "Absolute path to the stored file",
        "created": "Creation timestamp"
    }
]
```

#### Links

`GET /l`

```json
[
    {
        "id": "ID (URL) as an integer",
        "forward": "URL to forward to",
        "created": "Creation timestamp"
    }
]
```

#### Texts

`GET /t`

```json
[
    {
        "id": "ID (URL) as an integer",
        "contents": "Text contents",
        "created": "Creation timestamp"
    }
]
```

## Contributing

The project is open to contributions! Before submitting a PR, make sure your changes work both with and without the `dev` feature enabled.

### Requirements

* The Rust toolchain
* [diesel_cli](https://github.com/diesel-rs/diesel/tree/master/diesel_cli) with the `sqlite` feature enabled

### Setup

1. Copy [`.env.example`](./.env.example) to `.env` and set the variables to your liking
2. Run `diesel database setup`
3. Build or run with the `dev` feature enabled

## License

filite is licensed under the [MIT License](./LICENSE).
