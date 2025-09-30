-- Add migration script here

PRAGMA foreign_keys = OFF;

CREATE TABLE
    `entries` (`id` INTEGER PRIMARY KEY UNIQUE NOT NULL) STRICT;

CREATE TABLE
    `entry_ranks` (
        `id` INTEGER PRIMARY KEY AUTOINCREMENT UNIQUE NOT NULL,
        `top_entry` INTEGER NOT NULL REFERENCES `entries` (`id`),
        `bottom_entry` INTEGER NOT NULL REFERENCES `entries` (`id`),
        `equal` INTEGER NOT NULL CHECK(`equal` = 0 OR `equal` = 1) DEFAULT 0
    ) STRICT;

CREATE UNIQUE INDEX `idx_entry_ranks` ON `entry_ranks` (`better_entry`, `worse_entry`);

PRAGMA foreign_keys = ON;
