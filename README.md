# filite

> The README isn't representative of the current status of the `next` branch and will only be updated once the changes are stabilised.

A simple, light and standalone pastebin, URL shortener and file-sharing service that hosts **fi**les, redirects **li**nks and stores **te**xts.

[![GitHub Actions](https://img.shields.io/github/workflow/status/raftario/filite/Tests?label=tests)](https://github.com/raftario/filite/actions?workflowID=Tests)
[![Crates.io](https://img.shields.io/crates/v/filite.svg)](https://crates.io/crates/filite)

[Live Example](https://filite.raphaeltheriault.com) (file upload disabled and rate limited)

## Table of Contents

- [filite](#filite)
  - [Table of Contents](#table-of-contents)
  - [Features](#features)
    - [What it is](#what-it-is)
    - [What it is not](#what-it-is-not)
  - [Installation](#installation)
  - [Usage](#usage)
  - [Planned features](#planned-features)
  - [Config](#config)
  - [Client tools](#client-tools)
    - [ShareX](#sharex)
      - [File](#file)
      - [Link](#link)
      - [Text](#text)
  - [Reverse proxy](#reverse-proxy)
    - [NGINX](#nginx)
    - [Apache](#apache)
  - [Programmatic usage](#programmatic-usage)
    - [Listing existing entries](#listing-existing-entries)
    - [Creating new entries](#creating-new-entries)
      - [Files](#files)
      - [Links](#links)
      - [Texts](#texts)
    - [Deleting entries](#deleting-entries)
  - [Contributing](#contributing)
    - [Requirements](#requirements)
    - [Setup](#setup)
  - [License](#license)

## Features

### What it is

* Easy to use. Installation and set-up take less than a minute and a built-in web UI is provided.
* Standalone. No external dependencies required, everything that is needed is packed into the binary.
* Light and fast. The Rust web framework [Actix](https://actix.rs) is used under the hood, providing great speed with a minimal footprint.

### What it is not

* A tracking tool. No stats are stored to increase speed, reduce resource usage and maintain simplicity, if this is what you are looking for filite is not for you.

## Installation

1. Get the binary either from the [releases page](https://github.com/raftario/filite/releases) or [using Cargo](https://crates.io/crates/filite)
2. Run `filite init` to perform the initial setup (you can do this at any time to reset the config and password)
3. Edit your config file as you see fit (check the [dedicated section](#config) for details)
4. Run `filite`

That's it!

## Usage

When asked for a login, use whatever username you want and the password you provided during setup.
Details for programmatic usage are provided in [the dedicated section](#programmatic-usage).

## Planned features

* Decent test suite
* TLS support
* Simple admin page
* Multiple logins (?)

## Config

```toml
# Port to listen on
port = 8080
# SQLite database connection url
database_url = "database.db"
# SQLite database connection pool size
pool_size = 4
# Directory where to store static files
files_dir = "files"

# Highlight.js configuration
[highlight]
# Theme to use
theme = "github"
# Additional languages to include
languages = ["rust"]
```

## Client tools

### ShareX

- `<AUTHORIZATION>` is the result of encoding `<USERNAME>:<PASSWORD>` to base64
  - `<USERNAME>` is an arbitrary username, it doesn't matter
  - `<PASSWORD>` is the password entered during setup
- `<ADDRESS>` is the root address where the filite is running, for instance `http://localhost:8080` or `https://filite.raphaeltheriault.com`

#### File

```json
{
  "Version": "13.0.1",
  "Name": "filite (file)",
  "DestinationType": "ImageUploader, FileUploader",
  "RequestMethod": "POST",
  "RequestURL": "<ADDRESS>/f",
  "Headers": {
    "Authorization": "Basic <AUTORIZATION>"
  },
  "Body": "MultipartFormData",
  "FileFormName": "file",
  "URL": "<ADDRESS>/$response$"
}
```

#### Link

```json
{
  "Version": "13.0.1",
  "Name": "filite (link)",
  "DestinationType": "URLShortener",
  "RequestMethod": "POST",
  "RequestURL": "<ADDRESS>/l",
  "Headers": {
    "Authorization": "Basic <AUTORIZATION>"
  },
  "Body": "JSON",
  "Data": "{\"forward\":\"$input$\"}",
  "URL": "<ADDRESS>/l/$response$"
}
```

#### Text

> You can remove the prompt and always enable or disable syntax highlighting by replacing `$prompt:Highlight|false$` with `true` or `false`.

```json
{
  "Version": "13.0.1",
  "Name": "filite (text)",
  "DestinationType": "TextUploader",
  "RequestMethod": "POST",
  "RequestURL": "<ADDRESS>/t",
  "Headers": {
    "Authorization": "Basic <AUTORIZATION>"
  },
  "Body": "JSON",
  "Data": "{\"contents\":\"$input$\",\"highlight\":$prompt:Highlight|false$}",
  "URL": "<ADDRESS>/t/$response$"
}
```

## Reverse proxy

- `<DOMAIN>` is the domain the requests will be coming from, for instance `filite.raphaeltheriault.com`
- `<PORT>` is the port on which filite is listening

> Upload limits are set to 10M as an example

### NGINX

```nginx
server {
  listen 80;
  listen [::]:80;

  server_name <DOMAIN>;

  location / {
    proxy_pass http://localhost:<PORT>;

    location /f {
      client_max_body_size 10M;
    }
  }
}
```

### Apache

```apache
<VirtualHost *:80>
  ServerName <DOMAIN>

  ProxyPreserveHost On
  ProxyPass / http://localhost:<PORT>/
  ProxyPassReverse / http://localhost:<PORT>/

  <Location "/f">
    LimitRequestBody 10000000
  </Location>
</VirtualHost>
```

## Programmatic usage

> All requests that require authentication use HTTP Basic Auth (without taking the username into account).

### Listing existing entries

It's possible to get an array of all existing entries for each type with an authenticated request.

- `GET /f`
- `GET /l`
- `GET /t`

### Creating new entries

There are two ways to create new entries, `PUT` or `POST` requests.
`PUT` lets you choose the ID manually and `POST` assigns a free one automatically, but that's the only difference.
Both methods require authentication.

> `PUT` requests will overwrite any existing entry.

#### Files

- `PUT /f/{id}`
- `POST /f`

Files are sent as `multipart/form-data`. The field name isn't important but the file name needs to be included. Only one file is treated.

#### Links

- `PUT /l/{id}`
- `POST /l`

Links are sent as `application/json` according to the following schema.

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Link",
  "type": "object",
  "properties": {
    "forward": {
      "description": "URL this link forwards to",
      "type": "string"
    }
  }
}
```

#### Texts

- `PUT /t/{id}`
- `POST /t`

Texts are sent as `application/json` according to the following schema.

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Text",
  "type": "object",
  "properties": {
    "contents": {
      "description": "Text contents",
      "type": "string"
    },
    "highlight": {
      "description": "Whether to enable code highlighting or not for that text",
      "type": "boolean"
    }
  }
}
```

### Deleting entries

It's possible to delete any entry with an authenticated request.

- `DELETE /f`
- `DELETE /l`
- `DELETE /t`

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
