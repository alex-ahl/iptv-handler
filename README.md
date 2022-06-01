# IPTV Handler

Filter out M3U entires from existing playlist and then generate and proxify a new M3U playlist.

## Stack

Written in RUST with MariaDB for persistance.
Using Docker for development and shipping.
Pure SQL using SQLX as fully async driver.

## Setup

### _Development_

Change db-sync entry in docker-sync.yaml to existing folder on disc. then run `docker-compose up`.

This will fire up a MariaDB server and create a new DB.
Which in turn will create tables and basic data using sql files in the `server -> db -> migrations` folder.

## Good to know

Add DATABASE_URL="mysql://db:db@host.docker.internal:3306/iptvhandler" to an .env file

**SQLX** requirements for static type checking of SQL queries:

- Needs to be named DATABASE_URL
- Unfortunately have to have a common connection string for both docker container and host. Since IDE is running from host it needs access to db...
- DATABASE_URL needs to be in .env file to work. Env variable in docker-compose does not work.
- Add 127.0.0.1 host.docker.internal to your hosts file if docker hasn't done that for you
