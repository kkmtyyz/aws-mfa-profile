# aws-mfa-profile

## Usage
```sh
$ aws-mfa-profile -h

aws-mfa-profile 

USAGE:
    aws-mfa-profile [OPTIONS] --sts-file <STS_FILE> --credentials-file <CREDENTIALS_FILE>

OPTIONS:
    -c, --credentials-file <CREDENTIALS_FILE>
    -h, --help                 Print help information
    -p, --profile <PROFILE>    
    -s, --sts-file <STS_FILE>
```

## STS_FILE
`STS_FILE` is json.
```json
[
  {
    "profile": "your profile name in .aws/credentials",
    "serial": "your mfa device id",
    "sts_profile": "your profile name in .aws/credentials",
  }
]
```

example  
```json
[
  {
    "profile": "dev",
    "serial": "dev_mfa_device_id",
    "sts_profile": "dev",
  },
  {
    "profile": "prd",
    "serial": "prd_mfa_device_id",
    "sts_profile": "prd_admin_role",
  }
]
```
