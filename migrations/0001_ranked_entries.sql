-- Add migration script here
PRAGMA foreign_keys = OFF;

CREATE TABLE `entries` (`id` INTEGER PRIMARY KEY UNIQUE NOT NULL) STRICT;

CREATE TABLE `entry_ranks` (
    `id` INTEGER PRIMARY KEY AUTOINCREMENT UNIQUE NOT NULL,
    `top_entry` INTEGER NOT NULL REFERENCES `entries` (`id`) ON UPDATE CASCADE ON DELETE CASCADE,
    `bottom_entry` INTEGER NOT NULL REFERENCES `entries` (`id`) ON UPDATE CASCADE ON DELETE CASCADE,
    `ignored` INTEGER NOT NULL CHECK(
        `ignored` = 0
        OR `ignored` = 1
    ) DEFAULT 0
) STRICT;

CREATE UNIQUE INDEX `idx_entry_ranks` ON `entry_ranks` (`top_entry`, `bottom_entry`);

CREATE TRIGGER `unique_rel_direction` BEFORE
INSERT
    ON `entry_ranks` BEGIN
SELECT
    RAISE(ABORT, 'Relation already exists.')
WHERE
    EXISTS (
        SELECT
            1
        FROM
            `entry_ranks`
        WHERE
            `top_entry` = NEW.`bottom_entry`
            AND `bottom_entry` = NEW.`top_entry`
    );

END;

PRAGMA foreign_keys = ON;