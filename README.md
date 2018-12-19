# python-json-read-adapter

This crate implements a `Read` adapter that converts the invalid JSON
tokens `NaN` and `Infinity` into other tokens without otherwise distorting
the stream.  It achieves this by converting `NaN` and `Infinity` into `0.0`.

This is useful because the Python JSON library traditionally emits invalid
JSON if `NaN` and `Infinity` values are encountered.  If you have to support
clients like this, this wrapper can be used to still deserialize such a
JSON document.

This is just a way to get this to parse and `0.0` is the only value that can
be inserted in a standardized way that fits without changing any of the
positions.