# DTDL Tools

DTDL is fundamental to Ibeji. These tools will help devleopers to use DTDL to build their own in-vehicle digital twin model.

## DTDL Validator

DTDL Validator is an application that validates DTDL files. It uses the .Net DTDLParser.

The application is built by Cargo. It can be found here: ibeji/target/debug/dtdl-validator/bin/Debug/net7.0/dtdl-validator.
It takes two command line arguments:
* -d <directory name> - The directory that contains the DTDL files.
* -e <file extension> - The file extension used by the DTDL files. The default is "json".

The CI/CD pipeline automatically validates DTDL files found under the ibeji/digital-twin-model/dtdl directory via dtdl-tools
test suite. Additonal directories containing DTDL files can also be checked by addining new test cases based on the one for
the ibeji/digital-twin-model/dtdl directory.
