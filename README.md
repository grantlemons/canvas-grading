# Canvas Grading

A small program used as part of a grading script to download submissions and upload appropriate grades.

## Help Menu:

```
Usage: canvas-grading [OPTIONS] <ASSIGNMENT_ID> <COMMAND>

Commands:
  submissions  Download ungraded submissions and print the paths to standard output
  grade        Upload grades and comments from file
  count        Count the number of submissions meeting a requirement
  help         Print this message or the help of the given subcommand(s)

Arguments:
  <ASSIGNMENT_ID>  Assignment ID in Canvas

Options:
      --access-token <ACCESS_TOKEN>  Override the Canvas access token from config. Either this or the option in config MUST BE SET
  -c, --course-id <COURSE_ID>        Override the course id from config. Either this or the option in config MUST BE SET
  -b, --base-url <BASE_URL>          Override the base URL for Canvas from config. Either this or the option in config MUST BE SET
      --generate <GENERATE>          Generate shell completion [possible values: bash, elvish, fish, powershell, zsh]
  -h, --help                         Print help
  -V, --version                      Print version
```

### `count` Subcommands:

```
Usage: canvas-grading <ASSIGNMENT_ID> count <COMMAND>

Commands:
  unsubmitted  
  submitted    
  graded       
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

The `grade` command reads files in the following format from standard input (angle brackets denote a variable):

```
<CANVAS_USER_ID>: <POINTS>
<CANVAS_USER_ID>: <COMMENT>
```

Points do not need comments, and comments do not need grades.

Multiple comment lines for the same user id will be joined together into a single comment before uploading.

## Config File

Placing a configuration file `grading/config.toml` in the configuration directory for your system allows you to set the options needed to access Canvas.

Passing options will override each individually.

```toml
course_id = <COURSE_ID>
access_token = "<ACCESS_TOKEN>"
base_url = "<CANVAS_URL>"
```
