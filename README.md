# Overview

Exclude unwanted channels based on group and generate a new M3U file with proxied streams and attributes. Updates the playlist on specified hourly frequency. Streams get proxied through a webserver running on the app. XtreamCodes support (currently work in progress).

## Setup

### _Settable environment variables_

| Variable                | Default     | Required | Type     | Description                                                                            |
| ----------------------- | ----------- | -------- | -------- | -------------------------------------------------------------------------------------- |
| DATABASE_URL            | -           | Yes      | string   | Connection string to DB                                                                |
| M3U                     | -           | Yes      | string   | URL to the M3U playlist (.m3u)                                                         |
| INIT_APP                | true        | No       | boolean  | Initialize app with M3U playlist from environment variable.                            |
| HOURLY_UPDATE_FREQUENCY | 12          | No       | number   | Frequency of provider playlist update in hours                                         |
| GROUP_EXCLUDES          | -           | No       | string   | A comma separated list of groups to exclude from the final playlist. Case-insensitive. |
| PROXY_DOMAIN            | -           | Yes      | string   | Domain on which the app is running - to proxy m3u requests. (Example: localhost:3000)  |
| ENV                     | Development | No       | string   | Set environment Development or Production.                                             |
| PORT                    | 3001        | No       | number   | Port to run on (Default 3001)                                                          |
| RUST_LOG                | info        | Yes      | string   | Log level (warn,server=warn,iptv=info,api=warn rest-client=warn)                       |
| XTREAM_ENABLED          | false       | No       | boolean  | Enable Xtream                                                                          |
| XTREAM_BASE_DOMAIN      | -           | No       | string   | Xtream provider base domain                                                            |
| XTREAM_USERNAME         | -           | No       | string   | Xtream provider username                                                               |
| XTREAM_PASSWORD         | -           | No       | string   | Xtream provider username                                                               |
| XTREAM_PROXIED_USERNAME | -           | No       | string   | Proxied Xtream username                                                                |
| XTREAM_PROXIED_PASSWORD | -           | No       | string   | Proxied Xtream password                                                                |    

### _Development_

Set required environment variables in and ```.env ``` file and then run `docker-compose up`.

This will fire up a MariaDB container instance and create a new DB.
Which in turn will create tables and basic data using sql files in the `server -> db -> migrations` folder.
<br/>

### _Production_

To build and run a production docker image.

In src/server folder: `docker build -t image:tag .`

Create an external docker network with the name ```db``` and run your mariadb/mysql database there. 

Example of production ```docker-compose.yml```

```
version: '3.7'

services:
  iptvhandler:
    image: iptv-handler
    tty: true # color terminal output
    ports:
      - "3001:3001"
    restart: always
    environment:
      DATABASE_URL: mysql://username:password@db:3306/iptvhandler
      M3U: http://iptvprovider.tv:1234/get.php?username=username&password=password&type=m3u_plus&output=ts
      RUST_LOG: warn,server=info,iptv=debug
      HOURLY_UPDATE_FREQUENCY: 4
      GROUP_EXCLUDES: Music,Country,Cou
      PROXY_DOMAIN: myproxydomain.lan
      ENV: production
      XTREAM_ENABLE: "true"
      XTREAM_BASE_DOMAIN: iptvprovider.tv
      XTREAM_USERNAME: username
      XTREAM_PASSWORD: password
      XTREAM_PROXIED_USERNAME: my_new_username
      XTREAM_PROXIED_PASSWORD: my_new_password
    networks:
      - db
networks:
  db:
    external: true
```

## TODO
- Proxy external links in XtreamCodes metadata. For example on player_api.php?username=username&password=password&action=get_series
- Output correct m3u files for XtreamCodes endpoints get.php?username=usernamee&password=password&type=m3u_plus&output=m3u8/ts/rmtm
- Add streaming HLS (.m3u8) support


## DB

SQLX requirements for static type checking:

- Needs to be named DATABASE_URL
- Unfortunately have to have a common connection string for both docker container and host. Since IDE is running from host it needs access to db...
- DATABASE_URL needs to be in .env file to work. Env variable in docker-compose does not work.
- Add 127.0.0.1 host.docker.internal to your hosts file if docker hasn't done that for you

Example: `DATABASE_URL="mysql://db:db@host.docker.internal:3306/iptvhandler"`
