# Write

The log had the page. Compact copied
the notebook. That was a pager hiding
in the WAL. The arena is still the
pool. The page is not a WAL record.

A content key leaves for a blob. The
record keeps a serial. Serials do not
overwrite. A pin of an old Pos still
names the old file. That is isolation
for the body, the same rule as the
prefix.

Order:

1. Write the blob. fsync the file.
2. fsync the blob directory.
3. Append the WAL record. The bytes
   of the page are not in it.
4. fsync the log if this session is
   durable.

A crash after 2, before 3: an orphan
file. The prefix does not name it.
Compact deletes files the prefix does
not name.

A crash after 3, before 4: a torn tail.
The blob is ahead of the prefix. The
prefix wins. The file waits for GC.

WAL-first would let a Commit name a
page that is not there. The reader
who honors that Pos would see a hole.
Blob-first is no-steal for the page.
The log is still the truth for what
exists.

Replica copies the blob directory
with the log. Lag is still a Pos.
A serial the prefix does not name
is not read.

Do not make a buffer pool. Do not
steal a dirty page. Do not put the
body back in the record to make
replay one file. Replay is topology.
Hydrate is a second walk.

See `docs/content.md`, `docs/six.md`.
