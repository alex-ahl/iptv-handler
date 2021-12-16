# iptv-proxy
## SQLX
Add DATABASE_URL="mysql://db:db@host.docker.internal:3306/iptvproxy" to an .env file

SQLX requirements for static type checking:
- Needs to be named DATABASE_URL
- Unfortunately have to have a common connection string for both docker container and host. Since IDE is running from host it needs access to db...
- DATABASE_URL needs to be in .env file to work. Env variable in docker-compose does not work.
- Add 127.0.0.1 host.docker.internal to your hosts file if docker hasn't done that for you
