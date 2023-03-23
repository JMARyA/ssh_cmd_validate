# ssh_cmd_validate
`ssh_cmd_validate` is a cli-tool to limit what a user can execute over ssh.

## Usage
To use this tool add this to `sshd_config`:
```
Match User user
    ForceCommand ssh_cmd_validate config.json
```

## Configuration
The configuration is done in a JSON config file which is provided in the cli arguments.

Possible values to configure are:

- `log_file`: If this key is set, enables logging to the provided file
- `default_command`: This command is run when no command is given to ssh
- `allowed_commands`: a list containing objects describing commands

Every object inside `allowed_commands` can contain:
- `executable`: Absolute path to the executable that should be allowed
- `force_arguments`: a list of arguments which always overwrite the ones by the user (if this key is not set, the user can provide their own)

```json
{
    "log_file": "/ssh.log",
    "default_command": "uname -a",
    "allowed_commands": [
        {
            "executable": "/usr/bin/pacman",
            "force_arguments": ["-Syu"]
        },
        {
            "executable": "/bin/ls",
        }
    ]
}
```
