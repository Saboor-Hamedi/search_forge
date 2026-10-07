  cargo:warning=
  pkg-config exited with status code 1
  > PKG_CONFIG_ALLOW_SYSTEM_CFLAGS=1 pkg-config --libs --cflags glib-2.0 'glib-2.0 >= 2.56'

  pkg-config output:
    Package glib-2.0 was not found in the pkg-config search path.
    Perhaps you should add the directory containing `glib-2.0.pc'
    to the PKG_CONFIG_PATH environment variable
    Package 'glib-2.0', required by 'virtual:world', not found
    Package 'glib-2.0', required by 'virtual:world', not found

  The system library `glib-2.0` required by crate `glib-sys` was not found.
  The file `glib-2.0.pc` needs to be installed and the PKG_CONFIG_PATH environment variable must contain its parent directory.
  The PKG_CONFIG_PATH environment variable is not set.

  HINT: if you have installed the library, try setting PKG_CONFIG_PATH to the directory containing `glib-2.0.pc`.

warning: build failed, waiting for other jobs to finish...
Error: Process completed with exit code 101.
___
Error on line 23 in D:\a\search_forge\search_forge\installer.iss: Source file "D:\a\search_forge\search_forge\target\release\search_forge.exe" does not exist.
Compile aborted.
Move-Item: D:\a\_temp\81b33b10-1f02-4dda-ab12-6b1af80dc825.ps1:13
Line |
  13 |  Move-Item "SearchForge-Setup.exe" "SearchForge-UserSetup-windows-x64. …
     |  ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
     | Cannot find path 'D:\a\search_forge\search_forge\SearchForge-Setup.exe' because it does not exist.
Error: Process completed with exit code 1.
