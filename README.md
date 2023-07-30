# Domeneshop CLI

A CLI wrapper for the [Domeneshop API](https://api.domeneshop.no/docs/).

## Prerequisites

You need to generate API-credentials for the Domeneshop API. See their [Authentication](https://api.domeneshop.no/docs/#section/Authentication)-section for more information.

## Usage

`./domeneshop_cli [--flags] <command> <subcommand>`

Use `./domeneshop_cli --help` for an exhaustive list of options.

The CLI will accept credentials using the flags `--token` and `--secret`. As a fallback, the CLI will look for a file, `credentials.json`, in its data directory (current directory if not otherwise specified), on the format:

```json
{
  "token": "<token>",
  "secret": "<secret>"
}
```

Other accepted flags:

- `--data-directory <DIRECTORY>`: Directory to use for auxiliary files
- `--log-directory <DIRECTORY>`: Directory to use for execution logs
- `--debug`: Prints additional debug information, and routes the logs to the console in addition to log files

**NOTE**: The CLI does not support adding new DNS records or forwards as of now.
