# intermodal

A common structure into which data is embedded that provides a manifest
describing the type or schema of the data as well as other contextualizing
metadata.

The `intermodal` format defines two sections within a data structure:
- The `manifest` section contains a description of the data
- The `content` section contains the data itself

There are several goals for this strategy:
- Provide a standardized way data in which data is annotated.
- Allow data handling systems to confidently deserialize messages without
  knowing the precise schema of the data (in statically typed systems).
- Provide a generic mechanism for communicating arbitrary contextual
  information from a data source to a data handler outside of the schema of the
  data itself (decoupling representation and handling logic).

The manifest consists of the following fields:
- `domain` is a DNS-compatible name describing the owner of or organization
  responsible for the implementation of the data's schema
- `scope` is an arbitrary string that acts as a namespace
- `kind` is the name of the schema or type
- `version` is the incremental version number of that schema or type
- `origin` is an identifier describing the origination point of the data
- `ctime` is a timestamp of when the message was created
- `labels` is a set of zero or more arbitrary key-value string pairs

The combination of `domain`, `scope`, `kind`, and `version` should be
sufficient for application code to make a decision on how to route a message or
deserialize the data into a specific type. The `scope` attribute is an
arbitrary string used as a namespacing element between the `domain` and the
`kind`, and is conventionally formatted as a path (using a forward-slash
separator, `/`). This configuration is not intended to be used as a URI
identifying a particular library or schema definition file, nor is that
specifically excluded. At the present, those identifying elements are merely
strings that are interpreted by application code in whatever manner it sees
fit.

The `ctime` attribute is a timestamp, conventionally in the UTC time zone and
formatted as RFC 3339, that describes the creation time of the message itself.
Note that this _does not_ mean to assert the creation time of the data content
itself. Application code is responsible for either choosing to interpret the
`ctime` attribute as the data's creation time or supplying its own more precise
data creation time within the schema of the content.

The `labels` attribute is optional, and typically omitted from messages when
empty. It carries any arbitrary key-value string pair. Its typical case is
conveying informational context around some data (for instance, to display to a
user) or for providing instructions to handler routines specific to that type
of data. Note that in this latter case, the coupling between originator and
downstream handler is in no way facilitated by `intermodal` beyond provision of
the `labels`. For instance, if a given message represents some incomplete
portion of a complete data set, but the application constraints require
transmission of the incomplete set, the client code may choose to add a
sequence number and a boolean marker to the `labels` to inform the processing
system that it needs to buffer the message until the complete set has been
transmitted.

```
manifest:
  domain: foo.org
  scope: examples
  kind: numbers_under_ten
  version: 1
  origin: test
  ctime: 2020-08-25T16:02:20Z
  labels:
    sequence: "0"
    complete: "false"
content:
  numbers: [0, 1, 2, 3, 4, 5]
---
manifest:
  domain: foo.org
  scope: examples
  kind: numbers_under_ten
  version: 1
  origin: test
  ctime: 2020-08-25T16:02:20Z
  labels:
    sequence: "1"
    complete: "true"
content:
  numbers: [6, 7, 8, 9]
```
