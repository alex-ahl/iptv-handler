# IPTV Handler

Filter out M3U entires from existing playlist and then generate a new M3U playlist.

## Stack

Written in RUST.  
Using Docker for production and development (with docker-compose)  
Pure SQL using SQLX as fully async driver.

## Setup

### _Settable environment variables_

```bash
- ENV - Set environment Development of Production. Defaults to Development.
- DATABASE_URL - Connection string to your DB
- M3U - URL to the M3U playlist (.m3u)
- BACKEND_MODE_ONLY - If true, initialize app with M3U playlist environment variable. Requires M3U variable to be set and a valid URL.
- HOURLY_UPDATE_FREQUENCY - Frequency of provider playlist update. Defaults to every 12 hours.
- GROUP_EXCLUDES - A comma separated list of groups to exclude from the final playlist. Checks if the substring provided is part of the group title. Case-insensitive.
- PORT - Port to run on (Default 3001)
- RUST_LOG - Log level (Example: warn,server=warn,iptv=info,api=warn,rest-client=warn)
- DEV_DB_PATH - For development with docker-sync. A path on disc to store development database data.
```

### _Production_

To build and run a production docker image.

In src/server folder: `docker build -t image:tag .`

`docker run --env DATABASE_URL=<URL> --env M3U=<M3U-URL> --env BACKEND_MODE_ONLY=<true/false> -n iptvhandler image:tag`

### _Development_

Change db-sync entry in docker-sync.yaml to existing folder on disc.

Run `docker-sync start` and then run `docker-compose up`.

This will fire up a MariaDB container instance and create a new DB.
Which in turn will create tables and basic data using sql files in the `server -> db -> migrations` folder.

## DB

SQLX requirements for static type checking:

- Needs to be named DATABASE_URL
- Unfortunately have to have a common connection string for both docker container and host. Since IDE is running from host it needs access to db...
- DATABASE_URL needs to be in .env file to work. Env variable in docker-compose does not work.
- Add 127.0.0.1 host.docker.internal to your hosts file if docker hasn't done that for you

Example: `DATABASE_URL="mysql://db:db@host.docker.internal:3306/iptvhandler"`
