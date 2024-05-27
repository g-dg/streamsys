BEGIN TRANSACTION;

PRAGMA user_version = 1;

CREATE TABLE "config" (
    "id" BLOB PRIMARY KEY NOT NULL DEFAULT (randomblob(16)),
    "key" TEXT NOT NULL UNIQUE ON CONFLICT REPLACE,
    "value_json" TEXT NOT NULL
);

CREATE TABLE "users" (
    "id" BLOB PRIMARY KEY NOT NULL DEFAULT (randomblob(16)),
    "username" TEXT NOT NULL UNIQUE,
    "password" TEXT,
    "description" TEXT NOT NULL DEFAULT "",
    "enabled" INTEGER NOT NULL DEFAULT 1,
    "permissions" INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE "sessions" (
    "id" BLOB PRIMARY KEY NOT NULL DEFAULT (randomblob(16)),
    "token" TEXT NOT NULL UNIQUE,
    "user_id" BLOB NOT NULL,
    "timestamp" TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f+00:00')),
    "valid" INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX "index__sessions__user_id" ON "sessions" ("user_id");

CREATE TABLE "log" (
    "id" BLOB PRIMARY KEY NOT NULL DEFAULT (randomblob(16)),
    "user_id" BLOB,
    "timestamp" TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f+00:00')),
    "action" TEXT NOT NULL,
    "data" TEXT
);


CREATE TABLE "display_outputs" (
    "id" BLOB PRIMARY KEY NOT NULL DEFAULT (randomblob(16)),
    "name" TEXT NOT NULL UNIQUE,
    "template_vue_script" TEXT
);

CREATE TABLE "slide_types" (
    "id" BLOB PRIMARY KEY NOT NULL DEFAULT (randomblob(16)),
    "name" TEXT NOT NULL UNIQUE,
    "template_vue_script" TEXT
);

CREATE TABLE "slide_groups" (
    "id" BLOB PRIMARY KEY NOT NULL DEFAULT (randomblob(16)),
    "parent_group_id" BLOB REFERENCES "slide_groups" ("id") ON UPDATE CASCADE ON DELETE CASCADE,
    "name" TEXT NOT NULL
);
CREATE UNIQUE INDEX "index_null__slide_groups__parent_group_id__name" ON "slide_groups" (IFNULL("parent_group_id", 0), "name");
CREATE INDEX "index__slide_groups__parent_group_id__name" ON "slide_groups" ("parent_group_id", "name");

CREATE TABLE "slide_group_content" (
    "id" BLOB PRIMARY KEY NOT NULL DEFAULT (randomblob(16)),
    "slide_group_id" BLOB NOT NULL REFERENCES "slide_groups" ("id") ON UPDATE CASCADE ON DELETE CASCADE,
    "key" TEXT NOT NULL,
    "content" TEXT,
    UNIQUE("slide_group_id", "key") ON CONFLICT REPLACE
);

CREATE TABLE "slides" (
    "id" BLOB PRIMARY KEY NOT NULL DEFAULT (randomblob(16)),
    "slide_group_id" BLOB REFERENCES "slide_groups" ("id") ON UPDATE CASCADE ON DELETE CASCADE,
    "slide_type_id" BLOB REFERENCES "slide_types" ("id") ON UPDATE CASCADE ON DELETE SET NULL,
    "name" TEXT NOT NULL
);
CREATE UNIQUE INDEX "index_null__slides__slide_group_id__name" ON "slides" (IFNULL("slide_group_id", 0), "name");
CREATE INDEX "index__slides__slide_group_id__name" ON "slides" ("slide_group_id", "name");

CREATE TABLE "slide_content" (
    "id" BLOB PRIMARY KEY NOT NULL DEFAULT (randomblob(16)),
    "slide_id" BLOB NOT NULL REFERENCES "slides" ("id") ON UPDATE CASCADE ON DELETE CASCADE,
    "key" TEXT NOT NULL,
    "content" TEXT,
    UNIQUE("slide_id", "key") ON CONFLICT REPLACE
);

CREATE TABLE "slide_decks" (
    "id" BLOB PRIMARY KEY NOT NULL DEFAULT (randomblob(16)),
    "name" TEXT NOT NULL UNIQUE
);
CREATE TABLE "slide_deck_content_overrides" (
    "id" BLOB PRIMARY KEY NOT NULL DEFAULT (randomblob(16)),
    "slide_deck_id" BLOB NOT NULL REFERENCES "slide_decks" ("id") ON UPDATE CASCADE ON DELETE CASCADE,
    "key" TEXT NOT NULL,
    "content" TEXT,
    UNIQUE("slide_deck_id", "key") ON CONFLICT REPLACE
);

CREATE TABLE "slide_deck_sections" (
    "id" BLOB PRIMARY KEY NOT NULL DEFAULT (randomblob(16)),
    "slide_deck_id" BLOB NOT NULL REFERENCES "slide_decks" ("id") ON UPDATE CASCADE ON DELETE CASCADE,
    "name" TEXT,
    "order" INTEGER NOT NULL DEFAULT 0,
    "slide_group_id" BLOB REFERENCES "slide_groups" ("id") ON UPDATE CASCADE ON DELETE SET NULL,
    "slide_type_override_id" BLOB REFERENCES "slide_types" ("id") ON UPDATE CASCADE ON DELETE SET NULL
);
CREATE INDEX "index__slide_deck_sections__slide_deck_id" ON "slide_deck_sections" ("slide_deck_id");

CREATE TABLE "slide_deck_section_content_overrides" (
    "id" BLOB PRIMARY KEY NOT NULL DEFAULT (randomblob(16)),
    "slide_deck_section_id" BLOB NOT NULL REFERENCES "slide_deck_sections" ("id") ON UPDATE CASCADE ON DELETE CASCADE,
    "key" TEXT NOT NULL,
    "content" TEXT,
    UNIQUE("slide_deck_section_id", "key") ON CONFLICT REPLACE
);

CREATE TABLE "slide_deck_slides" (
    "id" BLOB PRIMARY KEY NOT NULL DEFAULT (randomblob(16)),
    "slide_deck_section_id" BLOB NOT NULL REFERENCES "slide_deck_sections" ("id") ON UPDATE CASCADE ON DELETE CASCADE,
    "name_override" TEXT,
    "order" INTEGER NOT NULL DEFAULT 0,
    "slide_id" BLOB REFERENCES "slides" ("id") ON UPDATE CASCADE ON DELETE SET NULL,
    "slide_type_override_id" BLOB REFERENCES "slide_types" ("id") ON UPDATE CASCADE ON DELETE SET NULL
);
CREATE INDEX "index__slide_deck_slides__slide_deck_section_id" ON "slide_deck_slides" ("slide_deck_section_id");

CREATE TABLE "slide_deck_slide_content_overrides" (
    "id" BLOB PRIMARY KEY NOT NULL DEFAULT (randomblob(16)),
    "slide_deck_slide_id" BLOB NOT NULL REFERENCES "slide_deck_slides" ("id") ON UPDATE CASCADE ON DELETE CASCADE,
    "key" TEXT NOT NULL,
    "content" TEXT,
    UNIQUE("slide_deck_slide_id", "key") ON CONFLICT REPLACE
);

COMMIT;
