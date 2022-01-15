CREATE TABLE IF NOT EXISTS provider (
    id BIGINT UNSIGNED PRIMARY KEY NOT NULL AUTO_INCREMENT,
    `name` TEXT NOT NULL,
    source TEXT NOT NULL,
    groups TEXT NOT NULL,
    channels TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS extinf (
    id BIGINT UNSIGNED PRIMARY KEY NOT NULL AUTO_INCREMENT,
    `name` TEXT NOT NULL
);


CREATE TABLE IF NOT EXISTS attribute (
    id BIGINT UNSIGNED PRIMARY KEY NOT NULL AUTO_INCREMENT,
    `key` TEXT NOT NULL,
    `value` TEXT NOT NULL,
     provider_id BIGINT UNSIGNED, 
     FOREIGN KEY (provider_id) REFERENCES provider(id)
);

INSERT INTO provider VALUES (1, "temp","https://iptv-org.github.io/iptv/countries/se.m3u","temp","temp");
INSERT INTO attribute VALUES (1, "key", "value", 1)