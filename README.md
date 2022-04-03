# aws-mfa-profile
Call sts:GetSessionToken to update a specific profile in the credentials file.  

## Usage
If you do not use the `p` option, the `default` profile is used.  
If you do not use the `m` option, the `mfa.json` file is used.  
```sh
$ aws-mfa-profile -h

aws-mfa-profile 

USAGE:
    aws-mfa-profile [OPTIONS]

OPTIONS:
    -c, --credentials-file <CREDENTIALS_FILE>
    -h, --help                 Print help information
    -m, --mfa-file <MFA_FILE>
    -p, --profile <PROFILE>    
```

## MFA_FILE
`MFA_FILE` is json.
```json
[
  {
    "profile": "your profile name in .aws/credentials",
    "serial": "your mfa device id",
    "mfa_profile": "your profile name in .aws/credentials"
  }
]
```

example  
```json
[
  {
    "profile": "default",
    "serial": "default_mfa_device_id",
    "mfa_profile": "mfa"
  },
  {
    "profile": "dev",
    "serial": "dev_mfa_device_id",
    "mfa_profile": "dev_mfa"
  },
  {
    "profile": "prd",
    "serial": "prd_mfa_device_id",
    "mfa_profile": "prd_admin_role"
  }
]
```

## Example
If the `dev_mfa` credential already exists, `aws_access_key_id`, `aws_secret_access_key` and `aws_session_token` will be overwritten.  
The original credentials file is backed up.

```sh
$ cd ~/.aws
$ ls
config credentials mfa.json

$ cat config
[default]
region = us-east-1

[dev]
region = us-east-1

[dev_mfa]
region = us-east-1

$ cat credentials
[default]
aws_access_key_id = hoge_default
aws_secret_access_key = fuga_default

[dev]
aws_access_key_id = hoge_dev
aws_secret_access_key = fuga_dev


$ cat mfa.json
[
  {
    "profile": "dev",
    "serial": "arn:aws:iam::000000000000:mfa/device_id",
    "mfa_profile": "dev_mfa"
  }
]

$ aws-mfa-profile -p dev
[input] token code: 123456
Success! "credentials" file has been updated.

$ cat credentials
[default]
aws_access_key_id = hoge_default
aws_secret_access_key = fuga_default

[dev]
aws_access_key_id = hoge_dev
aws_secret_access_key = fuga_dev

[dev_mfa]
aws_access_key_id = hoge_dev_mfa
aws_secret_access_key = fuga_dev_mfa
aws_session_token = token

$ ls
config credentials credentials.bkp mfa.json
```
