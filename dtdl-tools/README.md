# DTDL Tools

DTDL is fundamental to Ibeji. These tools will help developers to use DTDL to build their own in-vehicle digital twin model.

## DTDL Validator

The DTDL Validator is an application that validates DTDL files. It uses the .NET DTDLParser.

The DTDL Validator application is built by Cargo. It can be found here: ibeji/target/debug/dtdl-validator/bin/Debug/net7.0/dtdl-validator.
It takes two command line arguments:

* -d {directory name}  The directory that contains the DTDL files.
* -e {file extension}  The file extension used by the DTDL files. The default is "json".

The CI/CD pipeline automatically validates DTDL files found under the ibeji/digital-twin-model/dtdl directory via dtdl-tools
test suite. Additional directories containing DTDL files can also be checked by adding new test cases based on the one for
the ibeji/digital-twin-model/dtdl directory.

If you wish to manually run the the DTDL Validator application, then install it from Cargo's out directory to a custom directory by
using the ibeji/dtdl-tools/scripts/install-dtdl-validator.sh script. This script takes two command line arguments:

* -s {source directory}  The Cargo out directory where the application was built. The default is the current directory.
* -d {destination directory}  The custom directory where you want the application installed. The default is "$HOME/.cargo/bin/dtdl-validator".
