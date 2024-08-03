## How to install

1. [Download the latest release for your operating system here](https://github.com/ewelborn/Medusa-Programming-Language-Classic/releases) and unzip it somewhere on your device.
2. If you're using Windows, make sure that the Windows SDK is installed on your device; [you can download the Windows SDK here](https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/). If you have Visual Studio installed on your device, then it's possible that the Windows SDK has already been installed. The compiler will warn you if the Windows SDK is missing.

## How to use

There are two ways to compile programs on the Medusa compiler:

1. From the commandline, pass in the source file you want to compile as the first argument, then (optionally) pass in the name of the output file as the second argument. If the output file name is missing, then the name of the source file will be used. *Or...*
2. From your desktop or file explorer, drag and drop your source file onto the compiler's .exe file (The source file **must** be in the same directory as the executable). The output file will be produced in the same directory with the same name as the source file.

Regardless of which method you use, if compilation was successful, you will receive a .exe of your compiled Medusa program along with several intermediate files that may be ignored or discarded.