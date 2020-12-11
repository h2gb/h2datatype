# h2datatype

[![Crate](https://img.shields.io/crates/v/sized_number.svg)](https://crates.io/crates/sized_number)

A library for reading datatypes from, ultimately, a [`Vec<u8>`].

## Overview

`h2datatype` is based on the [`H2Type`] type. An [`H2Type`] represents a
single contiguous chunk of memory with an optional alignment directive.

An [`H2Type`] can be a basic type or a complex type. While these names are
somewhat arbitrary, the essential difference is that basic types are
fundamental building blocks, and complex types are made up of basic types
(and other complex types).

An [`H2Type`] is somewhat abstract: it defines what the type is, how to
calculate its size, how to convert it to a string, and so on. To calculate
any of those, an [`Offset`] is required. An [`Offset`] can either be
abstract (a numeric offset value) or concrete (a buffer of bytes in the form
of a [`sized_number::Context`]). Some types require a concrete buffer to do
anything useful (for example, while the length of an IPv4 value doesn't
change, the length of a UTF-8 character is based on the data).

Pretty much all operations on an [`H2Type`] require an [`Offset`], but
whether can work with a [`Offset::Static`] or [`Offset::Dynamic`] depends on
the implementation.

### Resolving

An [`H2Type`] can also be *resolved*. It's resolved against a particular
[`Offset`], and produces a [`ResolvedType`]. A [`ResolvedType`] has all the
same fields as a [`H2Type`], more or less, but they are now set in stone.
They can be fetched instantly, and have no chance of returning an error or
changing - the field has been resolved.

### Simple types

A simple type, as mentioned above, is defined as a type that's not made up
of other types. The distinction isn't really all that meaningful, it's
simply a logical grouping.

See the various classes in [`crate::basic_type`] for examples!

### Complex types

A complex type is made up of other types. For example, a
[`complex_type::H2Array`] is a series of the same type, a
[`complex_type::H2Struct`] is a series of different types (with names), and
a [`complex_type::H2Enum`] is a choice of overlapping values

#### String types

### Alignment

### Strings

## Examples
