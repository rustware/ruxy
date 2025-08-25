# About `.ruxy` directory

## Why it exists?

This directory contains Cargo's entrypoints for your project.
You shouldn't edit it by hand. Think of it as Ruxy internals.

## How is this directory updated?

This directory contents are very unlikely to chage over time,
but if it does, Ruxy will provide a codemod to update it in
your project automatically when upgrading to a new version.

## Should I commit it to version control?

**Yes**, please do. This directory is generated only once at project bootstrap.

## Recovery

If you changed or deleted any files in the `.ruxy` directory, or the directory itself,
you can fully restore it by running the following command from your project root:
```
ruxy repair dot-ruxy
```
