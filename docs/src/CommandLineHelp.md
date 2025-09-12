# Command-Line Help for `tagstudier`

This document contains the help content for the `tagstudier` command-line program.

**Command Overview:**

* [`tagstudier`↴](#tagstudier)
* [`tagstudier manage-folders`↴](#tagstudier-manage-folders)
* [`tagstudier merge-entries`↴](#tagstudier-merge-entries)
* [`tagstudier merge-tags`↴](#tagstudier-merge-tags)
* [`tagstudier mv`↴](#tagstudier-mv)

## `tagstudier`

Tools for TagStudio

**Usage:** `tagstudier [OPTIONS] [COMMAND]`

###### **Subcommands:**

* `manage-folders` — Manage folders based on simple rules
* `merge-entries` — Merge two tags together
* `merge-tags` — Merge two tags together
* `mv` — Move a file within the library, while keeping all the metadata attached

###### **Options:**

* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity
* `-l`, `--library <LIBRARY>` — The path to the TagStudio library. If left blank, it will try to find a library folder in the parent folder recursively



## `tagstudier manage-folders`

Manage folders based on simple rules

**Usage:** `tagstudier manage-folders`



## `tagstudier merge-entries`

Merge two tags together

**Usage:** `tagstudier merge-entries <ENTRY> [ENTRIES_TO_MERGE]...`

###### **Arguments:**

* `<ENTRY>` — The entry to merge into
* `<ENTRIES_TO_MERGE>` — The entry(ies) to merge into the target



## `tagstudier merge-tags`

Merge two tags together

**Usage:** `tagstudier merge-tags <TAG_TARGET> [TAGS_TO_MERGE]...`

###### **Arguments:**

* `<TAG_TARGET>` — The tag to merge into
* `<TAGS_TO_MERGE>` — The tag(s) to merge into the target



## `tagstudier mv`

Move a file within the library, while keeping all the metadata attached

**Usage:** `tagstudier mv [OPTIONS] <FROM> <TO>`

###### **Arguments:**

* `<FROM>` — The file to move
* `<TO>` — Where to move it

###### **Options:**

* `-d`, `--dry` — Add this flag to not make changes, and instead print them out



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

