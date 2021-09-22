# rusdb-core

Database driver for `rusdb`, which uses gRPC to communicate.

## A few notes

This is NOT production ready, and likely will never be. I'm exploring writing a database, and solving the problems that come up.

There is NO AUTHENTICATION. I plan to add something like this in the future, but for the time being, there is nothing there.

## Usage

This crate provides the client side proto builds, a client implementation, and helper structs for easily manipulating collections and documents.

```rs
use rusdb_core;
```

This crate also re-exports a few crates that are useful: 
* `tonic` - The gRPC implementation
* `bson` - The encoding used for the data being passed between server and client, as well as for storage on disk.