# A Records lock coordinates the readers and writers of a build directory

Several processes share one build directory as a matter of course: a `brio` command, the language server (PKG-30), a second build. Lens kept them out of each other's way without any coordination, and three faults remained that no care inside one process removes: a record that is replaced can be missing for a reader that lists the records at that moment, the same on a file system that gives a file no second name, and nothing can tell the unfinished record of a writer that was stopped from that of a writer still at work.

Lens therefore coordinates them with one lock, the Records lock: a file in the build directory that every reader and writer locks, through the file locks of the Rust standard library.

## The promise

The lock file and what its locks mean are part of the layout of a build directory, so that another tool can take part.

- The lock file is `lens.lock`, in the build directory, beside the `lens` directory of the records and never in it.
- A writer makes it, when it is not there, before it changes anything among the records, and writes nothing when it cannot make it. Lens never deletes it. A reader never makes it.
- It is a file, and is never opened through a link. Something else with its name is no lock file: a writer then writes nothing, and a reader reads without a lock.
- The holder of a **shared** lock lists the records, or makes a new record without replacing anything.
- The holder of the **exclusive** lock is alone at work among the records: it may replace or remove files there.
- The lock is advisory, and is released when its holder ends, however it ends.

## How Lens holds it

- A writer holds the shared lock for each new record, from before it makes its unfinished record until that record has its name and the unfinished name is gone. It holds no lock for a whole build: a build with many threads could then never have the exclusive lock.
- A reader that lists every record holds the shared lock while it lists the names, when the lock file is there, and releases it before it reads the files.
- No lock is taken to read one record by its inputs, or to write a record that is there with the same bytes: one file is read by its name, and a file is under a name whole or not at all.
- Where the lock file is there and the lock cannot be taken, readers and writers go on without it, silently, with the care they take of each other without a lock. Records are a cache (PKG-26): a build that cannot run on an unusual file system is worse than one without this guarantee there.
- A lock file that can only be read locks as any other: to be read is enough to be locked.
- No lock type is in the public interface of `brio_lens`. The Engine context may later coordinate the whole build directory and take this lock's place without breaking any phase.

Replacing a record only under the exclusive lock, removing unfinished records under it, and what a reader makes of a directory with no lock file are decided here and built in their own changes (#16).

## Considered Options

- **No lock, and more care.** Rejected. Without a lock nothing can tell a writer that is at work from one that was stopped, and a file cannot be replaced without a moment where a listing misses it.
- **Removing unfinished records by their age.** Rejected. PKG-26 forbids selecting entries by timestamps, and it is unsound: another process can take the name between the look at its age and the removal.
- **A lock file among the records.** Rejected. The records directory holds records and unfinished records alone, and a reader reports anything else. A lock file there would be the first exception, and a copy of the records would carry a lock along.
- **A reader that makes the lock file when it is missing.** Rejected. Reading a reference record set would leave a file in the source tree (PKG-25), and would fail on a checkout that cannot be written to.
- **A time limit on waiting.** Rejected. Readers hold the lock for a listing, a few milliseconds, and a time limit would report a slow disk as a failed build.
- **One lock for the whole build directory, in the Engine context.** Deferred. Lens is the only cache that exists, and `brio_lens` depends on no other workspace crate (ADR-0001).

## Consequences

- A build directory holds one file of Lens beside its records.
- Only programs that take the lock are bound by it.
- A file system that cannot lock keeps the limits Lens had without the lock.
- A reader that hangs in the middle of a listing holds back the holder of the exclusive lock until it ends.
- A writer that waits for the exclusive lock has no promised turn behind an unbroken stream of readers. Removing this needs a second lock file, which was judged heavier than the risk.
- Writing records directly into a reference record set leaves the lock file in the source tree. A set made by copying the records has none.
- A build directory in which no file can be made takes no record, even when its records directory can be written to.
- In a build directory that has a lock file, a record renamed by hand to the name of an unfinished record is passed over. The next build makes the record again.
