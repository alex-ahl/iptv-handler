# Overview

Exclude unwanted channels based on group and generate a new M3U file with proxied streams and attributes. Updates the playlist on specified hourly frequency. Streams get proxied through a webserver running on the app.

## Setup

### _Settable environment variables_

| Variable                | Default     | Type    | Description                                                                            |
| ----------------------- | ----------- | ------- | -------------------------------------------------------------------------------------- |
| DATABASE_URL            | -           | string  | Connection string to DB                                                                |
| M3U                     | -           | string  | URL to the M3U playlist (.m3u)                                                         |
| INIT_APP                | true        | boolean | Initialize app with M3U playlist from environment variable.                            |
| HOURLY_UPDATE_FREQUENCY | 12          | number  | Frequency of provider playlist update in hours                                         |
| GROUP_EXCLUDES          | -           | string  | A comma separated list of groups to exclude from the final playlist. Case-insensitive. |
| PROXY_DOMAIN            | -           | string  | Domain on which the app is running - to proxy m3u requests. (Example: localhost:3000)  |
| ENV                     | Development | string  | Set environment Development or Production.                                             |
| PORT                    | 3001        | number  | Port to run on (Default 3001)                                                          |
| RUST_LOG                | -           | string  | Log level (warn,server=warn,iptv=info,api=warn,rest-client=warn )                      |

### _Production_

To build and run a production docker image.

In src/server folder: `docker build -t image:tag .`

`docker run --env DATABASE_URL=<URL> --env M3U=<M3U-URL> --env PROXY_DOMAIN=localhost:3000 -n iptvhandler image:tag`
<br />
<br />

### _Development_

Set required environment variables and then run `docker-compose up`.

This will fire up a MariaDB container instance and create a new DB.
Which in turn will create tables and basic data using sql files in the `server -> db -> migrations` folder.

#### **DB**

SQLX requirements for static type checking:

- Needs to be named DATABASE_URL
- Unfortunately have to have a common connection string for both docker container and host. Since IDE is running from host it needs access to db...
- DATABASE_URL needs to be in .env file to work. Env variable in docker-compose does not work.
- Add 127.0.0.1 host.docker.internal to your hosts file if docker hasn't done that for you

Example: `DATABASE_URL="mysql://db:db@host.docker.internal:3306/iptvhandler"`
