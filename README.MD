# SFTP explorer
## What it is
SFTP explorer is web application used to allow user explore sftp server file through http server.
I made manually implementation for two things:
- Rust server inject for Vue3 env variables
- I manually inject cors to rocket 0.5-rc1

## What is in repo

### Backend
Backend written in Rust using rocket webserver + ssh2 package

### Frontend
Vue3 project

## Usage

Tested and used only through docker image, but also you can build it manually (check dockerfile to analyze what you need)

## Envs

* **API_PREFIX** - info for vue, where is backend (could be absolute path, for example: http://hostname:9900, or relative path, like: /api)
* **BE_PREFIX** - path prefix for backend (default in dockerfile is /api)
* **STATIC_PREFIX** - static files prefix (for vue), default is /ui
* **SFTP_HOSTNAME** - sftp destination server hostname, no default
* **SFTP_PORT** - sftp destination server port, default 22
* **SFTP_USERNAME** - sftp server username
* **SFTP_PASSWORd** - sftp server password

## Info

Dockerhub - https://hub.docker.com/r/czmoku/sftp_explorer