CREATE TABLE IF NOT EXISTS provider (
    id BIGINT UNSIGNED PRIMARY KEY NOT NULL AUTO_INCREMENT,
    `name` TEXT,
    source TEXT NOT NULL,
    groups INT UNSIGNED,
    channels INT UNSIGNED
);

CREATE TABLE IF NOT EXISTS m3u (
    id BIGINT UNSIGNED PRIMARY KEY NOT NULL AUTO_INCREMENT,
    provider_id BIGINT UNSIGNED, 
    FOREIGN KEY (provider_id) REFERENCES provider(id)
);

CREATE TABLE IF NOT EXISTS extinf (
    id BIGINT UNSIGNED PRIMARY KEY NOT NULL AUTO_INCREMENT,
    `name` TEXT NOT NULL,
    `url` TEXT NOT NULL, 
     m3u_id BIGINT UNSIGNED, 
     FOREIGN KEY (m3u_id) REFERENCES m3u(id)
);


CREATE TABLE IF NOT EXISTS attribute (
    id BIGINT UNSIGNED PRIMARY KEY NOT NULL AUTO_INCREMENT,
    `key` TEXT NOT NULL,
    `value` TEXT NOT NULL,
     extinf_id BIGINT UNSIGNED, 
     FOREIGN KEY (extinf_id) REFERENCES extinf(id)
);

INSERT INTO provider VALUES (1, "temp", "https://iptv-org.github.io/iptv/countries/se.m3u", 0, 0);
INSERT INTO m3u VALUES (1, 1);
INSERT INTO extinf VALUES (1, "Channel name 1", "https://google.se", 1);
INSERT INTO extinf VALUES (2, "Channel name 2", "https://google.se", 1);
INSERT INTO attribute VALUES (1, "key 1", "value 1", 1);
INSERT INTO attribute VALUES (2, "key 2", "value 2", 1); 


